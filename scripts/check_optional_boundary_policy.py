#!/usr/bin/env python3
"""Optional parser and sanitization dependency boundary checks."""

from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ETH_MANIFEST = ROOT / "crates" / "eth" / "Cargo.toml"
VERIFY_MANIFEST = ROOT / "crates" / "eth-valkyoth-verify" / "Cargo.toml"
SANITIZATION_MANIFEST = ROOT / "crates" / "eth-valkyoth-sanitization" / "Cargo.toml"
DEFAULT_FORBIDDEN = frozenset(("eth-valkyoth-sanitization", "sanitization", "serde", "serde_json"))
JSON_REQUIRED = frozenset(("serde", "serde_json"))
SANITIZATION_REQUIRED = frozenset(("eth-valkyoth-sanitization", "sanitization"))
DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def feature(manifest: dict, name: str) -> list[str]:
    features = manifest.get("features")
    if not isinstance(features, dict) or not isinstance(features.get(name), list):
        raise RuntimeError(f"missing feature {name!r}")
    return features[name]


def dependency(manifest: dict, name: str) -> object:
    dependencies = manifest.get("dependencies")
    if not isinstance(dependencies, dict):
        raise RuntimeError("missing [dependencies]")
    return dependencies.get(name)


def optional_dependency(manifest: dict, name: str) -> bool:
    entry = dependency(manifest, name)
    return isinstance(entry, dict) and entry.get("optional") is True


def dependency_feature_enabled(entry: object, package: str, feature_name: str) -> bool:
    if not isinstance(entry, dict):
        return False
    if entry.get("package", package) != package:
        return False
    features = entry.get("features")
    return isinstance(features, list) and feature_name in features


def manifest_dependency_tables(manifest: dict) -> list[dict]:
    tables: list[dict] = []
    for table_name in DEPENDENCY_TABLES:
        table = manifest.get(table_name)
        if isinstance(table, dict):
            tables.append(table)
    targets = manifest.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            if isinstance(target, dict):
                for table_name in DEPENDENCY_TABLES:
                    table = target.get(table_name)
                    if isinstance(table, dict):
                        tables.append(table)
    return tables


def check_serde_json_depth_feature() -> list[str]:
    errors: list[str] = []
    for manifest_path in sorted(ROOT.rglob("Cargo.toml")):
        if "target" in manifest_path.parts:
            continue
        manifest = load_toml(manifest_path)
        for table in manifest_dependency_tables(manifest):
            for name, entry in table.items():
                is_serde_json = name == "serde_json" or (
                    isinstance(entry, dict) and entry.get("package") == "serde_json"
                )
                if is_serde_json and dependency_feature_enabled(
                    entry, "serde_json", "unbounded_depth"
                ):
                    errors.append(
                        f"{manifest_path.relative_to(ROOT)} must not enable serde_json/unbounded_depth"
                    )
    return errors


def cargo_tree(*args: str) -> str:
    return subprocess.check_output(
        ["cargo", "tree", "-p", "eth", *args, "-e", "normal", "--prefix", "none"],
        cwd=ROOT,
        text=True,
    )


def tree_contains(tree: str, package: str) -> bool:
    prefix = f"{package} v"
    return any(line.startswith(prefix) for line in tree.splitlines())


def missing_from_tree(tree: str, packages: frozenset[str]) -> list[str]:
    return sorted(package for package in packages if not tree_contains(tree, package))


def present_in_tree(tree: str, packages: frozenset[str]) -> list[str]:
    return sorted(package for package in packages if tree_contains(tree, package))


def check_feature_wiring() -> list[str]:
    errors: list[str] = []
    eth = load_toml(ETH_MANIFEST)
    verify = load_toml(VERIFY_MANIFEST)
    sanitization = load_toml(SANITIZATION_MANIFEST)

    if feature(eth, "default") != []:
        errors.append("eth default feature set must stay empty")
    if feature(eth, "eip712-json") != ["eth-valkyoth-verify/json"]:
        errors.append("eth/eip712-json must only forward eth-valkyoth-verify/json")
    if feature(eth, "sanitization") != ["dep:eth-valkyoth-sanitization"]:
        errors.append("eth/sanitization must only admit eth-valkyoth-sanitization")
    if not optional_dependency(eth, "eth-valkyoth-sanitization"):
        errors.append("eth-valkyoth-sanitization must stay optional in eth")

    if feature(verify, "json") != ["std", "dep:serde", "dep:serde_json"]:
        errors.append("verify/json must require std, serde, and serde_json explicitly")
    if not optional_dependency(verify, "serde"):
        errors.append("serde must stay optional in eth-valkyoth-verify")
    if not optional_dependency(verify, "serde_json"):
        errors.append("serde_json must stay optional in eth-valkyoth-verify")

    if feature(sanitization, "default") != []:
        errors.append("eth-valkyoth-sanitization default feature set must stay empty")
    if dependency(sanitization, "sanitization") is None:
        errors.append("eth-valkyoth-sanitization must depend on sanitization")

    return errors


def check_graphs() -> list[str]:
    errors: list[str] = []
    default_tree = cargo_tree()
    json_tree = cargo_tree("--no-default-features", "--features", "eip712-json")
    sanitization_tree = cargo_tree("--no-default-features", "--features", "sanitization")

    for package in present_in_tree(default_tree, DEFAULT_FORBIDDEN):
        errors.append(f"default eth graph must not include {package}")
    for package in missing_from_tree(json_tree, JSON_REQUIRED):
        errors.append(f"eth/eip712-json graph must include {package}")
    for package in present_in_tree(json_tree, SANITIZATION_REQUIRED):
        errors.append(f"eth/eip712-json graph must not include {package}")
    for package in missing_from_tree(sanitization_tree, SANITIZATION_REQUIRED):
        errors.append(f"eth/sanitization graph must include {package}")
    for package in present_in_tree(sanitization_tree, JSON_REQUIRED):
        errors.append(f"eth/sanitization graph must not include {package}")

    return errors


def main() -> int:
    errors = check_feature_wiring()
    errors.extend(check_serde_json_depth_feature())
    errors.extend(check_graphs())
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    print("optional parser and sanitization boundaries are explicit")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
