#!/usr/bin/env python3
"""Runtime dependency policy checks for the eth workspace."""

from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"

DEFAULT_FORBIDDEN = frozenset(
    (
        "alloy-rlp",
        "k256",
        "revm",
        "revm-primitives",
        "sanitization",
        "serde",
        "serde_json",
        "sha3",
        "tiny-keccak",
    )
)
REFERENCE_ORACLES = frozenset(("alloy-rlp",))
FIXTURE_PARSERS = frozenset(("serde_json",))


def fail(message: str) -> int:
    print(f"error: {message}", file=sys.stderr)
    return 1


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def dependency_names(table: object) -> set[str]:
    if not isinstance(table, dict):
        return set()
    return {name for name in table if isinstance(name, str)}


def crate_manifests() -> list[Path]:
    return sorted(CRATES.glob("*/Cargo.toml"))


def package_name(manifest: dict, path: Path) -> str:
    package = manifest.get("package")
    if not isinstance(package, dict) or not isinstance(package.get("name"), str):
        raise RuntimeError(f"{path} is missing [package].name")
    return package["name"]


def dependency_entry(table: object, name: str) -> object:
    if not isinstance(table, dict):
        return None
    return table.get(name)


def is_optional(entry: object) -> bool:
    return isinstance(entry, dict) and entry.get("optional") is True


def check_reference_dependency_manifests() -> list[str]:
    errors: list[str] = []
    for path in crate_manifests():
        manifest = load_toml(path)
        package = package_name(manifest, path)
        normal = manifest.get("dependencies")
        build = manifest.get("build-dependencies")
        dev = manifest.get("dev-dependencies")

        normal_names = dependency_names(normal)
        build_names = dependency_names(build)
        dev_names = dependency_names(dev)

        for name in sorted(REFERENCE_ORACLES.intersection(normal_names | build_names)):
            errors.append(f"{package} may not use reference oracle {name} at runtime")
        for name in sorted(REFERENCE_ORACLES.intersection(dev_names)):
            if package != "eth-valkyoth-codec":
                errors.append(f"{package} may not use reference oracle {name} as a dev dependency")

        for name in sorted(FIXTURE_PARSERS.intersection(dev_names)):
            if package != "eth-valkyoth-codec":
                errors.append(f"{package} may not use fixture parser {name} as a dev dependency")
        for name in sorted(FIXTURE_PARSERS.intersection(normal_names)):
            entry = dependency_entry(normal, name)
            if package != "eth-valkyoth-verify" or not is_optional(entry):
                errors.append(f"{package} may not use fixture parser {name} at runtime")

    return errors


def cargo_tree_default_runtime() -> str:
    return subprocess.check_output(
        [
            "cargo",
            "tree",
            "-p",
            "eth",
            "--no-default-features",
            "-e",
            "normal",
            "--prefix",
            "none",
        ],
        cwd=ROOT,
        text=True,
    )


def tree_contains_package(tree: str, package: str) -> bool:
    prefix = f"{package} v"
    return any(line.startswith(prefix) for line in tree.splitlines())


def check_default_runtime_tree() -> list[str]:
    tree = cargo_tree_default_runtime()
    return [
        f"default eth runtime graph must not include {package}"
        for package in sorted(DEFAULT_FORBIDDEN)
        if tree_contains_package(tree, package)
    ]


def main() -> int:
    errors = check_reference_dependency_manifests()
    errors.extend(check_default_runtime_tree())
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1

    print("runtime dependency policy keeps reference crates out of default runtime")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
