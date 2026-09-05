#!/usr/bin/env python3
"""Validate the unpublished roadmap migration and explicit prerequisite edges."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HEADING = re.compile(r"(?m)^#{2,3} (v\d+\.\d+\.\d+(?:-rc\.\d+)?) - (.+)$")
MINOR = re.compile(r"v0\.(\d+)\.0")


def blocks(plan: str) -> dict[str, tuple[str, str]]:
    matches = list(HEADING.finditer(plan))
    result = {}
    for index, match in enumerate(matches):
        version, title = match.groups()
        if version in result:
            raise ValueError(f"duplicate milestone: {version}")
        end = matches[index + 1].start() if index + 1 < len(matches) else len(plan)
        body = plan[match.end():end]
        body = re.split(r"(?m)^## ", body, maxsplit=1)[0]
        result[version] = (title, body)
    return result


def validate(plan: str, manifest: dict) -> int:
    if not re.fullmatch(r"[0-9a-f]{40}", manifest.get("source_commit", "")):
        raise ValueError("map needs an exact source commit")
    if manifest.get("published_through") != "v0.55.0":
        raise ValueError("migration must not rewrite the published v0.55.0 baseline")
    entries = manifest.get("mapping", [])
    if not entries or len(entries) != manifest.get("retained_contracts"):
        raise ValueError("retained contract count does not match map")
    milestones = blocks(plan)
    assigned, previous_sources = [], set()
    previous_source_key = (55, 0)
    for entry in entries:
        source = entry["previous"]
        parsed = re.fullmatch(r"v0\.(\d+)\.(\d+)", source)
        if parsed is None:
            raise ValueError("invalid previous version")
        source_key = tuple(map(int, parsed.groups()))
        if source in previous_sources or source_key <= previous_source_key:
            raise ValueError("previous versions must be unique and ordered")
        previous_sources.add(source)
        previous_source_key = source_key
        versions = entry["passes"]
        if not versions or any(MINOR.fullmatch(version) is None for version in versions):
            raise ValueError("implementation passes must be minor versions")
        for version in versions:
            if version not in milestones or version in assigned:
                raise ValueError(f"missing or duplicate mapped milestone: {version}")
            assigned.append(version)
        title = entry["title"] + (" Completion" if len(versions) > 1 else "")
        if milestones[versions[-1]][0] != title:
            raise ValueError(f"completion title differs from map: {versions[-1]}")
    if len(assigned) - len(entries) != manifest.get("added_passes"):
        raise ValueError("added pass count differs from map")
    if assigned != [f"v0.{n}.0" for n in range(56, 56 + len(assigned))]:
        raise ValueError("migration has holes or reordered passes")
    for version, (_, body) in milestones.items():
        match = MINOR.fullmatch(version)
        if match is None or int(match[1]) < 56:
            continue
        if not re.search(r"(?m)^Status: planned", body):
            continue
        scopes = re.findall(r"(?ms)^Scope:(.*?)(?=\n\nDeliverables:)", body)
        if len(scopes) != 1:
            raise ValueError(f"missing or duplicate Scope: {version}")
        dependencies = re.findall(r"Depends on (v0\.\d+\.\d+)\.", scopes[0])
        expected = f"v0.{int(match[1]) - 1}.0"
        if dependencies != [expected] or expected not in milestones:
            raise ValueError(f"missing, forward or incorrect predecessor: {version}")
    return len(assigned)


def main() -> int:
    plan = (ROOT / "docs/RELEASE_PLAN.md").read_text(encoding="utf-8")
    manifest = json.loads((ROOT / "docs/roadmap-version-map.json").read_text(encoding="utf-8"))
    try:
        count = validate(plan, manifest)
    except (ValueError, KeyError, TypeError) as error:
        print(f"roadmap map validation failed: {error}")
        return 1
    print(f"validated {count} mapped unpublished minor milestones and prerequisite edges")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
