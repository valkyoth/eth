#!/usr/bin/env python3
"""Compare paid G2ADD fixtures with three digest-pinned, isolated execution clients."""

import argparse
import json
import secrets
import sys
import urllib.request

import run_modexp_client_differential as runner

PRECOMPILE = "0x000000000000000000000000000000000000000d"


def load_vectors():
    output = runner.run(["cargo", "run", "--quiet", "-p", "eth-valkyoth-evm-core",
                         "--example", "bls12_g2_client_vectors"], capture=True).stdout
    vectors = []
    names = set()
    for line in output.splitlines():
        fields = line.split("\t")
        if len(fields) != 3:
            raise ValueError("invalid G2 vector columns")
        name, data, expected = fields
        if (not runner.CASE_PATTERN.fullmatch(name) or name in names
                or not runner.HEX_PATTERN.fullmatch(data)
                or (expected != "ERROR" and not runner.HEX_PATTERN.fullmatch(expected))):
            raise ValueError("invalid G2 vector")
        if expected != "ERROR" and (len(data) != 1026 or len(expected) != 514):
            raise ValueError("invalid positive G2 frame length")
        names.add(name)
        vectors.append((name, data, expected))
    if (len(vectors) != 16 or sum(v[2] == "ERROR" for v in vectors) != 7):
        raise ValueError("incomplete G2 fixture set")
    return vectors


def call(port, data):
    payload = {"jsonrpc": "2.0", "id": 1, "method": "eth_call", "params": [
        {"to": PRECOMPILE, "data": data, "gas": runner.RPC_GAS}, "latest"]}
    request = urllib.request.Request(f"http://127.0.0.1:{port}",
        data=json.dumps(payload).encode("ascii"), headers={"Content-Type": "application/json"})
    with runner.RPC_OPENER.open(request, timeout=runner.RPC_TIMEOUT_SECONDS) as response:
        document = json.loads(runner.read_bounded(response, runner.MAX_RPC_RESPONSE_BYTES, "RPC"))
    if (not isinstance(document, dict) or document.get("jsonrpc") != "2.0"
            or type(document.get("id")) is not int or document["id"] != 1
            or (("error" in document) == ("result" in document))):
        raise RuntimeError("malformed G2 RPC response")
    if "error" in document:
        error = document["error"]
        if (not isinstance(error, dict) or type(error.get("code")) is not int
                or not isinstance(error.get("message"), str)):
            raise RuntimeError("malformed G2 RPC error")
        # Protocol/method/parameter errors are harness failures, not execution rejection.
        if error["code"] not in (-32000, -32015, 3):
            raise RuntimeError(f"unexpected G2 RPC error: {error!r}")
        message = error["message"].lower()
        if not any(marker in message for marker in (
                "out of gas", "outofgas", "invalid input", "invalid point",
                "invalid field", "invalid fp.element encoding", "precompile", "precompiled")):
            raise RuntimeError(f"unrecognized G2 execution error: {error!r}")
        return "ERROR"
    result = document["result"]
    if not isinstance(result, str) or not runner.HEX_PATTERN.fullmatch(result):
        raise RuntimeError("invalid G2 result hex")
    return result


def compare(client, vectors, network, run_id):
    container = runner.create_container(client, f"eth-g2-{client.name}-{run_id}", network, run_id)
    try:
        runner.run(["podman", "start", container], capture=True)
        port = runner.published_port(container)
        version = runner.wait_for_rpc(container, port)
        if client.version_marker not in version:
            raise RuntimeError(f"unexpected client identity: {version!r}")
        for name, data, expected in vectors:
            if call(port, data) != expected:
                raise RuntimeError(f"{client.name}/{name}: G2 outcome mismatch")
        return version
    except Exception:
        runner.write_bounded_logs(container, f"g2-{client.name}")
        raise
    finally:
        runner.cleanup_container(container)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    runner.validate_configuration()
    vectors = load_vectors()
    if args.check:
        print(f"validated {len(vectors)} paid G2 vectors and three immutable client pins")
        return 0
    runner.require_isolated_podman()
    runner.check_latest_releases()
    for client in runner.CLIENTS:
        runner.run(["podman", "pull", client.image], capture=True,
                   timeout=runner.IMAGE_PULL_TIMEOUT_SECONDS)
    run_id = secrets.token_hex(16)
    network = runner.create_network(f"eth-g2-differential-{run_id}", run_id)
    try:
        for client in runner.CLIENTS:
            version = compare(client, vectors, network, run_id)
            print(f"{client.name}\t{version}\t{client.image}\t{len(vectors)} vectors passed", flush=True)
    finally:
        runner.cleanup_network(network)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
