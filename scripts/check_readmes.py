#!/usr/bin/env python3
"""Validate package READMEs and optionally run examples with documented features."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import tomllib
from pathlib import Path

from release_crates import ROOT, cargo_metadata, workspace_packages

LOGO = "https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp"


def fences(text: str):
    language = None
    lines: list[str] = []
    for line in text.splitlines():
        if line.startswith("```"):
            if language is None:
                language = line[3:].strip()
                lines = []
            elif line.strip() == "```":
                yield language, "\n".join(lines)
                language = None
            else:
                raise ValueError("nested Markdown fence")
        elif language is not None:
            lines.append(line)
    if language is not None:
        raise ValueError("unclosed Markdown fence")


def documented_dependencies(text: str, packages: dict) -> dict[str, set[str]]:
    dependencies: dict[str, set[str]] = {}
    for language, body in fences(text):
        if language == "toml":
            parsed = tomllib.loads(body)
            for table in ("dependencies", "dev-dependencies", "build-dependencies"):
                if set(parsed.get(table, {})) & packages.keys():
                    raise ValueError("use cargo add for workspace dependencies, not stale version snippets")
        if language not in ("sh", "bash", "shell"):
            continue
        for line in body.splitlines():
            words = shlex.split(line)
            if words[:2] != ["cargo", "add"]:
                continue
            args = words[2:]
            names: list[str] = []
            features: set[str] = set()
            if "--features" in args:
                index = args.index("--features")
                if len(args) != index + 2:
                    raise ValueError("cargo add example must use one explicit feature list")
                features = set(args[index + 1].replace(",", " ").split())
                args = args[:index]
            for name in args:
                if name not in packages:
                    raise ValueError(f"unknown or version-pinned cargo add dependency: {name}")
                names.append(name)
            if not names or (features and len(names) != 1):
                raise ValueError("ambiguous cargo add example")
            for name in names:
                if not features.issubset(packages[name]["features"]):
                    raise ValueError(f"unknown features for {name}: {features}")
                dependencies.setdefault(name, set()).update(features)
    if not dependencies:
        raise ValueError("README needs a cargo add example")
    return dependencies


def validate_readme(text: str, packages: dict) -> dict[str, set[str]]:
    if f'src="{LOGO}"' not in text:
        raise ValueError("README must use the remote eth.webp logo")
    if "https://crates.io/crates/eth" not in text:
        raise ValueError("README must link to the eth facade")
    if not any(language == "rust" for language, _ in fences(text)):
        raise ValueError("README needs at least one runnable Rust example")
    return documented_dependencies(text, packages)


def test_examples(name: str, readme: Path, dependencies: dict, packages: dict) -> None:
    directory = ROOT / "target" / "readme-tests" / name
    directory.mkdir(parents=True, exist_ok=True)
    manifest = ('[workspace]\n[package]\nname = "readme-' + name +
                '"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n'
                '[lib]\npath = "lib.rs"\n[dependencies]\n')
    for dependency, features in sorted(dependencies.items()):
        path = str(Path(packages[dependency]["manifest_path"]).parent)
        manifest += (f'{dependency} = {{ path = {json.dumps(path)}, '
                     f'features = {json.dumps(sorted(features))} }}\n')
    (directory / "Cargo.toml").write_text(manifest)
    (directory / "lib.rs").write_text(f'#![doc = include_str!({json.dumps(str(readme))})]\n')
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target" / "readme-tests-build"))
    subprocess.run(["cargo", "generate-lockfile", "--offline", "--manifest-path",
                    str(directory / "Cargo.toml")], cwd=ROOT, env=env, check=True)
    subprocess.run(["cargo", "test", "--doc", "--offline", "--locked", "--manifest-path",
                    str(directory / "Cargo.toml")], cwd=ROOT, env=env, check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--test", action="store_true", help="compile and run README doctests")
    args = parser.parse_args()
    packages = workspace_packages(cargo_metadata())
    if (ROOT / "README.md").read_bytes() != (ROOT / "crates/eth/README.md").read_bytes():
        raise ValueError("GitHub and facade READMEs differ")
    examples = []
    for name, package in sorted(packages.items()):
        path = Path(package["manifest_path"]).parent / "README.md"
        if package["readme"] != "README.md":
            raise ValueError(f"{name}: manifest must package its local README.md")
        dependencies = validate_readme(path.read_text(), packages)
        examples.append((name, path, dependencies))
    if args.test:
        # Fresh CI hosts must populate the reviewed graph before offline examples.
        subprocess.run(["cargo", "fetch", "--locked"], cwd=ROOT, check=True)
    for name, path, dependencies in examples:
        if args.test:
            test_examples(name, path, dependencies, packages)
        print(f"{name}: README passed", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
