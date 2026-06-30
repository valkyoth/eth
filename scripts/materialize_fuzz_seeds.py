#!/usr/bin/env python3
"""Validate or materialize committed fuzz seed corpus files."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SEED_ROOT = ROOT / "fuzz" / "seed-corpus"
CORPUS_ROOT = ROOT / "fuzz" / "corpus"
CARGO_TOML = ROOT / "fuzz" / "Cargo.toml"
HEX_RE = re.compile(r"^[0-9a-fA-F]*$")


def fuzz_targets() -> set[str]:
    names: set[str] = set()
    for line in CARGO_TOML.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("name = "):
            names.add(stripped.split('"', 2)[1])
    return names


def seed_files() -> list[Path]:
    return sorted(
        path
        for path in SEED_ROOT.glob("*/*.hex")
        if path.is_file() and path.parent.name != "README.md"
    )


def decode_seed(path: Path) -> bytes:
    text = "".join(
        line.partition("#")[0].strip()
        for line in path.read_text(encoding="utf-8").splitlines()
    )
    if len(text) % 2 != 0 or not HEX_RE.fullmatch(text):
        raise ValueError(f"{path} is not even-length hexadecimal")
    return bytes.fromhex(text)


def check() -> list[tuple[str, Path, bytes]]:
    targets = fuzz_targets()
    decoded: list[tuple[str, Path, bytes]] = []
    for path in seed_files():
        target = path.parent.name
        if target not in targets:
            raise ValueError(f"{path} targets unknown fuzz binary {target!r}")
        decoded.append((target, path, decode_seed(path)))
    if not decoded:
        raise ValueError("no fuzz seed files found")
    return decoded


def materialize(decoded: list[tuple[str, Path, bytes]]) -> None:
    for target, path, payload in decoded:
        digest = hashlib.sha256(payload).hexdigest()
        output = CORPUS_ROOT / target / f"{path.stem}-{digest[:16]}"
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(payload)
        print(output.relative_to(ROOT))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate committed seed files without writing fuzz/corpus",
    )
    args = parser.parse_args()

    decoded = check()
    if args.check:
        print(f"validated {len(decoded)} fuzz seed files")
        return 0
    materialize(decoded)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
