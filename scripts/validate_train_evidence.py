#!/usr/bin/env python3
"""Check historical and current release evidence at manual readiness admission."""

import os
import sys
import tomllib
from pathlib import Path

from release_train import validate_release_context, validate_repository_train
from release_evidence import authenticated_candidate, git, validate_report, validate_train_reports

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    try:
        with (ROOT / "release-crates.toml").open("rb") as handle:
            plan = validate_release_context(tomllib.load(handle)["release"])
        if len(sys.argv) != 2 or plan["version"] != sys.argv[1]:
            raise RuntimeError("candidate version differs from release metadata")
        publish_tag = os.environ.get("ETH_RELEASE_PUBLISH_TAG", "")
        if publish_tag and publish_tag != f"v{plan['version']}":
            raise RuntimeError("publish tag differs from release metadata")
        candidate = (authenticated_candidate(ROOT, plan["version"]) if publish_tag
                     else git(ROOT, "rev-parse", "--verify", "HEAD^{commit}"))
        validate_repository_train(plan)
        if not plan["anchor"]:
            validate_train_reports(ROOT, plan)
            assessment = "CUMULATIVE" if plan["stage"] == "public" else "INCREMENTAL"
            baseline = plan["baseline"] if plan["stage"] == "public" else plan["review_baseline"]
            validate_report(ROOT, candidate, plan["version"], assessment, baseline)
        else:
            validate_report(ROOT, candidate, plan["version"])
        if git(ROOT, "rev-parse", "--verify", "HEAD^{commit}") != candidate:
            raise RuntimeError("candidate HEAD moved during evidence validation")
    except (RuntimeError, ValueError, KeyError, OSError) as error:
        print(f"release evidence invalid: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
