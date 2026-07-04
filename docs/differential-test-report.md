# Differential Test Report

Status: v0.36.0 implementation report.

## Claimed Reference Paths

| Area | Reference | Local test | Result |
| --- | --- | --- | --- |
| Structural RLP | `alloy-rlp` `0.3.16` | `eth-valkyoth-codec::differential_rlp_reference` | Passing locally |

## Evidence

The v0.36.0 implementation ran:

```sh
scripts/run_differential_tests.py
```

The release gate also runs the same command so the differential claim is not
only a documentation statement.

## Deliberate Limits

The first differential path is intentionally narrow. It does not compare full
transactions, receipts, MPT proofs, EIP-712 typed data, or integer-domain
semantic validation. Those areas remain planned as the matching protocol layers
gain independent reference adapters.
