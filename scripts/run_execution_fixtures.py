#!/usr/bin/env python3
"""Run claimed Ethereum execution fixtures against this workspace."""

from __future__ import annotations

import argparse
import os
import subprocess
import tomllib
from pathlib import Path

from sync_spec_sources import git_stdout, validate_repo_url, validate_revision

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "conformance" / "execution-fixtures.toml"
SPEC_LOCK = ROOT / "spec-lock.toml"


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def spec_lock() -> dict:
    data = load_toml(SPEC_LOCK).get("ethereum")
    if not isinstance(data, dict):
        raise ValueError("spec-lock.toml is missing [ethereum]")
    return data


def manifest() -> dict:
    data = load_toml(MANIFEST)
    section = data.get("execution_fixtures")
    if not isinstance(section, dict):
        raise ValueError("execution fixture manifest is missing [execution_fixtures]")
    claimed = data.get("claimed")
    unsupported = data.get("unsupported")
    if not isinstance(claimed, list) or not claimed:
        raise ValueError("execution fixture manifest must claim at least one fixture group")
    if not isinstance(unsupported, list) or not unsupported:
        raise ValueError("execution fixture manifest must list unsupported fixture groups")
    return data


def default_store(lock: dict) -> Path:
    env_name = lock.get("local_reference_store_env", "")
    if env_name and not isinstance(env_name, str):
        raise ValueError("local_reference_store_env must be a string")
    override = os.environ.get(env_name) if env_name else None
    if override:
        return Path(override).resolve()
    default = lock.get("local_reference_store_default")
    if not isinstance(default, str) or not default:
        raise ValueError("spec-lock.toml is missing local_reference_store_default")
    return (ROOT / default).resolve()


def verify_execution_tests(checkout: Path, lock: dict) -> None:
    if not lock.get("execution_tests_repo") or not lock.get("execution_tests_rev"):
        raise ValueError("spec-lock.toml is missing execution_tests_repo or execution_tests_rev")
    expected_repo = validate_repo_url("execution_tests", lock.get("execution_tests_repo"))
    expected_rev = validate_revision("execution_tests", lock.get("execution_tests_rev"))
    if not checkout.exists():
        raise FileNotFoundError(f"{checkout} does not exist; run scripts/sync_spec_sources.py")
    origin = git_stdout(["git", "remote", "get-url", "origin"], checkout)
    if origin != expected_repo:
        raise ValueError(f"execution_tests origin is {origin}, expected {expected_repo}")
    actual = git_stdout(["git", "rev-parse", "HEAD"], checkout)
    if actual != expected_rev:
        raise ValueError(f"execution_tests is at {actual}, expected {expected_rev}")
    dirty = git_stdout(["git", "status", "--porcelain"], checkout)
    if dirty:
        raise ValueError("execution_tests checkout has uncommitted modifications")


def run_rlp_fixtures(execution_tests: Path) -> None:
    rlp_dir = execution_tests / "RLPTests"
    if not rlp_dir.exists():
        raise FileNotFoundError(f"{rlp_dir} does not exist")
    env = {
        **os.environ,
        "ETH_EXECUTION_TESTS_RLP_DIR": str(rlp_dir),
        "ETH_REQUIRE_EXECUTION_FIXTURES": "1",
    }
    subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "eth-valkyoth-codec",
            "--test",
            "execution_rlp_fixtures",
            "--features",
            "testing",
        ],
        cwd=ROOT,
        check=True,
        env=env,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate the fixture manifest without requiring local checkouts",
    )
    parser.add_argument(
        "--execution-tests",
        type=Path,
        help="path to a pinned ethereum/tests checkout; defaults to the reference store",
    )
    args = parser.parse_args()

    data = manifest()
    lock = spec_lock()
    if data["execution_fixtures"].get("revision") != lock.get("execution_tests_rev"):
        raise ValueError("execution fixture manifest revision differs from spec-lock.toml")
    if args.check:
        print(f"validated {len(data['claimed'])} claimed execution fixture groups")
        return 0

    execution_tests = args.execution_tests.resolve() if args.execution_tests else default_store(lock) / "execution_tests"
    verify_execution_tests(execution_tests, lock)
    run_rlp_fixtures(execution_tests)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
