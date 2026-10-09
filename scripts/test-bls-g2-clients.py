#!/usr/bin/env python3
"""Fail-closed G2 oracle transport and owned-container lifecycle regressions."""
import io
import json
import unittest
from types import SimpleNamespace
from unittest.mock import patch

import run_bls_g2_client_differential as g2


class G2ClientTests(unittest.TestCase):
    def call(self, doc):
        with patch.object(g2.runner.RPC_OPENER, "open", return_value=io.BytesIO(json.dumps(doc).encode())):
            return g2.call(1234, "0x")

    def test_valid_success_and_execution_rejection(self):
        self.assertEqual(self.call(dict(jsonrpc="2.0", id=1, result="0x00")), "0x00")
        for code in (-32000, -32015, 3):
            self.assertEqual(self.call(dict(jsonrpc="2.0", id=1,
                error=dict(code=code, message="invalid input length"))), "ERROR")

    def test_protocol_and_infrastructure_errors_are_not_negative_vectors(self):
        self.assertEqual(self.call(dict(jsonrpc="2.0", id=1,
            error=dict(code=-32000, message="invalid fp.Element encoding"))), "ERROR")
        for code, message in ((-32601, "method not found"), (-32602, "invalid input"),
                              (-32000, "rate limit exceeded"), (-32000, "internal error")):
            with self.subTest(code=code, message=message), self.assertRaises(RuntimeError):
                self.call(dict(jsonrpc="2.0", id=1, error=dict(code=code, message=message)))

    def test_malformed_responses_fail(self):
        for doc in ([], {}, dict(jsonrpc="2.0", id=True, result="0x"),
                    dict(jsonrpc="2.0", id=2, result="0x"),
                    dict(jsonrpc="2.0", id=1, result="0x", error={}),
                    dict(jsonrpc="2.0", id=1, result="0x0"),
                    dict(jsonrpc="2.0", id=1, error=dict(code=True, message="out of gas"))):
            with self.subTest(doc=doc), self.assertRaises(RuntimeError):
                self.call(doc)

    def test_transport_failure_is_not_an_execution_rejection(self):
        with patch.object(g2.runner.RPC_OPENER, "open", side_effect=TimeoutError):
            with self.assertRaises(TimeoutError):
                g2.call(1234, "0x")

    @staticmethod
    def vectors():
        return "\n".join([f"valid-{n}\t0x{'00' * 512}\t0x{'00' * 256}" for n in range(9)]
                         + [f"invalid-{n}\t0x\tERROR" for n in range(7)])

    def test_complete_vector_set_is_required(self):
        body = self.vectors()
        for output in ("", body.splitlines()[0], body + "\n" + body.splitlines()[0],
                       body.replace("valid-0", "valid 0"), body.replace("0x", "0x0", 1)):
            with self.subTest(output=output[:40]), patch.object(g2.runner, "run", return_value=SimpleNamespace(stdout=output)):
                with self.assertRaises(ValueError):
                    g2.load_vectors()
        with patch.object(g2.runner, "run", return_value=SimpleNamespace(stdout=body)):
            self.assertEqual(len(g2.load_vectors()), 16)

    def test_cleanup_after_success_mismatch_and_startup_failure(self):
        client = g2.runner.CLIENTS[0]
        for mode in ("success", "mismatch", "startup", "identity"):
            with self.subTest(mode=mode), \
                 patch.object(g2.runner, "create_container", return_value="owned") as create, \
                 patch.object(g2.runner, "run", side_effect=RuntimeError("start") if mode == "startup" else None), \
                 patch.object(g2.runner, "published_port", return_value=1234), \
                 patch.object(g2.runner, "wait_for_rpc", return_value="wrong" if mode == "identity" else client.version_marker), \
                 patch.object(g2, "call", return_value="0x01" if mode == "mismatch" else "0x00"), \
                 patch.object(g2.runner, "write_bounded_logs") as logs, \
                 patch.object(g2.runner, "cleanup_container") as cleanup:
                if mode == "success":
                    self.assertEqual(g2.compare(client, [("a", "0x", "0x00")], "network", "run"), client.version_marker)
                    logs.assert_not_called()
                else:
                    with self.assertRaises(RuntimeError):
                        g2.compare(client, [("a", "0x", "0x00")], "network", "run")
                    logs.assert_called_once()
                create.assert_called_once()
                cleanup.assert_called_once_with("owned")

    def test_stopped_containers_are_retained_for_logs_until_explicit_cleanup(self):
        command = g2.runner.container_command(g2.runner.CLIENTS[0], "name", "net", "run")
        self.assertNotIn("--rm", command)
        for option in ("--read-only", "--cap-drop=all", "--memory=2g", "--pids-limit=512"):
            self.assertIn(option, command)


if __name__ == "__main__":
    unittest.main()
