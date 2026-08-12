#!/usr/bin/env python3
"""Unit tests for the external-client ModExp differential runner."""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("run_modexp_client_differential.py")
SPEC = importlib.util.spec_from_file_location("modexp_client_differential", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load ModExp client differential runner")
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def object_id() -> str:
    return MODULE.secrets.token_hex(32)


class FakeResponse:
    def __init__(self, tag: str) -> None:
        self.payload = json.dumps({"tag_name": tag}).encode("ascii")

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def read(self, amount: int = -1) -> bytes:
        return self.payload if amount < 0 else self.payload[:amount]


class DifferentialRunnerTests(unittest.TestCase):
    def test_required_clients_use_immutable_digests(self) -> None:
        MODULE.validate_configuration()
        self.assertEqual(
            {client.name for client in MODULE.CLIENTS},
            {"geth", "besu", "nethermind"},
        )
        for client in MODULE.CLIENTS:
            self.assertTrue(client.release_tag)
            self.assertTrue(client.version_marker)

    def test_vector_parser_accepts_byte_aligned_lowercase_hex(self) -> None:
        self.assertEqual(
            MODULE.parse_vectors("scalar\t0x0001\t0x02\n"),
            [("scalar", "0x0001", "0x02")],
        )

    def test_vector_parser_rejects_duplicates_and_malformed_hex(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.parse_vectors("same\t0x00\t0x01\nsame\t0x02\t0x03\n")
        with self.assertRaises(ValueError):
            MODULE.parse_vectors("case\t0x0\t0x01\n")
        with self.assertRaises(ValueError):
            MODULE.parse_vectors("case\t0xAA\t0x01\n")

    def test_latest_release_check_accepts_only_exact_tags(self) -> None:
        current = [FakeResponse(client.release_tag) for client in MODULE.CLIENTS]
        with mock.patch.object(MODULE.urllib.request, "urlopen", side_effect=current):
            MODULE.check_latest_releases()

        with mock.patch.object(
            MODULE.urllib.request, "urlopen", return_value=FakeResponse("stale")
        ):
            with self.assertRaises(RuntimeError):
                MODULE.check_latest_releases()

    def test_bounded_reader_rejects_oversized_payload(self) -> None:
        response = FakeResponse("payload-that-is-too-long")
        with self.assertRaises(RuntimeError):
            MODULE.read_bounded(response, 4, "fixture")

    def test_run_wraps_timeout_and_failure_without_command_arguments(self) -> None:
        with mock.patch.object(
            MODULE.subprocess, "run", side_effect=MODULE.subprocess.TimeoutExpired("tool", 1)
        ):
            with self.assertRaisesRegex(RuntimeError, "command timed out: podman"):
                MODULE.run(["podman", "secret-value"])
        with mock.patch.object(
            MODULE.subprocess,
            "run",
            side_effect=MODULE.subprocess.CalledProcessError(9, ["podman", "secret-value"]),
        ):
            with self.assertRaisesRegex(RuntimeError, "status 9: podman"):
                MODULE.run(["podman", "secret-value"])

    def test_published_port_accepts_only_podman_loopback_mapping(self) -> None:
        completed = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=0, stdout="127.0.0.1:18545\n", stderr=""
        )
        with mock.patch.object(MODULE, "run", return_value=completed):
            self.assertEqual(MODULE.published_port("container"), 18545)
        completed.stdout = "0.0.0.0:18545\n"
        with mock.patch.object(MODULE, "run", return_value=completed):
            with self.assertRaises(RuntimeError):
                MODULE.published_port("container")

    def test_resource_controller_preflight_fails_closed(self) -> None:
        complete = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout='["cpu", "memory", "pids"]\n',
            stderr="",
        )
        with mock.patch.object(MODULE, "run", return_value=complete):
            MODULE.require_resource_controllers()

        incomplete = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=0, stdout='["pids"]\n', stderr=""
        )
        with mock.patch.object(MODULE, "run", return_value=incomplete):
            with self.assertRaisesRegex(RuntimeError, "must delegate"):
                MODULE.require_resource_controllers()

    def test_isolation_preflight_requires_exact_rootless_metadata(self) -> None:
        rootless = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=0, stdout="true\n", stderr=""
        )
        controllers = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout='["cpu", "memory", "pids"]\n',
            stderr="",
        )
        with mock.patch.object(MODULE, "run", side_effect=[rootless, controllers]):
            MODULE.require_isolated_podman()

        for output in ("false\n", "not-json\n"):
            result = MODULE.subprocess.CompletedProcess(
                args=["podman"], returncode=0, stdout=output, stderr=""
            )
            with mock.patch.object(MODULE, "run", return_value=result):
                with self.assertRaises(RuntimeError):
                    MODULE.require_isolated_podman()

    def test_container_boundary_is_loopback_only_and_has_no_mounts(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        command = MODULE.container_command(
            MODULE.CLIENTS[0], "test-client", "test-internal", run_id
        )
        self.assertEqual(command[:2], ["podman", "create"])
        self.assertIn(f"{MODULE.OWNERSHIP_LABEL}={run_id}", command)
        self.assertIn("127.0.0.1::8545", command)
        self.assertIn("test-internal", command)
        self.assertIn("--security-opt=no-new-privileges", command)
        self.assertIn("--cap-drop=all", command)
        self.assertIn("--memory=2g", command)
        self.assertIn("--memory-swap=2g", command)
        self.assertIn("--cpus=2", command)
        self.assertIn("--pids-limit=512", command)
        self.assertIn("--read-only", command)
        self.assertIn(
            "--tmpfs=/data:rw,nosuid,nodev,size=4g,mode=1777",
            command,
        )
        self.assertIn("--log-opt=max-size=10mb", command)
        self.assertNotIn("--volume", command)
        self.assertNotIn("-v", command)
        besu = MODULE.container_command(
            MODULE.CLIENTS[1], "test-besu", "test-internal", run_id
        )
        self.assertIn("--user", besu)
        self.assertIn("1000:1000", besu)

    def test_object_existence_is_strict_and_bounded(self) -> None:
        for status, expected in ((0, True), (1, False)):
            completed = MODULE.subprocess.CompletedProcess(
                args=["podman"], returncode=status, stdout="", stderr=""
            )
            with mock.patch.object(MODULE.subprocess, "run", return_value=completed):
                self.assertEqual(
                    MODULE.podman_object_exists("container", object_id()), expected
                )

        invalid = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=125, stdout="", stderr=""
        )
        with mock.patch.object(MODULE.subprocess, "run", return_value=invalid):
            with self.assertRaisesRegex(RuntimeError, "existence check failed"):
                MODULE.podman_object_exists("network", object_id())

    def test_container_cleanup_fails_if_owned_id_remains(self) -> None:
        with mock.patch.object(MODULE, "run", side_effect=RuntimeError("remove")):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=True):
                with self.assertRaisesRegex(RuntimeError, "container cleanup failed"):
                    MODULE.cleanup_container(object_id())

        with mock.patch.object(MODULE, "run"):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=False):
                MODULE.cleanup_container(object_id())

    def test_creation_error_without_object_does_not_cleanup_name(self) -> None:
        failures = [RuntimeError("create"), RuntimeError("inspect")]
        with mock.patch.object(MODULE, "run", side_effect=failures):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=False):
                with mock.patch.object(MODULE, "cleanup_container") as cleanup:
                    with self.assertRaisesRegex(RuntimeError, "container creation failed"):
                        MODULE.create_container(
                            MODULE.CLIENTS[0],
                            "random-name",
                            object_id(),
                            MODULE.secrets.token_hex(16),
                        )
                    cleanup.assert_not_called()

    def test_uncertain_container_creation_recovers_matching_object(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        identifier = object_id()
        inspected = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout=f"{run_id}\t{identifier}\n",
            stderr="",
        )
        with mock.patch.object(
            MODULE, "run", side_effect=[RuntimeError("timeout"), inspected]
        ):
            with mock.patch.object(MODULE, "cleanup_container") as cleanup:
                with self.assertRaisesRegex(RuntimeError, "container creation failed"):
                    MODULE.create_container(
                        MODULE.CLIENTS[0], "random-name", object_id(), run_id
                    )
                cleanup.assert_called_once_with(identifier)

    def test_uncertain_network_creation_recovers_matching_object(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        identifier = object_id()
        inspected = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout=f"{run_id}\t{identifier}\n",
            stderr="",
        )
        with mock.patch.object(
            MODULE, "run", side_effect=[RuntimeError("timeout"), inspected]
        ):
            with mock.patch.object(MODULE, "cleanup_network") as cleanup:
                with self.assertRaisesRegex(RuntimeError, "network creation failed"):
                    MODULE.create_network("random-network", run_id)
                cleanup.assert_called_once_with(identifier)

    def test_recovery_rejects_wrong_owner_and_inspection_failure(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        wrong_owner = MODULE.secrets.token_hex(16)
        inspected = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout=f"{wrong_owner}\t{object_id()}\n",
            stderr="",
        )
        with mock.patch.object(MODULE, "run", return_value=inspected):
            with self.assertRaisesRegex(RuntimeError, "unexpected object"):
                MODULE.recover_owned_object("container", "random-name", run_id)

        with mock.patch.object(MODULE, "run", side_effect=RuntimeError("inspect")):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=True):
                with self.assertRaisesRegex(RuntimeError, "inspection failed"):
                    MODULE.recover_owned_object("network", "random-network", run_id)

    def test_malformed_container_id_rejects_wrong_label_replacement(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        created = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=0, stdout="malformed\n", stderr=""
        )
        replacement = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout=f"{MODULE.secrets.token_hex(16)}\t{object_id()}\n",
            stderr="",
        )
        with mock.patch.object(MODULE, "run", side_effect=[created, replacement]):
            with mock.patch.object(MODULE, "cleanup_container") as cleanup:
                with self.assertRaisesRegex(RuntimeError, "unexpected object"):
                    MODULE.create_container(
                        MODULE.CLIENTS[0], "random-name", object_id(), run_id
                    )
                cleanup.assert_not_called()

    def test_network_inspection_error_rejects_wrong_label_replacement(self) -> None:
        run_id = MODULE.secrets.token_hex(16)
        created = MODULE.subprocess.CompletedProcess(
            args=["podman"], returncode=0, stdout="random-network\n", stderr=""
        )
        replacement = MODULE.subprocess.CompletedProcess(
            args=["podman"],
            returncode=0,
            stdout=f"{MODULE.secrets.token_hex(16)}\t{object_id()}\n",
            stderr="",
        )
        with mock.patch.object(
            MODULE,
            "run",
            side_effect=[created, RuntimeError("inspect"), replacement],
        ):
            with mock.patch.object(MODULE, "cleanup_network") as cleanup:
                with self.assertRaisesRegex(RuntimeError, "unexpected object"):
                    MODULE.create_network("random-network", run_id)
                cleanup.assert_not_called()

    def test_network_cleanup_failure_is_not_suppressed(self) -> None:
        with mock.patch.object(MODULE, "run", side_effect=RuntimeError("remove")):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=True):
                with self.assertRaisesRegex(RuntimeError, "network cleanup failed"):
                    MODULE.cleanup_network(object_id())

        with mock.patch.object(MODULE, "run"):
            with mock.patch.object(MODULE, "podman_object_exists", return_value=False):
                MODULE.cleanup_network(object_id())


if __name__ == "__main__":
    unittest.main()
