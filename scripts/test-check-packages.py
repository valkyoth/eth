#!/usr/bin/env python3
"""Archive and complete dependency-closure regressions for checkpoint verification."""
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

from check_packages import package_command, validate_archive


class PackageTests(unittest.TestCase):
    def test_includes_transitive_packages_but_not_self(self):
        packages = {name: {"manifest_path": f"/tmp/space path/{name}/Cargo.toml"}
                    for name in ("eth", "direct", "transitive")}
        command = package_command("eth", packages)
        self.assertEqual(command[:5], ["cargo", "package", "-p", "eth", "--allow-dirty"])
        self.assertEqual(command.count("--config"), 2)
        self.assertIn('patch.crates-io.transitive.path=' +
                      json.dumps("/tmp/space path/transitive"), command)
        self.assertFalse(any("patch.crates-io.eth.path" in arg for arg in command))

    def archive(self, path, contents):
        with tarfile.open(path, "w:gz") as archive:
            for name, body in contents.items():
                item = tarfile.TarInfo("eth-0.60.0/" + name)
                item.size = len(body)
                archive.addfile(item, io.BytesIO(body))

    def test_matching_readme_without_bitmap(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "example.crate"
            self.archive(path, {"README.md": b"remote logo"})
            validate_archive(path, "eth", "0.60.0", b"remote logo")

    def test_rejects_changed_missing_readme_or_bundled_logo(self):
        for contents in ({"README.md": b"old"}, {"src/lib.rs": b""},
                         {"README.md": b"current", "eth.webp": b"image"},
                         {"README.md": b"current", ".github/images/eth.png": b"image"}):
            with self.subTest(contents=contents), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "example.crate"
                self.archive(path, contents)
                with self.assertRaises((ValueError, KeyError)):
                    validate_archive(path, "eth", "0.60.0", b"current")


if __name__ == "__main__":
    unittest.main()
