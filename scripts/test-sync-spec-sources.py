#!/usr/bin/env python3
"""Regression tests for sync_spec_sources.py trust-boundary checks."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "sync_spec_sources.py"
MODULE_SPEC = importlib.util.spec_from_file_location("sync_spec_sources", SCRIPT)
assert MODULE_SPEC is not None
sync_spec_sources = importlib.util.module_from_spec(MODULE_SPEC)
assert MODULE_SPEC.loader is not None
sys.modules[MODULE_SPEC.name] = sync_spec_sources
MODULE_SPEC.loader.exec_module(sync_spec_sources)


def run(args: list[str], cwd: Path) -> str:
    result = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout.strip()


def write_spec_lock(path: Path, repo: str) -> None:
    path.write_text(
        f"""
[ethereum]
local_reference_store_env = "ETH_REFERENCE_STORE"
local_reference_store_default = "../reference-store" # inline comments must parse
execution_specs_repo = "{repo}"
execution_specs_rev = "0123456789abcdef0123456789abcdef01234567"
execution_tests_repo = "https://github.com/ethereum/tests"
execution_tests_rev = "1123456789abcdef0123456789abcdef01234567"
eips_repo = "https://github.com/ethereum/EIPs"
eips_rev = "2123456789abcdef0123456789abcdef01234567"
execution_apis_repo = "https://github.com/ethereum/execution-apis"
execution_apis_rev = "3123456789abcdef0123456789abcdef01234567"
consensus_specs_repo = "https://github.com/ethereum/consensus-specs"
consensus_specs_rev = "4123456789abcdef0123456789abcdef01234567"
""".strip(),
        encoding="utf-8",
    )


def expect_error(message: str, func) -> None:
    try:
        func()
    except Exception as error:
        if message in str(error):
            return
        raise AssertionError(f"expected {message!r}, got {error!r}") from error
    raise AssertionError(f"expected error containing {message!r}")


def test_default_store_fallback_and_repo_validation(tmp: Path) -> None:
    spec_lock = tmp / "spec-lock.toml"
    repo_root = tmp / "repo"
    repo_root.mkdir(exist_ok=True)

    sync_spec_sources.SPEC_LOCK = spec_lock
    sync_spec_sources.ROOT = repo_root

    os.environ.pop("ETH_REFERENCE_STORE", None)
    write_spec_lock(spec_lock, "https://github.com/ethereum/execution-specs")
    store, sources = sync_spec_sources.parse_spec_lock()
    assert store == (repo_root / "../reference-store").resolve()
    assert len(sources) == 5

    os.environ["ETH_REFERENCE_STORE"] = str(tmp / "override")
    store, _ = sync_spec_sources.parse_spec_lock()
    assert store == (tmp / "override").resolve()
    os.environ.pop("ETH_REFERENCE_STORE", None)

    write_spec_lock(spec_lock, "ext::sh -c 'id > /tmp/pwned'")
    expect_error("official Ethereum HTTPS repository", sync_spec_sources.parse_spec_lock)


def test_selected_sources_limits_sync_scope(tmp: Path) -> None:
    spec_lock = tmp / "spec-lock.toml"
    repo_root = tmp / "repo"
    repo_root.mkdir(exist_ok=True)

    sync_spec_sources.SPEC_LOCK = spec_lock
    sync_spec_sources.ROOT = repo_root
    write_spec_lock(spec_lock, "https://github.com/ethereum/execution-specs")

    _, sources = sync_spec_sources.parse_spec_lock()
    selected = sync_spec_sources.selected_sources(sources, "execution_tests")
    assert [source.name for source in selected] == ["execution_tests"]


def test_verify_source_rejects_dirty_checkout(tmp: Path) -> None:
    checkout = tmp / "reference" / "execution_specs"
    checkout.mkdir(parents=True)
    run(["git", "init", "-q"], cwd=checkout)
    run(["git", "config", "user.email", "sync-spec@example.invalid"], cwd=checkout)
    run(["git", "config", "user.name", "Sync Spec Test"], cwd=checkout)
    run(
        ["git", "remote", "add", "origin", "https://github.com/ethereum/execution-specs"],
        cwd=checkout,
    )
    (checkout / "README.md").write_text("fixture\n", encoding="utf-8")
    run(["git", "add", "README.md"], cwd=checkout)
    run(["git", "commit", "-q", "-m", "fixture"], cwd=checkout)
    rev = run(["git", "rev-parse", "HEAD"], cwd=checkout)

    source = sync_spec_sources.Source(
        name="execution_specs",
        repo="https://github.com/ethereum/execution-specs",
        rev=rev,
    )
    sync_spec_sources.verify_source(tmp / "reference", source)

    (checkout / "README.md").write_text("tampered\n", encoding="utf-8")
    expect_error("uncommitted modifications", lambda: sync_spec_sources.verify_source(tmp / "reference", source))


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        tmp = Path(directory)
        test_default_store_fallback_and_repo_validation(tmp)
        test_selected_sources_limits_sync_scope(tmp)
        test_verify_source_rejects_dirty_checkout(tmp)
    print("sync_spec_sources.py tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
