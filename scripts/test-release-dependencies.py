#!/usr/bin/env python3
"""Effective dependency regression tests using isolated Cargo baseline snapshots."""

import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import release_train
from release_dependencies import dependency_contract
from release_test_support import Repository

ROOT_MANIFEST = '''[workspace]
members = ["crate", "local"]
resolver = "2"
[workspace.dependencies]
syntax = { package = "syn", version = "3.0.3", default-features = false }
target_syntax = { package = "syn", version = "3.0.3", default-features = false }
local = { path = "local", version = "0.1.0" }
'''
CRATE_MANIFEST = '''[package]
name = "fixture"
version = "0.1.0"
edition = "2024"
[dependencies]
syntax = { workspace = true, optional = true }
local.workspace = true
[target.'cfg(unix)'.build-dependencies]
target_syntax.workspace = true
[features]
parser = ["dep:syntax"]
'''


class DependencyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Repository(Path(self.temp.name) / "repo")
        self.repo.write("Cargo.toml", ROOT_MANIFEST)
        self.repo.write("crate/Cargo.toml", CRATE_MANIFEST)
        self.repo.write("crate/src/lib.rs", "// fixture\n")
        self.repo.write("local/Cargo.toml", '[package]\nname="local"\nversion="0.1.0"\n')
        self.repo.write("local/src/lib.rs", "// dependency fixture\n")
        self.repo.report("0.55.0")

    def packages(self):
        raw = subprocess.check_output(["cargo", "metadata", "--offline", "--no-deps",
                                       "--format-version", "1"], cwd=self.repo.root, text=True)
        return {p["name"]: p for p in json.loads(raw)["packages"]}

    def test_matching_contracts_ignore_checkout_path(self):
        with patch.object(release_train, "ROOT", self.repo.root):
            packages = self.packages()
            before = self.repo.git("status", "--porcelain")
            self.assertEqual(release_train.changed_packages(packages, "0.55.0"), set())
            self.assertEqual(self.repo.git("status", "--porcelain"), before)

    def test_archive_toolchain_cannot_control_metadata_execution(self):
        self.repo.git("tag", "-d", "v0.55.0")
        self.repo.write("rust-toolchain.toml", '[toolchain]\nchannel="not-a-real-toolchain"\n')
        self.repo.report("0.55.0")
        (self.repo.root / "rust-toolchain.toml").unlink()
        with patch.object(release_train, "ROOT", self.repo.root):
            self.assertEqual(release_train.changed_packages(self.packages(), "0.55.0"), set())

    def test_root_only_requirement_features_and_target_alias_change(self):
        changes = (
            ROOT_MANIFEST.replace('version = "3.0.3"', 'version = "3.0.5"', 1),
            ROOT_MANIFEST.replace('default-features = false', 'default-features = false, features = ["full"]', 1),
            ROOT_MANIFEST.replace('default-features = false', 'default-features = true', 1),
            ROOT_MANIFEST.replace('target_syntax = { package = "syn", version = "3.0.3"',
                                  'target_syntax = { package = "syn", version = "3.0.5"'),
        )
        for manifest in changes:
            with self.subTest(manifest=manifest), patch.object(release_train, "ROOT", self.repo.root):
                self.repo.write("Cargo.toml", manifest)
                packages = self.packages()
                self.assertEqual(release_train.changed_packages(packages, "0.55.0"), {"fixture"})
                plan = dict(stage="public", anchor=False, baseline="0.55.0", crates={
                    name: dict(change="unchanged", previous_version="0.1.0", version="0.1.0")
                    for name in packages})
                with self.assertRaisesRegex(RuntimeError, "changed after"):
                    release_train.validate_cumulative_package_changes(packages, plan)
                plan["crates"]["fixture"]["change"] = "dependency"
                plan["crates"]["fixture"]["version"] = "0.1.1"
                release_train.validate_cumulative_package_changes(packages, plan)

    def test_every_dependency_identity_field_and_feature_map_matters(self):
        package = self.packages()["fixture"]
        original = dependency_contract(package, self.repo.root)
        for name, value in (("name", "different"), ("source", "git+https://example.invalid/repo"),
                            ("req", "^9.0"), ("kind", "dev"), ("rename", "other"),
                            ("optional", False), ("uses_default_features", True),
                            ("target", "cfg(windows)"), ("registry", "https://example.invalid/index"),
                            ("features", ["extra"])):
            changed = copy.deepcopy(package)
            dependency = next(d for d in changed["dependencies"] if d["rename"] == "syntax")
            dependency[name] = value
            with self.subTest(field=name):
                self.assertNotEqual(original, dependency_contract(changed, self.repo.root))
        changed = copy.deepcopy(package)
        changed["features"]["new"] = ["parser"]
        self.assertNotEqual(original, dependency_contract(changed, self.repo.root))

    def test_unsigned_baseline_rejected(self):
        self.repo.git("tag", "-d", "v0.55.0")
        self.repo.git("tag", "v0.55.0")
        with patch.object(release_train, "ROOT", self.repo.root), self.assertRaises(RuntimeError):
            release_train.changed_packages(self.packages(), "0.55.0")


if __name__ == "__main__":
    unittest.main()
