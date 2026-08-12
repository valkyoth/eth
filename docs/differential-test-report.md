# Differential Test Report

Status: `v0.55.0` implementation report.

## Claimed Reference Paths

| Area | Reference | Local test | Result |
| --- | --- | --- | --- |
| Structural RLP | `alloy-rlp` `0.3.16` | `eth-valkyoth-codec::differential_rlp_reference` | Passing locally |
| ModExp arithmetic | `num-bigint` `0.5.1` | `eth-valkyoth-evm-core::modexp_differential` | Passing locally through 256-byte and adversarial operand shapes |
| ModExp client | Geth `1.17.5` | `eth_call` to precompile `0x05` | All 11 outputs match |
| ModExp client | Besu `26.7.1` | `eth_call` to precompile `0x05` | All 11 outputs match |
| ModExp client | Nethermind `1.39.3` | `eth_call` to precompile `0x05` | All 11 outputs match |

## Evidence

The v0.55.0 implementation and pentest remediation ran:

```sh
scripts/run_differential_tests.py
```

The release gate also runs the same command so the differential claim is not
only a documentation statement.

The external-client comparison was executed on 2026-08-12 with these immutable
multi-platform image identities and runtime-reported versions:

| Client | OCI index digest | Runtime identity |
| --- | --- | --- |
| [Geth `1.17.5`](https://github.com/ethereum/go-ethereum/releases/tag/v1.17.5) | `sha256:523d3ba26623a619e912019068dc2784f02934070ac46bdae4d5b9df0d917814` | `Geth/v1.17.5-stable-9621c6ad/linux-amd64/go1.26.5` |
| [Besu `26.7.1`](https://github.com/besu-eth/besu/releases/tag/26.7.1) | `sha256:5c319f8f5f3449438c03ea7fa2c9bf24b866dc55ac98d802bb41ad793e740587` | `besu/v26.7.1/linux-x86_64/openjdk-java-25` |
| [Nethermind `1.39.3`](https://github.com/NethermindEth/nethermind/releases/tag/1.39.3) | `sha256:1b6b01419de4ff75ed3d61995904bccc2fdcc2865fee6dae07d88c14a0758e40` | `Nethermind/v1.39.3+28cbe2a0/linux-x64/dotnet10.0.10` |

All clients passed scalar, leading-zero modulus boundaries, unequal widths,
even and zero moduli, sparse exponent, virtual right-padding, and 80/256-byte
operand cases. The runner validated each runtime identity against its expected
stable release and each pin against the official latest-release API before
accepting the comparison.

The pentest remediation adds mandatory rootless execution plus CPU, memory,
PID, filesystem, log, subprocess-time, port-binding, proxy, response-size,
object-ownership, and verified-cleanup isolation to the same comparison. A
rootful host or a host without delegated `cpu`, `memory`, and `pids` cgroup v2
controllers is rejected before client execution; there is no weaker release
mode. Containers are created before they are started so cleanup operates on a
confirmed immutable ID, network creation is tracked independently, and the
gate fails when any owned object remains. The remediation host exposed only
`pids`, so the fail-closed preflight was verified there and all remaining
hardened controls independently completed all 33 comparisons. The exact-commit
retest and release gate must run the complete command on a rootless host with
all three controllers delegated.

## Deliberate Limits

These differential paths remain intentionally narrow. External-client results
cover output bytes, not gas across unlike development-chain fork schedules.
They also do not compare full transactions, receipts, MPT proofs, EIP-712 typed
data, or integer-domain semantic validation. Those areas remain planned as the
matching protocol layers gain independent reference adapters.
