"""Small interactive helpers for the crates.io publisher."""

from __future__ import annotations

import sys
import time


def confirm_no_verify(args: object) -> int:
    if not getattr(args, "no_verify") or getattr(args, "dry_run"):
        return 0
    print(
        "\nWARNING: --no-verify bypasses cargo package verification.\n"
        "Use it only with a documented release incident or crates.io issue.\n"
        "Type 'no-verify confirmed' to continue:",
        file=sys.stderr,
    )
    if input().strip() != "no-verify confirmed":
        print("Aborted.", file=sys.stderr)
        return 1
    return 0


def selected_steps(start_at: str, steps: tuple[str, ...]) -> tuple[str, ...]:
    if not steps:
        return ()
    try:
        index = steps.index(start_at)
    except ValueError as error:
        raise RuntimeError(f"unknown package for --start-at: {start_at}") from error
    return steps[index:]


def wait_for_index(package: str, version: str, *, dry_run: bool) -> None:
    print()
    print(f"Published {package} {version}.")
    print(f"Wait until crates.io shows: https://crates.io/crates/{package}/{version}")
    print("Then press Enter to continue with dependent crates.")
    if dry_run:
        print("[dry-run] skipping wait")
        return
    input()
    time.sleep(5)
