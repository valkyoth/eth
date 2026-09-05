#!/usr/bin/env python3
"""Authenticate release tags and their exact, report-only review attestations."""

from __future__ import annotations

import datetime
import re
import subprocess
import tempfile
from pathlib import Path


def git(root: Path, *args: str) -> str:
    try:
        return subprocess.check_output(
            ["git", *args], cwd=root, text=True, stderr=subprocess.PIPE
        ).strip()
    except subprocess.CalledProcessError as error:
        raise RuntimeError(f"git {' '.join(args)} failed: {error.stderr.strip()}") from error


def authenticated_tag(root: Path, version: str, candidate: str = "HEAD") -> str:
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version):
        raise RuntimeError("noncanonical release version")
    ref = f"refs/tags/v{version}"
    oid = git(root, "rev-parse", "--verify", ref)
    candidate_commit = git(root, "rev-parse", f"{candidate}^{{commit}}")
    if git(root, "cat-file", "-t", oid) != "tag":
        raise RuntimeError(f"{ref}: release tag must be signed and annotated")
    policy_body = git(root, "show", f"{candidate_commit}:security/release-allowed-signers")
    tag_body = git(root, "cat-file", "tag", oid)
    header = tag_body.split("\n\n", 1)[0]
    if (re.findall(r"^tag (.+)$", header, re.MULTILINE) != [f"v{version}"]
            or re.findall(r"^type (.+)$", header, re.MULTILINE) != ["commit"]):
        raise RuntimeError(f"{ref}: signed tag name or target type differs")
    if "-----BEGIN SSH SIGNATURE-----" not in tag_body:
        raise RuntimeError(f"{ref}: release tag needs an authorized SSH signature")
    # Use committed trust policy, never a dirty file or the user's wider keyring.
    with tempfile.TemporaryDirectory(prefix="eth-release-signers-") as directory:
        policy = Path(directory) / "allowed-signers"
        policy.write_text(policy_body + "\n", encoding="utf-8")
        git(root, "-c", "gpg.format=ssh", "-c", "gpg.ssh.program=ssh-keygen",
            "-c", f"gpg.ssh.allowedSignersFile={policy}", "verify-tag", oid)
    commit = git(root, "rev-parse", f"{oid}^{{commit}}")
    git(root, "merge-base", "--is-ancestor", commit, candidate_commit)
    return commit


def field(body: str, name: str) -> str:
    values = re.findall(rf"^{re.escape(name)}:[ \t]*(.*)$", body, re.MULTILINE)
    if len(values) != 1 or not values[0].strip():
        raise RuntimeError(f"report needs exactly one nonempty {name}")
    return values[0].strip()


def validate_report(root: Path, commit: str, version: str,
                    assessment: str | None = None, baseline: str | None = None) -> None:
    path = f"security/pentest/v{version}.md"
    body = git(root, "show", f"{commit}:{path}")
    if field(body, "Status") != "PASS":
        raise RuntimeError(f"v{version}: report Status must be PASS")
    reviewed = field(body, "Reviewed-Commit")
    if not re.fullmatch(r"[0-9a-f]{40}", reviewed):
        raise RuntimeError(f"v{version}: invalid reviewed commit")
    field(body, "Tester")
    field(body, "Scope")
    date = field(body, "Date")
    if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", date):
        raise RuntimeError("invalid report Date")
    try:
        datetime.date.fromisoformat(date)
    except ValueError as error:
        raise RuntimeError("invalid report Date") from error
    if assessment is not None:
        if (field(body, "Assessment") != assessment
                or field(body, "Baseline") != f"v{baseline}"
                or field(body, "Range-End") != f"v{version}"):
            raise RuntimeError(f"v{version}: invalid milestone assessment")
    parents = git(root, "rev-list", "--parents", "-n", "1", commit).split()
    if len(parents) != 2 or parents[1] != reviewed:
        raise RuntimeError(f"v{version}: report must be a linear child of reviewed commit")
    if git(root, "diff", "--name-only", reviewed, commit).splitlines() != [path]:
        raise RuntimeError(f"v{version}: report commit also changes implementation")


def validate_train_reports(root: Path, plan: dict) -> None:
    # The 0.55 anchor predates train fields, but still needs its signed PASS
    # report and direct report-only lineage. No earlier history is rewritten.
    baseline = plan["baseline"]
    baseline_commit = authenticated_tag(root, baseline)
    if baseline == "0.55.0":
        validate_report(root, baseline_commit, baseline)
    else:
        minor = int(baseline.split(".")[1])
        validate_report(root, baseline_commit, baseline, "CUMULATIVE", f"0.{minor - 5}.0")
    previous = baseline
    previous_commit = baseline_commit
    for version in plan["cumulative_milestones"]:
        if version == plan["version"]:
            break
        commit = authenticated_tag(root, version)
        git(root, "merge-base", "--is-ancestor", previous_commit, commit)
        validate_report(root, commit, version, "INCREMENTAL", previous)
        previous, previous_commit = version, commit
