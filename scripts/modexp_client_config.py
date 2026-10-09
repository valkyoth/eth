"""Pinned external clients for the ModExp differential runner."""

from __future__ import annotations

from dataclasses import dataclass


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
        "docker.io/ethereum/client-go@sha256:abf3605177f8bdcfce436985a8054cca4ae59256323a4ffb72c56c04f3d1fadd",
        "https://api.github.com/repos/ethereum/go-ethereum/releases/latest",
        "v1.17.8",
        "Geth/v1.17.8-",
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
        "docker.io/hyperledger/besu@sha256:fc1813d67d630d7660eea6ee555c01e73bfecda3e54d24f9484c1c4a52c6fcb0",
        "https://api.github.com/repos/besu-eth/besu/releases/latest",
        "26.9.0",
        "besu/v26.9.0/",
        "1000:1000",
        (
            "--network=ephemery",
            "--sync-mode=FULL",
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
        "docker.io/nethermind/nethermind@sha256:25f72063f6fc8ba89f1c08abe1ed3211203b3cf303c9618cc3a2488e1e4ab2e0",
        "https://api.github.com/repos/NethermindEth/nethermind/releases/latest",
        "2.1.0",
        "Nethermind/v2.1.0+",
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
