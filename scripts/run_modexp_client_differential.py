#!/usr/bin/env python3
"""Compare first-party ModExp output with pinned Ethereum clients."""

from __future__ import annotations

import argparse
import json
import os
import re
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
COMMAND_TIMEOUT_SECONDS = 30
IMAGE_PULL_TIMEOUT_SECONDS = 300
MAX_RPC_RESPONSE_BYTES = 64 * 1024
MAX_RELEASE_RESPONSE_BYTES = 128 * 1024
MAX_SAVED_LOG_CHARS = 256 * 1024
IMAGE_PATTERN = re.compile(r"^[a-z0-9./-]+@sha256:[0-9a-f]{64}$")
HEX_PATTERN = re.compile(r"^0x(?:[0-9a-f]{2})*$")
CASE_PATTERN = re.compile(r"^[a-z0-9-]+$")
PORT_PATTERN = re.compile(r"^127\.0\.0\.1:([0-9]{1,5})$")
RPC_OPENER = urllib.request.build_opener(urllib.request.ProxyHandler({}))


@dataclass(frozen=True)
class Client:
    name: str
    image: str
    release_api: str
    release_tag: str
    version_marker: str
    user: str | None
    arguments: tuple[str, ...]


CLIENTS = (
    Client(
        "geth",
        "docker.io/ethereum/client-go@sha256:523d3ba26623a619e912019068dc2784f02934070ac46bdae4d5b9df0d917814",
        "https://api.github.com/repos/ethereum/go-ethereum/releases/latest",
        "v1.17.5",
        "Geth/v1.17.5-",
        None,
        (
            "--dev",
            "--http",
            "--http.addr=0.0.0.0",
            "--http.port=8545",
            "--http.api=eth,web3",
            "--http.vhosts=*",
            "--nodiscover",
            "--datadir=/data/geth",
        ),
    ),
    Client(
        "besu",
        "docker.io/hyperledger/besu@sha256:5c319f8f5f3449438c03ea7fa2c9bf24b866dc55ac98d802bb41ad793e740587",
        "https://api.github.com/repos/besu-eth/besu/releases/latest",
        "26.7.1",
        "besu/v26.7.1/",
        "1000:1000",
        (
            "--network=dev",
            "--rpc-http-enabled=true",
            "--rpc-http-host=0.0.0.0",
            "--rpc-http-port=8545",
            "--rpc-http-api=ETH,WEB3",
            "--host-allowlist=*",
            "--p2p-enabled=false",
            "--data-path=/data/besu",
        ),
    ),
    Client(
        "nethermind",
        "docker.io/nethermind/nethermind@sha256:1b6b01419de4ff75ed3d61995904bccc2fdcc2865fee6dae07d88c14a0758e40",
        "https://api.github.com/repos/NethermindEth/nethermind/releases/latest",
        "1.39.3",
        "Nethermind/v1.39.3+",
        None,
        (
            "--config=spaceneth",
            "--JsonRpc.Host=0.0.0.0",
            "--JsonRpc.Port=8545",
            "--JsonRpc.EnabledModules=Eth,Web3",
            "--Init.WebSocketsEnabled=false",
            "--Network.ExternalIp=127.0.0.1",
            "--Network.LocalIp=127.0.0.1",
            "--Init.BaseDbPath=/data/nethermind",
            "--Init.LogDirectory=/data/nethermind-logs",
            "--KeyStore.KeyStoreDirectory=/data/nethermind-keystore",
        ),
    ),
)


def run(
    command: list[str],
    *,
    capture: bool = False,
    timeout: int = COMMAND_TIMEOUT_SECONDS,
) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            command,
            cwd=ROOT,
            check=True,
            text=True,
            capture_output=capture,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(f"command timed out: {command[0]}") from error
    except subprocess.CalledProcessError as error:
        raise RuntimeError(
            f"command failed with status {error.returncode}: {command[0]}"
        ) from error


def read_bounded(response: object, limit: int, source: str) -> bytes:
    read = getattr(response, "read", None)
    if not callable(read):
        raise RuntimeError(f"{source} response is unreadable")
    payload = read(limit + 1)
    if not isinstance(payload, bytes) or len(payload) > limit:
        raise RuntimeError(f"{source} response exceeds its byte limit")
    return payload


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
            document = json.loads(
                read_bounded(response, MAX_RELEASE_RESPONSE_BYTES, "release API")
            )
        actual = document.get("tag_name") if isinstance(document, dict) else None
        if actual != client.release_tag:
            raise RuntimeError(
                f"{client.name} pin is stale: expected latest {client.release_tag}, got {actual!r}"
            )


