#!/usr/bin/env python3
"""Reproduce the admitted CC0 EIP-2537 G1 addition fixture without executing upstream code."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SHA256 = "224b9b22ddf72efb0cd9c8ffb6cd742d68af7b062e592e8b8a5378a22d6f9d02"
OUTPUT = ROOT / "crates/eth-valkyoth-evm-core/tests/fixtures/bls12_g1_add.txt"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="pinned EIPs/assets/eip-2537/add_G1_bls.json")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    raw = args.source.read_bytes()
    if hashlib.sha256(raw).hexdigest() != SOURCE_SHA256:
        raise ValueError("EIP-2537 source hash differs from admitted revision")
    vectors = json.loads(raw)
    if len(vectors) != 9:
        raise ValueError("unexpected vector count")
    lines = ["# CC0 EIP-2537 add_G1_bls.json; EIPs 582684e2d7d372c09f45777be8ea603e485e9e9d",
             "# name|input|expected (all nine official vectors; no gas-execution claim)"]
    for vector in vectors:
        wire = bytes.fromhex(vector["Input"])
        expected = bytes.fromhex(vector["Expected"])
        if len(wire) != 256 or len(expected) != 128:
            raise ValueError("unexpected frame length")
        lines.append("|".join([vector["Name"], wire.hex(), expected.hex()]))
    result = "\n".join(lines) + "\n"
    if args.check:
        if OUTPUT.read_text() != result:
            raise ValueError("committed fixture differs from admitted source")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(result)
    print("verified nine pinned EIP-2537 G1 addition vectors")


if __name__ == "__main__":
    main()
