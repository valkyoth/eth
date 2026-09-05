#!/usr/bin/env python3
"""Compare inherited Cargo contracts without modifying the active checkout."""

from __future__ import annotations

import io
import json
import subprocess
import tarfile
import tempfile
from pathlib import Path

from release_evidence import authenticated_tag


def dependency_contract(package: dict, root: Path) -> str:
    dependencies = []
    fields = ("name", "source", "req", "kind", "rename", "optional",
              "uses_default_features", "target", "registry")
    for dependency in package.get("dependencies", []):
        item = {key: dependency.get(key) for key in fields}
        item["features"] = sorted(dependency.get("features", []))
        path = dependency.get("path")
        if path is not None:
            try:
                item["path"] = Path(path).resolve().relative_to(root.resolve()).as_posix()
            except ValueError as error:
                raise RuntimeError("release dependency path escapes its workspace") from error
        dependencies.append(item)
    dependencies.sort(key=lambda item: json.dumps(item, sort_keys=True))
    return json.dumps({"dependencies": dependencies, "features": {
        name: sorted(values) for name, values in package.get("features", {}).items()
    }}, sort_keys=True)


def baseline_contracts(root: Path, baseline: str) -> dict[str, str]:
    commit = authenticated_tag(root, baseline)
    archive = subprocess.check_output(["git", "archive", "--format=tar", commit], cwd=root)
    with tempfile.TemporaryDirectory(prefix="eth-release-contracts-") as directory:
        snapshot = Path(directory)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            # No symlinks, device nodes, submodule execution or archive traversal.
            for member in tar.getmembers():
                path = Path(member.name)
                if (path.is_absolute() or ".." in path.parts
                        or not (member.isfile() or member.isdir())):
                    raise RuntimeError("baseline snapshot contains unsupported archive entry")
            tar.extractall(snapshot, filter="data")
        # CWD stays at the reviewed checkout: archived .cargo configuration and
        # rust-toolchain files cannot select an old wrapper/toolchain to execute.
        # --no-deps performs manifest inspection, not build-script compilation.
        result = subprocess.run(
            ["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1",
             "--manifest-path", str(snapshot / "Cargo.toml")],
            cwd=root, text=True, capture_output=True, check=False,
        )
        if result.returncode:
            raise RuntimeError(f"baseline Cargo metadata failed: {result.stderr.strip()}")
        metadata = json.loads(result.stdout)
        members = set(metadata["workspace_members"])
        return {package["name"]: dependency_contract(package, snapshot)
                for package in metadata["packages"] if package["id"] in members}


def changed_contracts(root: Path, packages: dict[str, dict], baseline: str) -> set[str]:
    old = baseline_contracts(root, baseline)
    return {name for name, package in packages.items()
            if old.get(name) != dependency_contract(package, root)}
