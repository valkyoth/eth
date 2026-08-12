#!/usr/bin/env python3
"""Compare first-party ModExp output with pinned Ethereum clients."""

from __future__ import annotations

import argparse
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOG_DIR = ROOT / "target" / "modexp-client-differential"
PRECOMPILE = "0x0000000000000000000000000000000000000005"
RPC_GAS = "0x4c4b40"
START_TIMEOUT_SECONDS = 120
RPC_TIMEOUT_SECONDS = 10
IMAGE_PATTERN = re.compile(r"^[a-z0-9./-]+@sha256:[0-9a-f]{64}$")
HEX_PATTERN = re.compile(r"^0x(?:[0-9a-f]{2})*$")
CASE_PATTERN = re.compile(r"^[a-z0-9-]+$")


@dataclass(frozen=True)
class Client:
    name: str
    image: str
    release_api: str
    release_tag: str
    version_marker: str
    arguments: tuple[str, ...]


CLIENTS = (
    Client(
        "geth",
        "docker.io/ethereum/client-go@sha256:523d3ba26623a619e912019068dc2784f02934070ac46bdae4d5b9df0d917814",
        "https://api.github.com/repos/ethereum/go-ethereum/releases/latest",
        "v1.17.5",
        "Geth/v1.17.5-",
        (
            "--dev",
            "--http",
            "--http.addr=0.0.0.0",
            "--http.port=8545",
            "--http.api=eth,web3",
            "--http.vhosts=*",
            "--nodiscover",
        ),
    ),
    Client(
        "besu",
        "docker.io/hyperledger/besu@sha256:5c319f8f5f3449438c03ea7fa2c9bf24b866dc55ac98d802bb41ad793e740587",
        "https://api.github.com/repos/besu-eth/besu/releases/latest",
        "26.7.1",
        "besu/v26.7.1/",
        (
            "--network=dev",
            "--rpc-http-enabled=true",
            "--rpc-http-host=0.0.0.0",
            "--rpc-http-port=8545",
            "--rpc-http-api=ETH,WEB3",
            "--host-allowlist=*",
            "--p2p-enabled=false",
            "--data-path=/tmp/besu",
        ),
    ),
    Client(
        "nethermind",
        "docker.io/nethermind/nethermind@sha256:1b6b01419de4ff75ed3d61995904bccc2fdcc2865fee6dae07d88c14a0758e40",
        "https://api.github.com/repos/NethermindEth/nethermind/releases/latest",
        "1.39.3",
        "Nethermind/v1.39.3+",
        (
            "--config=spaceneth",
            "--JsonRpc.Host=0.0.0.0",
            "--JsonRpc.Port=8545",
            "--JsonRpc.EnabledModules=Eth,Web3",
            "--Init.WebSocketsEnabled=false",
            "--Network.ExternalIp=127.0.0.1",
            "--Network.LocalIp=127.0.0.1",
        ),
    ),
)


def run(command: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=capture,
    )


def validate_configuration() -> None:
    if {client.name for client in CLIENTS} != {"geth", "besu", "nethermind"}:
        raise ValueError("the required client set is incomplete")
    for client in CLIENTS:
        if not IMAGE_PATTERN.fullmatch(client.image):
            raise ValueError(f"{client.name} image is not immutably pinned")
        if not client.release_api.startswith("https://api.github.com/repos/"):
            raise ValueError(f"{client.name} release API is not official GitHub HTTPS")
        if not client.release_tag or not client.version_marker:
            raise ValueError(f"{client.name} release identity is incomplete")


def check_latest_releases() -> None:
    for client in CLIENTS:
        request = urllib.request.Request(
            client.release_api,
            headers={"User-Agent": "valkyoth-eth-modexp-differential/0.55.0"},
        )
        with urllib.request.urlopen(request, timeout=RPC_TIMEOUT_SECONDS) as response:
            document = json.loads(response.read())
        actual = document.get("tag_name") if isinstance(document, dict) else None
        if actual != client.release_tag:
            raise RuntimeError(
                f"{client.name} pin is stale: expected latest {client.release_tag}, got {actual!r}"
            )


