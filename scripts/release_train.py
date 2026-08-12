#!/usr/bin/env python3
"""Validate tagged milestones and five-minor crates.io checkpoints."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STAGES = ("internal", "public")
CADENCE_BASELINE = (0, 55, 0)


def parse_version(version: str) -> tuple[int, int, int]:
    parts = version.split(".")
    if len(parts) != 3:
        raise RuntimeError(f"version must be MAJOR.MINOR.PATCH: {version}")
    try:
        parsed = tuple(int(part) for part in parts)
    except ValueError as error:
        raise RuntimeError(f"version must be numeric: {version}") from error
    return parsed  # type: ignore[return-value]


def is_public_checkpoint(version: str) -> bool:
    major, minor, patch = parse_version(version)
    return major == 0 and patch == 0 and minor >= 55 and minor % 5 == 0


def next_public_checkpoint(version: str) -> str:
    major, minor, _patch = parse_version(version)
    if major != 0:
        raise RuntimeError("checkpoint calculation is only defined before v1.0.0")
    next_minor = ((minor // 5) + 1) * 5
    return "1.0.0" if next_minor >= 1000 else f"0.{next_minor}.0"


def publication_allowed(plan: dict) -> bool:
    return plan["stage"] == "public"


def validate_release_context(release: dict) -> dict:
    required = (
        "version",
        "milestone",
        "baseline",
        "review_baseline",
        "stage",
    )
    values = tuple(release.get(field) for field in required)
    milestones = release.get("cumulative_milestones")
    if not all(isinstance(value, str) for value in values) or not isinstance(
        milestones, list
    ):
        raise RuntimeError("release train metadata is incomplete")
    if not all(isinstance(value, str) for value in milestones):
        raise RuntimeError("cumulative_milestones must contain versions")

    version, milestone, baseline, review_baseline, stage = values
    if stage not in STAGES:
        raise RuntimeError(f"release stage must be one of {STAGES}")
    if milestone != version:
        raise RuntimeError("release milestone must match release version")

    parsed = parse_version(version)
    public_baseline = parse_version(baseline)
    review = parse_version(review_baseline)
    if min(parsed, public_baseline, review) < CADENCE_BASELINE:
        raise RuntimeError("five-minor release trains begin at v0.55.0")
    anchor = parsed == CADENCE_BASELINE and public_baseline == parsed
    if public_baseline > parsed or (public_baseline == parsed and not anchor):
        raise RuntimeError("public baseline must precede the milestone")
    if review > parsed or (review == parsed and not anchor):
        raise RuntimeError("review baseline must precede the milestone")

    parsed_milestones = tuple(parse_version(value) for value in milestones)
    if len(set(parsed_milestones)) != len(parsed_milestones):
        raise RuntimeError("cumulative_milestones contains duplicates")
    if parsed_milestones != tuple(sorted(parsed_milestones)):
        raise RuntimeError("cumulative_milestones must be in version order")
    if anchor:
        if stage != "public" or milestones:
            raise RuntimeError("v0.55.0 must remain the public cadence anchor")
    else:
        if not parsed_milestones or parsed_milestones[-1] != parsed:
            raise RuntimeError("cumulative_milestones must end at the milestone")
        if any(item <= public_baseline or item > parsed for item in parsed_milestones):
            raise RuntimeError("cumulative milestones must follow the public baseline")

    scheduled = is_public_checkpoint(version) or parsed[0] >= 1
    if (stage == "public") != scheduled:
        raise RuntimeError("release stage conflicts with five-minor cadence")

    return {
        "version": version,
        "milestone": milestone,
        "baseline": baseline,
        "review_baseline": review_baseline,
        "cumulative_milestones": tuple(milestones),
        "stage": stage,
        "anchor": anchor,
    }


def semantic_tags_before(version: str) -> tuple[str, ...]:
    candidate = parse_version(version)
    raw = subprocess.check_output(
        ["git", "tag", "--list", "v*.*.*"], cwd=ROOT, text=True
    )
    parsed: list[tuple[int, int, int]] = []
    for tag in raw.splitlines():
        try:
            tagged = parse_version(tag.removeprefix("v"))
        except RuntimeError:
            continue
        if tagged < candidate:
            parsed.append(tagged)
    return tuple(".".join(str(part) for part in item) for item in sorted(parsed))


def validate_repository_train(plan: dict) -> None:
    if plan["anchor"]:
        return
    baseline = parse_version(plan["baseline"])
    preceding = semantic_tags_before(plan["version"])
    expected = tuple(
        version for version in preceding if parse_version(version) > baseline
    ) + (plan["version"],)
    if plan["cumulative_milestones"] != expected:
        raise RuntimeError(
            "cumulative_milestones must list every tag after "
            f"v{plan['baseline']} through {plan['version']}: expected {expected}"
        )
    expected_review = preceding[-1] if preceding else plan["baseline"]
    if plan["review_baseline"] != expected_review:
        raise RuntimeError(
            "review_baseline must be the immediately preceding release tag: "
            f"expected {expected_review}, actual {plan['review_baseline']}"
        )


def validate_facade_previous_version(plan: dict) -> None:
    if plan["anchor"]:
        return
    if plan["stage"] == "public":
        expected = plan["baseline"]
    else:
        tags = semantic_tags_before(plan["version"])
        expected = tags[-1] if tags else plan["baseline"]
    actual = plan["crates"]["eth"]["previous_version"]
    if actual != expected:
        raise RuntimeError(
            "eth previous_version does not match the release train: "
            f"expected {expected}, actual {actual}"
        )


def changed_packages(packages: dict[str, dict], baseline: str) -> set[str]:
    tag = f"v{baseline}"
    if subprocess.run(
        ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode != 0:
        raise RuntimeError(f"release baseline tag is missing: {tag}")
    changed: set[str] = set()
    for name, package in packages.items():
        relative = Path(package["manifest_path"]).resolve().parent.relative_to(ROOT)
        tracked = subprocess.check_output(
            ["git", "diff", "--name-only", tag, "--", str(relative)],
            cwd=ROOT,
            text=True,
        ).strip()
        untracked = subprocess.check_output(
            ["git", "ls-files", "--others", "--exclude-standard", "--", str(relative)],
            cwd=ROOT,
            text=True,
        ).strip()
        if tracked or untracked:
            changed.add(name)
    return changed


def validate_dependency_closure(packages: dict[str, dict], plan: dict) -> None:
    changed_versions = {
        name
        for name, entry in plan["crates"].items()
        if entry["version"] != entry["previous_version"]
    }
    for name, package in packages.items():
        if plan["crates"][name]["change"] != "unchanged":
            continue
        dependencies = sorted(
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency["name"] in changed_versions
        )
        if dependencies:
            raise RuntimeError(
                f"{name} depends on changed internal packages but is marked "
                f"unchanged: {tuple(dependencies)}"
            )


def validate_cumulative_package_changes(packages: dict[str, dict], plan: dict) -> None:
    if plan["stage"] != "public" or plan["anchor"]:
        return
    for name in changed_packages(packages, plan["baseline"]):
        if plan["crates"][name]["change"] == "unchanged":
            raise RuntimeError(
                f"{name} changed after v{plan['baseline']} but is marked unchanged"
            )
    validate_dependency_closure(packages, plan)
