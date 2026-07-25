#!/usr/bin/env python3
"""Unit tests for the latest direct-crate checker."""

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check_latest_crates.py"
SPEC = importlib.util.spec_from_file_location("check_latest_crates", CHECKER_PATH)
if SPEC is None or SPEC.loader is None:
    raise SystemExit("could not load check_latest_crates.py")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def metadata(*versions):
    return {
        "versions": [
            {"num": number, "rust_version": rust, "yanked": yanked}
            for number, rust, yanked in versions
        ]
    }


assert CHECKER.numeric_version("2.0.3") == (2, 0, 3)
assert CHECKER.numeric_version("2") is None
assert (
    CHECKER.latest_compatible(
        metadata(
            ("3.0.0", "1.91", False),
            ("2.5.0", "1.90", False),
            ("2.4.0", "1.80", False),
        ),
        (1, 90, 0),
    )
    == "2.5.0"
)
assert (
    CHECKER.latest_compatible(
        metadata(
            ("2.0.0-alpha.1", "1.80", False),
            ("1.9.0", None, False),
            ("1.8.0", "1.70", True),
        ),
        (1, 90, 0),
    )
    == "1.9.0"
)
print("latest direct-crate checker tests passed")
