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


class FakeResponse:
    def __init__(self, tag: str) -> None:
        self.payload = json.dumps({"tag_name": tag}).encode("ascii")

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def read(self) -> bytes:
        return self.payload


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

    def test_container_boundary_is_loopback_only_and_has_no_mounts(self) -> None:
        command = MODULE.container_command(
            MODULE.CLIENTS[0], "test-client", "test-internal", 18545
        )
        self.assertIn("127.0.0.1:18545:8545", command)
        self.assertIn("test-internal", command)
        self.assertIn("--security-opt=no-new-privileges", command)
        self.assertNotIn("--volume", command)
        self.assertNotIn("-v", command)


if __name__ == "__main__":
    unittest.main()
