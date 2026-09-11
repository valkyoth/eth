#!/usr/bin/env python3
"""Regression tests for dependency-free README installation and example checks."""
import unittest
import tempfile
from pathlib import Path
from unittest.mock import patch

import check_readmes

from check_readmes import LOGO, documented_dependencies, validate_readme

PACKAGES = {"eth": {"features": {"evm-core": [], "sanitization": []}},
            "eth-valkyoth-codec": {"features": {}}}
README = (f'<img src="{LOGO}">\nhttps://crates.io/crates/eth\n'
          '```sh\ncargo add eth --features evm-core\n```\n'
          '```rust\nassert_eq!(1, 1);\n```\n')


class ReadmeTests(unittest.TestCase):
    def test_current_example(self):
        self.assertEqual(validate_readme(README, PACKAGES), {"eth": {"evm-core"}})

    def test_features_union_and_multiple_packages(self):
        text = ('```sh\ncargo add eth eth-valkyoth-codec\n'
                'cargo add eth --features evm-core,sanitization\n```')
        self.assertEqual(documented_dependencies(text, PACKAGES),
                         {"eth": {"evm-core", "sanitization"}, "eth-valkyoth-codec": set()})

    def test_rejects_stale_wildcard_and_missing_version_manifests(self):
        for value in ('"0.52.4"', '"*"', '{ features = ["evm-core"] }'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                validate_readme(README + f'```toml\n[dependencies]\neth = {value}\n```', PACKAGES)

    def test_rejects_bad_commands(self):
        for command in ("cargo add eth@0.52.4", "cargo add unknown",
                        "cargo add eth --features typo", "cargo add",
                        "cargo add eth eth-valkyoth-codec --features evm-core",
                        "cargo add eth --features"):
            with self.subTest(command=command), self.assertRaises(ValueError):
                documented_dependencies(f'```sh\n{command}\n```', PACKAGES)

    def test_rejects_missing_logo_facade_example_or_fence(self):
        for text in (README.replace(LOGO, "eth.png"),
                     README.replace("https://crates.io/crates/eth", ""),
                     README.replace("```rust", "```rust,ignore"),
                     README + "```rust\n"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                validate_readme(text, PACKAGES)

    def test_clean_ci_fetches_locked_graph_before_offline_examples(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            crate = root / "crates/eth"
            crate.mkdir(parents=True)
            (root / "README.md").write_text(README)
            (crate / "README.md").write_text(README)
            packages = {"eth": {"manifest_path": str(crate / "Cargo.toml"),
                                "readme": "README.md", **PACKAGES["eth"]}}
            for args, expected in (([], []), (["--test"], ["fetch", "examples"])):
                calls = []
                with self.subTest(args=args), patch("sys.argv", ["readmes", *args]), \
                     patch.object(check_readmes, "ROOT", root), \
                     patch.object(check_readmes, "cargo_metadata", return_value={}), \
                     patch.object(check_readmes, "workspace_packages", return_value=packages), \
                     patch.object(check_readmes.subprocess, "run",
                                  side_effect=lambda *a, **k: calls.append("fetch")) as fetch, \
                     patch.object(check_readmes, "test_examples",
                                  side_effect=lambda *a, **k: calls.append("examples")):
                    self.assertEqual(check_readmes.main(), 0)
                    self.assertEqual(calls, expected)
                    if args:
                        fetch.assert_called_once_with(["cargo", "fetch", "--locked"],
                                                      cwd=root, check=True)


if __name__ == "__main__":
    unittest.main()
