#!/usr/bin/env python3
"""Run differential checks against independent Ethereum implementations."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIFFERENTIAL_TESTS = [
    [
        "cargo",
        "test",
        "-p",
        "eth-valkyoth-codec",
        "--test",
        "differential_rlp_reference",
        "--features",
        "testing",
    ],
    [
        "cargo",
        "test",
        "-p",
        "eth-valkyoth-evm-core",
        "--test",
        "modexp_differential",
    ],
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
        for command in DIFFERENTIAL_TESTS:
            run([*command, "--no-run"])
        print(f"validated {len(DIFFERENTIAL_TESTS)} differential reference paths")
        return 0

    for command in DIFFERENTIAL_TESTS:
        run(command)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
