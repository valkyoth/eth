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
