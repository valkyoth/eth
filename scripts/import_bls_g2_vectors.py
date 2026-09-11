#!/usr/bin/env python3
"""Reproduce the admitted CC0 EIP-2537 G2 addition fixture without executing upstream code."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SHA256 = "c4e5efd1d487ccb11aa171e44b40d20228cd6e251a83d967d30cdd0f45c06bf7"
OUTPUT = ROOT / "crates/eth-valkyoth-evm-core/tests/fixtures/bls12_g2_add.txt"
FAILURE_SHA256 = "e7ee1a0d2e67febb0da69310bf0cdd3f3b07cbe55fcf5045852e1025d98ac976"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="pinned EIPs/assets/eip-2537/add_G2_bls.json")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--failures", action="store_true", help="import fail-add_G2_bls.json")
    args = parser.parse_args()
    raw = args.source.read_bytes()
    digest = FAILURE_SHA256 if args.failures else SOURCE_SHA256
    output = OUTPUT.with_name("bls12_g2_add_fail.txt") if args.failures else OUTPUT
    count = 7 if args.failures else 9
    if hashlib.sha256(raw).hexdigest() != digest:
        raise ValueError("EIP-2537 source hash differs from admitted revision")
    vectors = json.loads(raw)
    if len(vectors) != count:
        raise ValueError("unexpected vector count")
    lines = ["# CC0 EIP-2537 add_G2_bls.json; EIPs 582684e2d7d372c09f45777be8ea603e485e9e9d",
             "# name|input|expected (all nine official vectors; no gas-execution claim)"]
    if args.failures:
        lines = ["# CC0 EIP-2537 fail-add_G2_bls.json; EIPs 582684e2d7d372c09f45777be8ea603e485e9e9d",
                 "# name|input|expected-error (all seven official rejection vectors)"]
    for vector in vectors:
        wire = bytes.fromhex(vector["Input"])
        if args.failures:
            lines.append("|".join([vector["Name"], wire.hex(), vector["ExpectedError"]]))
            continue
        expected = bytes.fromhex(vector["Expected"])
        if len(wire) != 512 or len(expected) != 256:
            raise ValueError("unexpected frame length")
        lines.append("|".join([vector["Name"], wire.hex(), expected.hex()]))
    result = "\n".join(lines) + "\n"
    if args.check:
        if output.read_text() != result:
            raise ValueError("committed fixture differs from admitted source")
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(result)
    print(f"verified {count} pinned EIP-2537 G2 addition vectors")


if __name__ == "__main__":
    main()
