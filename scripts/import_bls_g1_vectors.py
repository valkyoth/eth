#!/usr/bin/env python3
"""Reproduce the admitted CC0 EIP-2537 G1 addition fixture without executing upstream code."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SHA256 = "224b9b22ddf72efb0cd9c8ffb6cd742d68af7b062e592e8b8a5378a22d6f9d02"
OUTPUT = ROOT / "crates/eth-valkyoth-evm-core/tests/fixtures/bls12_g1_add.txt"
FAILURE_SHA256 = "92a85348a2c172d6e12ce858566febdc9f8794f542b2a18826fa3f7170a62bfd"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="pinned EIPs/assets/eip-2537/add_G1_bls.json")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--failures", action="store_true", help="import fail-add_G1_bls.json")
    args = parser.parse_args()
    raw = args.source.read_bytes()
    digest = FAILURE_SHA256 if args.failures else SOURCE_SHA256
    output = OUTPUT.with_name("bls12_g1_add_fail.txt") if args.failures else OUTPUT
    count = 7 if args.failures else 9
    if hashlib.sha256(raw).hexdigest() != digest:
        raise ValueError("EIP-2537 source hash differs from admitted revision")
    vectors = json.loads(raw)
    if len(vectors) != count:
        raise ValueError("unexpected vector count")
    lines = ["# CC0 EIP-2537 add_G1_bls.json; EIPs 582684e2d7d372c09f45777be8ea603e485e9e9d",
             "# name|input|expected (all nine official vectors; no gas-execution claim)"]
    if args.failures:
        lines = ["# CC0 EIP-2537 fail-add_G1_bls.json; EIPs 582684e2d7d372c09f45777be8ea603e485e9e9d",
                 "# name|input|expected-error (all seven official rejection vectors)"]
    for vector in vectors:
        wire = bytes.fromhex(vector["Input"])
        if args.failures:
            lines.append("|".join([vector["Name"], wire.hex(), vector["ExpectedError"]]))
            continue
        expected = bytes.fromhex(vector["Expected"])
        if len(wire) != 256 or len(expected) != 128:
            raise ValueError("unexpected frame length")
        lines.append("|".join([vector["Name"], wire.hex(), expected.hex()]))
    result = "\n".join(lines) + "\n"
    if args.check:
        if output.read_text() != result:
            raise ValueError("committed fixture differs from admitted source")
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(result)
    print(f"verified {count} pinned EIP-2537 G1 addition vectors")


if __name__ == "__main__":
    main()