def parse_vectors(output: str) -> list[tuple[str, str, str]]:
    vectors: list[tuple[str, str, str]] = []
    names: set[str] = set()
    for line_number, line in enumerate(output.splitlines(), start=1):
        fields = line.split("\t")
        if len(fields) != 3:
            raise ValueError(f"invalid vector line {line_number}")
        name, calldata, expected = fields
        if not CASE_PATTERN.fullmatch(name) or name in names:
            raise ValueError(f"invalid or duplicate case name on line {line_number}")
        if not HEX_PATTERN.fullmatch(calldata) or not HEX_PATTERN.fullmatch(expected):
            raise ValueError(f"invalid lowercase byte-aligned hex on line {line_number}")
        names.add(name)
        vectors.append((name, calldata, expected))
    if not vectors:
        raise ValueError("the first-party vector set is empty")
    return vectors


def load_vectors() -> list[tuple[str, str, str]]:
    command = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "eth-valkyoth-evm-core",
        "--features",
        "std",
        "--example",
        "modexp_client_vectors",
    ]
    return parse_vectors(run(command, capture=True).stdout)


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def rpc(port: int, method: str, params: list[object]) -> object:
    payload = json.dumps(
        {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    ).encode("ascii")
    request = urllib.request.Request(
        f"http://127.0.0.1:{port}",
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=RPC_TIMEOUT_SECONDS) as response:
        document = json.loads(response.read())
    if not isinstance(document, dict) or document.get("id") != 1:
        raise RuntimeError("malformed JSON-RPC response")
    if "error" in document:
        raise RuntimeError(f"JSON-RPC error: {document['error']!r}")
    if "result" not in document:
        raise RuntimeError("JSON-RPC response omitted result")
    return document["result"]


def wait_for_rpc(process: subprocess.Popen[bytes], port: int) -> str:
    deadline = time.monotonic() + START_TIMEOUT_SECONDS
    last_error = "RPC did not answer"
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"client exited before RPC readiness: {process.returncode}")
        try:
            version = rpc(port, "web3_clientVersion", [])
            if not isinstance(version, str) or not version:
                raise RuntimeError("client version is not a non-empty string")
            return version
        except (OSError, ValueError, RuntimeError, urllib.error.URLError) as error:
            last_error = str(error)
            time.sleep(0.25)
    raise TimeoutError(f"client RPC startup timed out: {last_error}")


def container_command(client: Client, name: str, network: str, port: int) -> list[str]:
    return [
        "podman",
        "run",
        "--rm",
        "--name",
        name,
        "--network",
        network,
        "--publish",
        f"127.0.0.1:{port}:8545",
        "--security-opt=no-new-privileges",
        client.image,
        *client.arguments,
    ]


def compare_client(
    client: Client, vectors: list[tuple[str, str, str]], network: str
) -> str:
    port = free_port()
    name = f"eth-modexp-{client.name}-{os_process_id()}"
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_path = LOG_DIR / f"{client.name}.log"
    with log_path.open("wb") as log:
        process = subprocess.Popen(
            container_command(client, name, network, port),
            cwd=ROOT,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
        try:
            version = wait_for_rpc(process, port)
            if client.version_marker not in version:
                raise RuntimeError(
                    f"runtime identity {version!r} does not match {client.version_marker!r}"
                )
            for case_name, calldata, expected in vectors:
                actual = rpc(
                    port,
                    "eth_call",
                    [{"to": PRECOMPILE, "data": calldata, "gas": RPC_GAS}, "latest"],
                )
                if not isinstance(actual, str) or actual.lower() != expected:
                    raise AssertionError(
                        f"{client.name}/{case_name}: expected {expected}, got {actual!r}"
                    )
            return version
        except Exception as error:
            raise RuntimeError(f"{error}; client log: {log_path}") from error
        finally:
            subprocess.run(
                ["podman", "stop", "--time", "10", name],
                cwd=ROOT,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def os_process_id() -> int:
    return os.getpid()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="compile the vector producer and validate immutable client configuration",
    )
    args = parser.parse_args()
    validate_configuration()
    vectors = load_vectors()
    if args.check:
        print(f"validated {len(vectors)} ModExp vectors and {len(CLIENTS)} client pins")
        return 0

    run(["podman", "--version"], capture=True)
    check_latest_releases()
    network = f"eth-modexp-differential-{os_process_id()}"
    run(["podman", "network", "create", "--internal", network], capture=True)
    try:
        for client in CLIENTS:
            version = compare_client(client, vectors, network)
            print(f"{client.name}\t{version}\t{len(vectors)} vectors passed")
    finally:
        subprocess.run(
            ["podman", "network", "rm", network],
            cwd=ROOT,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, AssertionError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
