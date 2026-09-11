"""Disposable, genuinely SSH-signed repositories for release-control tests."""

import subprocess
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent


class Repository:
    def __init__(self, root: Path):
        self.root = root
        root.mkdir(parents=True)
        self.git("init", "-q")
        for key, value in (("user.name", "Release Test"),
                           ("user.email", "release@example.invalid"),
                           ("commit.gpgsign", "false"), ("tag.gpgsign", "false")):
            self.git("config", key, value)
        self.key = root.parent / f"{root.name}-key"
        subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(self.key)], check=True)
        self.write("security/release-allowed-signers",
                   'eth-release namespaces="git" ' + self.key.with_suffix(".pub").read_text())
        self.write("README.md", "fixture\n")
        self.commit("initial")

    def git(self, *args: str) -> str:
        return subprocess.check_output(["git", *args], cwd=self.root,
                                       text=True, stderr=subprocess.PIPE).strip()

    def write(self, name: str, body: str):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)

    def commit(self, message="implementation") -> str:
        self.git("add", ".")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")

    def sign(self, version: str, key: Path | None = None):
        self.git("-c", "gpg.format=ssh", "-c", f"user.signingkey={key or self.key}",
                 "tag", "-s", "-m", f"release v{version}", f"v{version}")

    def report(self, version: str, baseline="0.55.0", transform=None, signed=True):
        reviewed = self.commit()
        assessment = "INCREMENTAL"
        body = (f"Status: PASS\nReviewed-Commit: {reviewed}\nTester: Release fixture\n"
                f"Scope: Local test\nDate: 2026-09-05\nAssessment: {assessment}\n"
                f"Baseline: v{baseline}\nRange-End: v{version}\n")
        if transform:
            body = transform(body)
        self.write(f"security/pentest/v{version}.md", body)
        commit = self.commit("report")
        if signed:
            self.sign(version)
        return commit

    def readiness(self, version="0.60.0", baseline="0.55.0", review="0.59.0",
                  milestones=None):
        import json
        if milestones is None:
            milestones = [f"0.{minor}.0" for minor in range(56, 61)]
        for name in ("release_evidence.py", "release_train.py", "release_dependencies.py",
                     "validate_train_evidence.py", "validate-release-readiness.sh"):
            self.write(f"scripts/{name}", (SCRIPTS / name).read_text())
        self.write("scripts/generate-sbom.sh", "#!/bin/sh\nexit 0\n")
        (self.root / "scripts/generate-sbom.sh").chmod(0o755)
        stage = "public" if int(version.split(".")[1]) % 5 == 0 and version.endswith(".0") else "internal"
        self.write("release-crates.toml", "[release]\n" + "\n".join(
            f"{key} = {json.dumps(value)}" for key, value in dict(
                version=version, milestone=version, stage=stage, baseline=baseline,
                review_baseline=review, cumulative_milestones=milestones).items()))
        self.write("sbom/eth.spdx.json", "{}")
        from release_train import next_public_checkpoint
        publication = "PENDING" if stage == "public" else f"DEFERRED TO v{next_public_checkpoint(version)}"
        self.write(f"release-notes/RELEASE_NOTES_{version}.md", f"Publication: {publication}\n")
        self.report(version, review, signed=False)
        return subprocess.run(["sh", "scripts/validate-release-readiness.sh", f"v{version}"],
                              cwd=self.root, capture_output=True, text=True)
