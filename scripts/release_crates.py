#!/usr/bin/env python3
"""Publish eth workspace crates in crates.io dependency order."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise


ROOT = Path(__file__).resolve().parents[1]

PUBLISH_ORDER = (
    "eth-valkyoth-primitives",
    "eth-valkyoth-codec",
    "eth-valkyoth-protocol",
    "eth-valkyoth-verify",
    "eth-valkyoth-evm",
    "eth-valkyoth-rpc",
    "eth-valkyoth-signer",
    "eth-valkyoth-reth",
    "eth-valkyoth-testkit",
    "eth",
)


def run(command: list[str], *, dry_run: bool) -> None:
    print(f"+ {' '.join(command)}", flush=True)
    if dry_run:
        return
    subprocess.run(command, cwd=ROOT, check=True)


def capture(command: list[str]) -> str:
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def workspace_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        manifest = tomllib.load(handle)
    return manifest["workspace"]["package"]["version"]


def cargo_metadata() -> dict:
    raw = capture(["cargo", "metadata", "--format-version", "1", "--no-deps"])
    return json.loads(raw)


def workspace_packages(metadata: dict) -> dict[str, dict]:
    workspace_ids = set(metadata["workspace_members"])
    return {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_ids
    }


def require_clean_tree(*, allow_dirty: bool) -> None:
    if allow_dirty:
        return

    status = capture(["git", "status", "--porcelain"])
    if status:
        print("Refusing to publish from a dirty worktree:", file=sys.stderr)
        print(status, file=sys.stderr)
        print("Commit or stash changes, or pass --allow-dirty.", file=sys.stderr)
        sys.exit(1)


def verify_publish_order(packages: dict[str, dict], version: str) -> None:
    package_names = tuple(packages)
    expected_names = tuple(sorted(PUBLISH_ORDER))
    actual_names = tuple(sorted(package_names))
    if actual_names != expected_names:
        raise RuntimeError(
            "release_crates.py PUBLISH_ORDER is not in sync with workspace "
            f"packages: expected {expected_names}, actual {actual_names}"
        )

    seen: set[str] = set()
    for package_name in PUBLISH_ORDER:
        package = packages[package_name]
        if package["version"] != version:
            raise RuntimeError(
                f"{package_name} is version {package['version']}, expected {version}"
            )

        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            if dependency_name in packages and dependency_name not in seen:
                raise RuntimeError(
                    f"{package_name} depends on {dependency_name}, but "
                    f"{dependency_name} appears later in PUBLISH_ORDER"
                )
        seen.add(package_name)


def check_release_tag(version: str, *, require_tag: bool) -> None:
    tag = f"v{version}"
    try:
        head = capture(["git", "rev-parse", "HEAD"])
        tagged_commit = capture(["git", "rev-list", "-n", "1", tag])
    except subprocess.CalledProcessError:
        message = f"release tag {tag!r} was not found"
        if require_tag:
            print(f"Refusing to publish: {message}.", file=sys.stderr)
            sys.exit(1)
        print(f"Warning: {message}.", file=sys.stderr)
        return

    if head != tagged_commit:
        message = f"HEAD is not tagged as {tag} (HEAD {head}, {tag} {tagged_commit})"
        if require_tag:
            print(f"Refusing to publish: {message}.", file=sys.stderr)
            sys.exit(1)
        print(f"Warning: {message}.", file=sys.stderr)
        return

    print(f"Release tag {tag} points at HEAD.")


def run_preflight(args: argparse.Namespace) -> None:
    if args.skip_checks:
        print("Skipping preflight checks by request.")
        return

    run(["scripts/release_0_3_gate.sh"], dry_run=args.dry_run)
    run(["cargo", "deny", "check"], dry_run=args.dry_run)
    run(["cargo", "audit"], dry_run=args.dry_run)


def selected_steps(start_at: str) -> tuple[str, ...]:
    try:
        index = PUBLISH_ORDER.index(start_at)
    except ValueError as exc:
        raise RuntimeError(f"unknown package for --start-at: {start_at}") from exc
    return PUBLISH_ORDER[index:]


def wait_for_index(package: str, version: str, *, dry_run: bool) -> None:
    print()
    print(f"Published {package} {version}.")
    print(f"Wait until crates.io shows: https://crates.io/crates/{package}/{version}")
    print("Then press Enter to continue with dependent crates.")
    if dry_run:
        print("[dry-run] skipping wait")
        return
    input()
    time.sleep(5)


def publish(package: str, args: argparse.Namespace) -> None:
    command = ["cargo", "publish", "-p", package]
    if args.allow_dirty:
        command.append("--allow-dirty")
    if args.no_verify:
        command.append("--no-verify")
    run(command, dry_run=args.dry_run)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Publish eth workspace crates in crates.io order."
    )
    parser.add_argument(
        "--version",
        default=workspace_version(),
        help="Expected workspace/package version. Defaults to workspace version.",
    )
    parser.add_argument(
        "--start-at",
        default=PUBLISH_ORDER[0],
        choices=PUBLISH_ORDER,
        help="Resume publishing at a package if an earlier step already succeeded.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate publish order and versions, then exit.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print publish commands without running them or waiting.",
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow publishing from a dirty worktree and pass --allow-dirty to cargo.",
    )
    parser.add_argument(
        "--skip-checks",
        action="store_true",
        help="Skip local checks before publishing.",
    )
    parser.add_argument(
        "--no-verify",
        action="store_true",
        help="Pass --no-verify to cargo publish. Use only with a documented reason.",
    )
    parser.add_argument(
        "--require-tag",
        action="store_true",
        help="Refuse to publish unless HEAD matches the v<version> release tag.",
    )
    parser.add_argument(
        "--yes",
        action="store_true",
        help="Do not ask for the initial confirmation.",
    )
    args = parser.parse_args()

    metadata = cargo_metadata()
    packages = workspace_packages(metadata)
    verify_publish_order(packages, args.version)

    if args.check:
        print("release_crates.py publish order is up to date.")
        return 0

    require_clean_tree(allow_dirty=args.allow_dirty or args.dry_run)
    check_release_tag(args.version, require_tag=args.require_tag)

    steps = selected_steps(args.start_at)

    print(f"Workspace root: {ROOT}")
    print(f"Release version: {args.version}")
    print("Publish sequence:")
    for package in steps:
        print(f"  - {package}")
    print()

    if not args.yes:
        answer = input("Type the release version to start publishing: ").strip()
        if answer != args.version:
            print("Version confirmation did not match; aborting.", file=sys.stderr)
            return 1

    run_preflight(args)

    for index, package in enumerate(steps):
        publish(package, args)
        if index != len(steps) - 1:
            wait_for_index(package, args.version, dry_run=args.dry_run)

    print()
    print("Release publish sequence completed.")
    print(f"Recommended follow-up: cargo info eth@{args.version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
