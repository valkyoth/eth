#!/usr/bin/env python3
"""Gate phase separation and explicit differential-run selection regressions."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import run_differential_tests as differential

SCRIPTS = Path(__file__).resolve().parent


class GateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "scripts").mkdir()
        (self.root / "bin").mkdir()
        self.log = self.root / "calls"
        self.gate = self.root / "gate.sh"
        self.gate.write_text((SCRIPTS / "release_0_56_0_gate.sh").read_text())
        names = ("validate-release-metadata.sh", "validate-release-readiness.sh",
                 "checks.sh", "check_latest_tools.sh", "check_latest_crates.py",
                 "check_ethereum_upstream.py", "materialize_fuzz_seeds.py",
                 "run_differential_tests.py")
        for name in names:
            self.stub("scripts/" + name)
        self.stub("bin/cargo")
        self.stub("bin/rustc", 'printf "rustc 1.98.1 (fixture)\\n"\n')

    def stub(self, name, tail=""):
        path = self.root / name
        path.write_text('#!/bin/sh\nset -eu\n'
                        'printf "%s\\n" "$(basename "$0") $*" >> "$GATE_TEST_LOG"\n'
                        'if [ "${GATE_TEST_FAIL:-}" = "$(basename "$0")" ]; then exit 17; fi\n'
                        + tail)
        path.chmod(0o755)

    def invoke(self, *args, fail=""):
        if self.log.exists():
            self.log.unlink()
        env = dict(os.environ, PATH=f"{self.root / 'bin'}:{os.environ['PATH']}",
                   GATE_TEST_LOG=str(self.log), GATE_TEST_FAIL=fail)
        result = subprocess.run(["sh", str(self.gate), *args], cwd=self.root,
                                env=env, text=True, capture_output=True)
        calls = self.log.read_text().splitlines() if self.log.exists() else []
        return result, calls

    def test_default_and_explicit_tag_only_check_candidate_metadata_and_report(self):
        for args in ((), ("--tag",)):
            with self.subTest(args=args):
                result, calls = self.invoke(*args)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(calls, ["validate-release-metadata.sh ",
                                         "validate-release-readiness.sh v0.56.0"])

    def test_tag_errors_are_not_ignored(self):
        for fail in ("validate-release-metadata.sh", "validate-release-readiness.sh"):
            with self.subTest(fail=fail):
                result, calls = self.invoke("--tag", fail=fail)
                self.assertEqual(result.returncode, 17)
                self.assertTrue(calls[-1].startswith(fail))

    def test_implementation_runs_tests_freshness_audits_fuzz_and_compatibility(self):
        result, calls = self.invoke("--implementation")
        self.assertEqual(result.returncode, 0, result.stderr)
        for expected in ("checks.sh ", "check_latest_tools.sh ", "check_latest_crates.py ",
                         "check_ethereum_upstream.py ", "cargo deny check", "cargo audit",
                         "run_differential_tests.py --in-process",
                         "cargo +1.90.0 check --workspace --all-features",
                         "cargo +1.98.0 check --workspace --all-features"):
            self.assertIn(expected, calls)
        self.assertTrue(any("fuzz run bls12381_field" in call for call in calls))
        self.assertFalse(any("readiness" in call or "metadata.sh" in call for call in calls))
        self.assertNotIn("run_differential_tests.py ", calls)

    def test_implementation_failure_stops_without_tag_admission(self):
        result, calls = self.invoke("--implementation", fail="checks.sh")
        self.assertEqual(result.returncode, 17)
        self.assertEqual(calls, ["rustc --version", "checks.sh "])

    def test_unknown_or_extra_arguments_do_not_run_checks(self):
        for args in (("--skip",), ("--tag", "--implementation")):
            with self.subTest(args=args):
                result, calls = self.invoke(*args)
                self.assertEqual(result.returncode, 2)
                self.assertEqual(calls, [])


class DifferentialSelectionTests(unittest.TestCase):
    def test_in_process_is_explicit_and_default_still_runs_clients(self):
        for args, expected in (([], [*differential.DIFFERENTIAL_TESTS, differential.CLIENT_DIFFERENTIAL]),
                               (["--in-process"], differential.DIFFERENTIAL_TESTS)):
            with self.subTest(args=args), patch("sys.argv", ["runner", *args]), patch.object(differential, "run") as run:
                self.assertEqual(differential.main(), 0)
                self.assertEqual([call.args[0] for call in run.call_args_list], expected)

    def test_check_validates_both_paths_without_execution(self):
        with patch("sys.argv", ["runner", "--check"]), patch.object(differential, "run") as run:
            self.assertEqual(differential.main(), 0)
            self.assertEqual([call.args[0] for call in run.call_args_list], [
                *[[*command, "--no-run"] for command in differential.DIFFERENTIAL_TESTS],
                [*differential.CLIENT_DIFFERENTIAL, "--check"]])

    def test_explicit_client_failure_is_not_converted_to_success(self):
        def execute(command):
            if command == differential.CLIENT_DIFFERENTIAL:
                raise subprocess.CalledProcessError(1, command)

        with patch("sys.argv", ["runner"]), patch.object(differential, "run", side_effect=execute):
            with self.assertRaises(subprocess.CalledProcessError):
                differential.main()

    def test_modes_cannot_be_combined(self):
        with patch("sys.argv", ["runner", "--check", "--in-process"]), patch.object(differential, "run") as run:
            with self.assertRaises(SystemExit) as error:
                differential.main()
            self.assertEqual(error.exception.code, 2)
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