def require_resource_controllers() -> None:
    result = run(
        ["podman", "info", "--format", "{{json .Host.CgroupControllers}}"],
        capture=True,
    )
    try:
        controllers = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("Podman returned malformed cgroup-controller metadata") from error
    if not isinstance(controllers, list) or not all(
        isinstance(controller, str) for controller in controllers
    ):
        raise RuntimeError("Podman returned invalid cgroup-controller metadata")
    missing = {"cpu", "memory", "pids"}.difference(controllers)
    if missing:
        raise RuntimeError(
            "Podman must delegate cpu, memory, and pids controllers for client isolation"
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
    with RPC_OPENER.open(request, timeout=RPC_TIMEOUT_SECONDS) as response:
        document = json.loads(read_bounded(response, MAX_RPC_RESPONSE_BYTES, "RPC"))
    if not isinstance(document, dict) or document.get("id") != 1:
        raise RuntimeError("malformed JSON-RPC response")
    if "error" in document:
        raise RuntimeError(f"JSON-RPC error: {document['error']!r}")
    if "result" not in document:
        raise RuntimeError("JSON-RPC response omitted result")
    return document["result"]


def container_running(container_id: str) -> bool:
    try:
        state = run(
            ["podman", "inspect", "--format", "{{.State.Running}}", container_id],
            capture=True,
        ).stdout.strip()
    except RuntimeError:
        return False
    return state == "true"


def wait_for_rpc(container_id: str, port: int) -> str:
    deadline = time.monotonic() + START_TIMEOUT_SECONDS
    last_error = "RPC did not answer"
    while time.monotonic() < deadline:
        try:
            version = rpc(port, "web3_clientVersion", [])
            if not isinstance(version, str) or not version:
                raise RuntimeError("client version is not a non-empty string")
            return version
        except (OSError, ValueError, RuntimeError, urllib.error.URLError) as error:
            last_error = str(error)
            if not container_running(container_id):
                raise RuntimeError("client exited before RPC readiness") from error
            time.sleep(0.25)
    raise TimeoutError(f"client RPC startup timed out: {last_error}")


def container_command(client: Client, name: str, network: str) -> list[str]:
    command = [
        "podman",
        "run",
        "--detach",
        "--rm",
        "--pull=never",
        "--name",
        name,
        "--network",
        network,
        "--publish",
        "127.0.0.1::8545",
        "--security-opt=no-new-privileges",
        "--cap-drop=all",
        "--memory=2g",
        "--memory-swap=2g",
        "--cpus=2",
        "--pids-limit=512",
        "--read-only",
        "--tmpfs=/tmp:rw,nosuid,nodev,size=512m,mode=1777",
        "--tmpfs=/data:rw,nosuid,nodev,size=4g,mode=1777",
        "--log-driver=k8s-file",
        "--log-opt=max-size=10mb",
    ]
    if client.user is not None:
        command.extend(["--user", client.user])
    command.extend([client.image, *client.arguments])
    return command


def published_port(container_id: str) -> int:
    mapping = run(
        ["podman", "port", container_id, "8545/tcp"], capture=True
    ).stdout.strip()
    matched = PORT_PATTERN.fullmatch(mapping)
    if matched is None:
        raise RuntimeError("Podman returned an invalid loopback port mapping")
    port = int(matched.group(1))
    if not 1 <= port <= 65_535:
        raise RuntimeError("Podman returned an out-of-range loopback port")
    return port


def write_bounded_logs(name: str, client_name: str) -> Path | None:
    try:
        result = run(["podman", "logs", "--tail", "200", name], capture=True)
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        relative = Path("target") / "modexp-client-differential" / f"{client_name}.log"
        contents = f"{result.stdout}\n{result.stderr}"[-MAX_SAVED_LOG_CHARS:]
        (ROOT / relative).write_text(contents, encoding="utf-8")
        return relative
    except (OSError, RuntimeError):
        return None


def cleanup_container(name: str) -> None:
    for command in (
        ["podman", "stop", "--time", "10", name],
        ["podman", "rm", "--force", name],
    ):
        try:
            run(command, capture=True, timeout=20)
        except RuntimeError:
            pass


def compare_client(
    client: Client, vectors: list[tuple[str, str, str]], network: str
) -> str:
    name = f"eth-modexp-{client.name}-{os_process_id()}"
    container_id: str | None = None
    try:
        try:
            container_id = run(
                container_command(client, name, network), capture=True
            ).stdout.strip()
        except RuntimeError as error:
            raise RuntimeError(f"{client.name} container launch failed") from error
        if not container_id:
            raise RuntimeError(f"{client.name} did not return a container identity")
        port = published_port(container_id)
        version = wait_for_rpc(container_id, port)
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
                raise AssertionError(f"{client.name}/{case_name}: output mismatch")
        return version
    except Exception as error:
        log_path = (
            write_bounded_logs(name, client.name) if container_id is not None else None
        )
        diagnostic = f"; bounded log: {log_path}" if log_path is not None else ""
        raise RuntimeError(f"{error}{diagnostic}") from error
    finally:
        cleanup_container(name)


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
    require_resource_controllers()
    check_latest_releases()
    for client in CLIENTS:
        run(
            ["podman", "pull", client.image],
            capture=True,
            timeout=IMAGE_PULL_TIMEOUT_SECONDS,
        )
    network = f"eth-modexp-differential-{os_process_id()}"
    try:
        run(["podman", "network", "create", "--internal", network], capture=True)
        for client in CLIENTS:
            version = compare_client(client, vectors, network)
            print(f"{client.name}\t{version}\t{len(vectors)} vectors passed")
    finally:
        try:
            run(["podman", "network", "rm", network], capture=True, timeout=20)
        except RuntimeError:
            pass
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, AssertionError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
