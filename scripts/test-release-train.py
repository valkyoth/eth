#!/usr/bin/env python3
"""Regression tests for tagged milestones and public checkpoints."""

from __future__ import annotations

import release_train


def release(
    version: str,
    *,
    stage: str,
    baseline: str = "0.55.0",
    review_baseline: str | None = None,
    milestones: tuple[str, ...] = (),
) -> dict:
    return {
        "version": version,
        "milestone": version,
        "baseline": baseline,
        "review_baseline": review_baseline or baseline,
        "cumulative_milestones": list(milestones),
        "policy": "independent",
        "stage": stage,
    }


def assert_fails(expected: str, function, *args) -> None:
    try:
        function(*args)
    except RuntimeError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected failure")


def test_v055_is_public_anchor() -> None:
    context = release_train.validate_release_context(
        release("0.55.0", stage="public", baseline="0.55.0")
    )
    assert context["anchor"]
    assert release_train.publication_allowed(context)


def test_intermediate_minor_and_patch_are_internal() -> None:
    for version, milestones in (
        ("0.56.0", ("0.56.0",)),
        ("0.56.1", ("0.56.0", "0.56.1")),
        ("0.59.3", ("0.56.0", "0.56.1", "0.59.3")),
    ):
        context = release_train.validate_release_context(
            release(version, stage="internal", milestones=milestones)
        )
        assert not release_train.publication_allowed(context)
        assert release_train.next_public_checkpoint(version) == "0.60.0"


def test_every_fifth_minor_is_public() -> None:
    milestones = ("0.56.0", "0.57.0", "0.58.0", "0.59.0", "0.60.0")
    context = release_train.validate_release_context(
        release(
            "0.60.0",
            stage="public",
            review_baseline="0.59.0",
            milestones=milestones,
        )
    )
    assert release_train.publication_allowed(context)


def test_internal_checkpoint_and_public_intermediate_are_rejected() -> None:
    checkpoint = ("0.56.0", "0.57.0", "0.58.0", "0.59.0", "0.60.0")
    assert_fails(
        "stage conflicts",
        release_train.validate_release_context,
        release("0.60.0", stage="internal", milestones=checkpoint),
    )
    assert_fails(
        "stage conflicts",
        release_train.validate_release_context,
        release("0.57.0", stage="public", milestones=("0.57.0",)),
    )


def test_patch_does_not_advance_checkpoint() -> None:
    assert release_train.next_public_checkpoint("0.57.9") == "0.60.0"
    assert release_train.next_public_checkpoint("0.60.1") == "0.65.0"


def test_repository_train_rejects_omitted_patch_tag() -> None:
    plan = release_train.validate_release_context(
        release(
            "0.60.0",
            stage="public",
            review_baseline="0.59.0",
            milestones=("0.56.0", "0.57.0", "0.59.0", "0.60.0"),
        )
    )
    original = release_train.semantic_tags_before
    release_train.semantic_tags_before = lambda _version: (
        "0.55.0",
        "0.56.0",
        "0.57.0",
        "0.57.1",
        "0.59.0",
    )
    try:
        assert_fails(
            "must list every tag",
            release_train.validate_repository_train,
            plan,
        )
    finally:
        release_train.semantic_tags_before = original


def test_public_checkpoint_rejects_lost_package_delta() -> None:
    plan = {
        "stage": "public",
        "anchor": False,
        "baseline": "0.55.0",
        "crates": {"eth-valkyoth-evm-core": {"change": "unchanged"}},
    }
    original = release_train.changed_packages
    release_train.changed_packages = lambda _packages, _baseline: {
        "eth-valkyoth-evm-core"
    }
    try:
        assert_fails(
            "changed after v0.55.0",
            release_train.validate_cumulative_package_changes,
            {},
            plan,
        )
    finally:
        release_train.changed_packages = original


def test_public_checkpoint_rejects_lost_dependency_delta() -> None:
    packages = {
        "eth-valkyoth-evm-core": {"dependencies": []},
        "eth-valkyoth-evm": {
            "dependencies": [{"name": "eth-valkyoth-evm-core"}]
        },
    }
    plan = {
        "crates": {
            "eth-valkyoth-evm-core": {
                "previous_version": "0.29.0",
                "version": "0.30.0",
                "change": "code",
            },
            "eth-valkyoth-evm": {
                "previous_version": "0.12.2",
                "version": "0.12.2",
                "change": "unchanged",
            },
        }
    }
    assert_fails(
        "depends on changed internal packages",
        release_train.validate_dependency_closure,
        packages,
        plan,
    )


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} release train tests passed")


if __name__ == "__main__":
    main()
