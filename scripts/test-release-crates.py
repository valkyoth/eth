#!/usr/bin/env python3
"""Tests for the per-crate release policy helper."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path
from types import SimpleNamespace


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "release_crates.py"


def load_release_crates():
    spec = importlib.util.spec_from_file_location("release_crates", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load release_crates.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release_crates = load_release_crates()


def package(name: str, version: str, deps: tuple[str, ...] = ()) -> dict:
    return {
        "name": name,
        "version": version,
        "dependencies": [{"name": dep} for dep in deps],
    }


def base_plan() -> dict:
    return {
        "version": "0.4.0",
        "crates": {
            name: {
                "previous_version": "0.3.0",
                "version": "0.3.0",
                "change": "unchanged",
                "publish": False,
                "reason": "test",
            }
            for name in release_crates.PUBLISH_ORDER
        },
    }


def base_packages() -> dict[str, dict]:
    packages = {
        name: package(name, "0.3.0") for name in release_crates.PUBLISH_ORDER
    }
    packages["eth-valkyoth-primitives"]["dependencies"] = [
        {"name": "eth-valkyoth-codec"}
    ]
    packages["eth-valkyoth-protocol"]["dependencies"] = [
        {"name": "eth-valkyoth-primitives"}
    ]
    packages["eth"]["dependencies"] = [{"name": "eth-valkyoth-protocol"}]
    return packages


def assert_fails(expected: str, func, *args) -> None:
    try:
        func(*args)
    except RuntimeError as exc:
        if expected not in str(exc):
            raise AssertionError(f"expected {expected!r} in {exc!r}") from exc
        return
    raise AssertionError("expected failure")


def test_current_plan_accepts_unchanged_crates() -> None:
    release_crates.verify_publish_order(base_packages(), base_plan())


def test_facade_code_changes_must_use_milestone_version() -> None:
    plan = base_plan()
    plan["crates"]["eth"]["change"] = "code"
    plan["crates"]["eth"]["publish"] = True
    assert_fails(
        "version must be 0.4.0",
        release_crates.validate_plan_entry,
        "eth",
        plan["crates"]["eth"],
        "0.4.0",
    )


def test_support_crate_code_changes_use_next_independent_minor() -> None:
    entry = {
        "previous_version": "0.7.0",
        "version": "0.8.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("eth-valkyoth-verify", entry, "0.19.0")


def test_support_crate_code_changes_reject_release_counter_jump() -> None:
    entry = {
        "previous_version": "0.7.0",
        "version": "0.19.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "independent support-crate version must be 0.8.0",
        release_crates.validate_plan_entry,
        "eth-valkyoth-verify",
        entry,
        "0.19.0",
    )


def test_support_crate_bugfixes_use_next_patch() -> None:
    entry = {
        "previous_version": "0.7.2",
        "version": "0.7.3",
        "change": "bugfix",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("eth-valkyoth-verify", entry, "0.19.0")


def test_support_crate_bugfixes_reject_minor_bumps() -> None:
    entry = {
        "previous_version": "0.7.2",
        "version": "0.8.0",
        "change": "bugfix",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "support-crate version must be 0.7.3",
        release_crates.validate_plan_entry,
        "eth-valkyoth-verify",
        entry,
        "0.19.0",
    )


def test_dependency_only_changes_must_patch_bump() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "dependency",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "dependency-only bumps",
        release_crates.validate_plan_entry,
        "eth",
        entry,
        "0.4.0",
    )


def test_unchanged_crates_are_not_published() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.3.0",
        "change": "unchanged",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "unchanged but publish is true",
        release_crates.validate_plan_entry,
        "eth",
        entry,
        "0.4.0",
    )


def test_metadata_changes_use_milestone_version() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "metadata",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("eth", entry, "0.4.0")


def test_metadata_changes_must_be_published() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "metadata",
        "publish": False,
        "reason": "test",
    }
    assert_fails(
        "metadata changes but publish is false",
        release_crates.validate_plan_entry,
        "eth",
        entry,
        "0.4.0",
    )


def test_publish_plan_skips_unchanged_crates() -> None:
    plan = base_plan()
    plan["crates"]["eth-valkyoth-codec"] = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert release_crates.publish_plan(plan) == ("eth-valkyoth-codec",)


def test_post_tag_preflight_passes_guarded_publish_context() -> None:
    calls = []
    original_run = release_crates.run
    release_crates.run = lambda command, **kwargs: calls.append((command, kwargs))
    try:
        args = SimpleNamespace(
            version="0.52.3",
            skip_checks=False,
            dry_run=False,
        )
        release_crates.run_preflight(args, release_tag_at_head=True)
    finally:
        release_crates.run = original_run

    assert calls[0] == (
        ["scripts/release_0_52_3_gate.sh"],
        {
            "dry_run": False,
            "extra_env": {"ETH_RELEASE_PUBLISH_TAG": "v0.52.3"},
        },
    )
    assert calls[1][1] == {"dry_run": False}
    assert calls[2][1] == {"dry_run": False}


def test_pre_tag_preflight_does_not_set_publish_context() -> None:
    calls = []
    original_run = release_crates.run
    release_crates.run = lambda command, **kwargs: calls.append((command, kwargs))
    try:
        args = SimpleNamespace(
            version="0.52.3",
            skip_checks=False,
            dry_run=False,
        )
        release_crates.run_preflight(args, release_tag_at_head=False)
    finally:
        release_crates.run = original_run

    assert calls[0][1] == {"dry_run": False, "extra_env": None}


def run_tests() -> None:
    tests = (
        test_current_plan_accepts_unchanged_crates,
        test_facade_code_changes_must_use_milestone_version,
        test_support_crate_code_changes_use_next_independent_minor,
        test_support_crate_code_changes_reject_release_counter_jump,
        test_support_crate_bugfixes_use_next_patch,
        test_support_crate_bugfixes_reject_minor_bumps,
        test_dependency_only_changes_must_patch_bump,
        test_unchanged_crates_are_not_published,
        test_metadata_changes_use_milestone_version,
        test_metadata_changes_must_be_published,
        test_publish_plan_skips_unchanged_crates,
        test_post_tag_preflight_passes_guarded_publish_context,
        test_pre_tag_preflight_does_not_set_publish_context,
    )
    for test in tests:
        test()


if __name__ == "__main__":
    run_tests()
