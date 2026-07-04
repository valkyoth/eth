#!/usr/bin/env python3
"""Run differential checks against independent Ethereum implementations."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIFFERENTIAL_TEST = [
    "cargo",
    "test",
    "-p",
    "eth-valkyoth-codec",
    "--test",
    "differential_rlp_reference",
    "--features",
    "testing",
]


def run(command: list[str]) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate that the differential harness is configured",
    )
    args = parser.parse_args()

    if args.check:
        run([*DIFFERENTIAL_TEST, "--no-run"])
        print("validated 1 differential reference path")
        return 0

    run(DIFFERENTIAL_TEST)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
