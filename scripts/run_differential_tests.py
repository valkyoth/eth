#!/usr/bin/env python3
"""Run differential checks against independent Ethereum implementations."""

from __future__ import annotations

import argparse
import subprocess
import sys
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
CLIENT_DIFFERENTIAL = ["scripts/run_modexp_client_differential.py"]


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
        run([*CLIENT_DIFFERENTIAL, "--check"])
        print(
            f"validated {len(DIFFERENTIAL_TESTS)} in-process paths and "
            "the external-client differential path"
        )
        return 0

    for command in DIFFERENTIAL_TESTS:
        run(command)
    run(CLIENT_DIFFERENTIAL)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        print(
            f"error: command failed with status {error.returncode}: {error.cmd[0]}",
            file=sys.stderr,
        )
        raise SystemExit(error.returncode) from None
