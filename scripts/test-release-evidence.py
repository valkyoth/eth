#!/usr/bin/env python3
"""Signed-history regressions for F1/F2, including the actual readiness script."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import release_train
import release_crates
from release_evidence import authenticated_tag, validate_report, validate_train_reports
from release_test_support import Repository


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Repository(Path(self.temp.name) / "repo")
        self.repo.report("0.55.0")

    def plan(self, version="0.60.0", baseline="0.55.0", milestones=None):
        return release_train.validate_release_context(dict(
            version=version, milestone=version,
            stage="public" if version in ("0.60.0", "0.65.0") else "internal",
            baseline=baseline, review_baseline="0.59.0",
            cumulative_milestones=milestones or [f"0.{n}.0" for n in range(56, 61)]))

    def train(self):
        previous = "0.55.0"
        for minor in range(56, 60):
            version = f"0.{minor}.0"
            self.repo.report(version, previous)
            previous = version

    def test_real_signed_public_readiness_and_patch_chain(self):
        self.train()
        self.repo.report("0.59.1", "0.59.0")
        result = self.repo.readiness(review="0.59.1", milestones=[
            "0.56.0", "0.57.0", "0.58.0", "0.59.0", "0.59.1", "0.60.0"])
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_real_signed_internal_readiness(self):
        result = self.repo.readiness("0.56.0", review="0.55.0", milestones=["0.56.0"])
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_publisher_uses_same_authorized_signer_policy(self):
        self.repo.report("0.56.0")
        with patch.object(release_crates, "ROOT", self.repo.root):
            self.assertTrue(release_crates.check_release_tag("0.56.0", require_tag=True))
            self.repo.git("tag", "-d", "v0.56.0")
            other = Repository(Path(self.temp.name) / "outsider")
            self.repo.sign("0.56.0", other.key)
            with self.assertRaises(SystemExit):
                release_crates.check_release_tag("0.56.0", require_tag=True)

    def test_shortened_baseline_rejected_by_actual_readiness(self):
        self.train()
        result = self.repo.readiness(baseline="0.59.0", milestones=["0.60.0"])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("preceding public checkpoint", result.stderr)

    def test_wrong_and_missing_checkpoint_baselines(self):
        self.train()
        with patch.object(release_train, "ROOT", self.repo.root):
            for baseline in ("0.56.0", "0.59.0", "0.58.1"):
                with self.subTest(baseline=baseline), self.assertRaisesRegex(RuntimeError, "preceding public"):
                    release_train.validate_repository_train(self.plan(baseline=baseline, milestones=["0.60.0"]))
            with self.assertRaisesRegex(RuntimeError, "preceding public"):
                release_train.validate_repository_train(self.plan("0.65.0", milestones=["0.65.0"]))
            with self.assertRaisesRegex(RuntimeError, "scheduled public checkpoint"):
                release_train.validate_repository_train(self.plan("0.65.0", "0.60.0", ["0.65.0"]))
            with self.assertRaisesRegex(RuntimeError, "must precede"):
                self.plan(baseline="0.65.0")

    def test_missing_minor_and_omitted_patch_are_rejected(self):
        with patch.object(release_train, "ROOT", self.repo.root):
            with self.assertRaisesRegex(RuntimeError, "intermediate minor"):
                release_train.validate_repository_train(self.plan())
            self.train()
            self.repo.report("0.59.1", "0.59.0")
            with self.assertRaisesRegex(RuntimeError, "must list every tag"):
                release_train.validate_repository_train(self.plan())

    def test_invalid_reports_rejected_even_when_signed(self):
        reviewed = self.repo.git("rev-parse", "HEAD")
        transformations = (
            lambda b: b.replace("Status: PASS", "Status: FAIL"),
            lambda b: "",
            lambda b: b + "Status: PASS\n",
            lambda b: b.replace("Tester: Release fixture", "Tester: "),
            lambda b: b.replace("2026-09-05", "2026-02-31"),
            lambda b: b.replace("Baseline: v0.55.0", "Baseline: v0.54.0"),
            lambda b: b.replace("Range-End: v0.56.0", "Range-End: v0.57.0"),
            lambda b: b.replace("INCREMENTAL", "CUMULATIVE"),
            lambda b: b.replace(b.split("Reviewed-Commit: ")[1].splitlines()[0], "0" * 40),
            lambda b: b.replace(b.split("Reviewed-Commit: ")[1].splitlines()[0], reviewed),
        )
        for index, transform in enumerate(transformations):
            with self.subTest(index=index):
                commit = self.repo.report("0.56.0", transform=transform)
                authenticated_tag(self.repo.root, "0.56.0")
                with self.assertRaises(RuntimeError):
                    validate_report(self.repo.root, commit, "0.56.0", "INCREMENTAL", "0.55.0")
                self.repo.git("tag", "-d", "v0.56.0")

    def test_unsigned_untrusted_and_corrupt_signatures(self):
        self.repo.report("0.56.0", signed=False)
        self.repo.git("tag", "v0.56.0")
        with self.assertRaisesRegex(RuntimeError, "signed and annotated"):
            authenticated_tag(self.repo.root, "0.56.0")
        self.repo.git("tag", "-d", "v0.56.0")
        other = Repository(Path(self.temp.name) / "other")
        self.repo.sign("0.56.0", other.key)
        # A dirty policy file and a permissive local Git keyring cannot admit
        # a key that was not in the candidate's committed release policy.
        self.repo.write("security/release-allowed-signers", 'other ' + other.key.with_suffix(".pub").read_text())
        self.repo.git("config", "gpg.ssh.allowedSignersFile", str(self.repo.root / "security/release-allowed-signers"))
        with self.assertRaises(RuntimeError):
            authenticated_tag(self.repo.root, "0.56.0")
        self.repo.git("tag", "-d", "v0.56.0")
        self.repo.sign("0.56.0")
        raw = self.repo.git("cat-file", "tag", "v0.56.0").replace("release v", "tampered v")
        import subprocess
        oid = subprocess.check_output(["git", "hash-object", "-t", "tag", "-w", "--stdin"],
                                      cwd=self.repo.root, input=raw, text=True).strip()
        self.repo.git("update-ref", "refs/tags/v0.56.0", oid)
        with self.assertRaises(RuntimeError):
            authenticated_tag(self.repo.root, "0.56.0")

    def test_unrelated_ancestry_and_merge_or_mixed_reports(self):
        anchor = self.repo.git("rev-parse", "HEAD")
        self.repo.report("0.56.0")
        commit = self.repo.git("rev-parse", "HEAD")
        self.repo.git("checkout", "--detach", anchor)
        with self.assertRaises(RuntimeError):
            authenticated_tag(self.repo.root, "0.56.0")
        self.repo.git("checkout", "--detach", commit)
        tree = self.repo.git("rev-parse", "HEAD^{tree}")
        parent = self.repo.git("rev-parse", "HEAD^")
        merge = self.repo.git("commit-tree", tree, "-p", parent, "-p", anchor, "-m", "merge report")
        with self.assertRaisesRegex(RuntimeError, "linear child"):
            validate_report(self.repo.root, merge, "0.56.0")
        self.repo.write("extra.rs", "// unexpected implementation change\n")
        self.repo.git("add", ".")
        self.repo.git("commit", "--amend", "--no-edit", "-q")
        with self.assertRaisesRegex(RuntimeError, "also changes implementation"):
            validate_report(self.repo.root, "HEAD", "0.56.0")

    def test_internal_readiness_rejects_bad_prior_report(self):
        self.repo.report("0.56.0", transform=lambda body: body.replace("PASS", "FAIL"))
        result = self.repo.readiness("0.57.0", review="0.56.0", milestones=["0.56.0", "0.57.0"])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Status must be PASS", result.stderr)


if __name__ == "__main__":
    unittest.main()
