#!/usr/bin/env python3
"""Verify package archives before publication with the full local dependency closure."""

import json
import subprocess
import tarfile

from release_crates import ROOT, PUBLISH_ORDER, cargo_metadata, workspace_packages
from pathlib import Path


def package_command(name: str, packages: dict) -> list[str]:
    command = ["cargo", "package", "-p", name, "--allow-dirty"]
    for dependency, package in sorted(packages.items()):
        if dependency == name:
            continue
        path = Path(package["manifest_path"]).parent
        # Cargo config is TOML: JSON quoting handles spaces and literal backslashes.
        command += ["--config", f"patch.crates-io.{dependency}.path={json.dumps(str(path))}"]
    return command


def validate_archive(archive: Path, name: str, version: str, expected_readme: bytes) -> None:
    with tarfile.open(archive, "r:gz") as package:
        if any(Path(item.name).name in ("eth.webp", "eth.png") for item in package):
            raise ValueError(f"{name}: remote logo must not be bundled")
        readme = package.extractfile(f"{name}-{version}/README.md")
        if readme is None or readme.read() != expected_readme:
            raise ValueError(f"{name}: packaged README differs from source")


def main() -> int:
    metadata = cargo_metadata()
    packages = workspace_packages(metadata)
    if set(packages) != set(PUBLISH_ORDER):
        raise ValueError("package verification order is out of date")
    for name in PUBLISH_ORDER:
        subprocess.run(package_command(name, packages), cwd=ROOT, check=True)
        package = packages[name]
        version = package["version"]
        readme = Path(package["manifest_path"]).parent / "README.md"
        validate_archive(Path(metadata["target_directory"]) / "package" / f"{name}-{version}.crate",
                         name, version, readme.read_bytes())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
