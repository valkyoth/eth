# Differential Test Report

Status: `v0.55.0` implementation report.

## Claimed Reference Paths

| Area | Reference | Local test | Result |
| --- | --- | --- | --- |
| Structural RLP | `alloy-rlp` `0.3.16` | `eth-valkyoth-codec::differential_rlp_reference` | Passing locally |
| ModExp arithmetic | `num-bigint` `0.5.1` | `eth-valkyoth-evm-core::modexp_differential` | Passing locally through 256-byte and adversarial operand shapes |

## Evidence

The v0.55.0 implementation and pentest remediation ran:

```sh
scripts/run_differential_tests.py
```

The release gate also runs the same command so the differential claim is not
only a documentation statement.

## Deliberate Limits

These differential paths remain intentionally narrow. They do not compare full
transactions, receipts, MPT proofs, EIP-712 typed data, or integer-domain
semantic validation. Those areas remain planned as the matching protocol layers
gain independent reference adapters.
