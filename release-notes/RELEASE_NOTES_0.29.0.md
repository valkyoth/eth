# eth 0.29.0 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging

## Summary

`0.29.0` adds syntactic receipt decoding for legacy and EIP-2718 typed
receipts.

The decoder is intentionally not a receipt-validity engine. It admits one
canonical receipt under explicit `DecodeLimits`, decodes the status/root field,
checks the 256-byte logs bloom, validates borrowed log and topic shape, and
keeps log data borrowed.

## Added

- Added `decode_receipt_envelope` for legacy and typed receipt classification.
- Added `decode_receipt` for legacy and typed receipt payloads.
- Added `UnvalidatedReceipt`, `ReceiptKind`, and
  `ReceiptStatusOrStateRoot`.
- Added `ReceiptLogsBloom`, `ReceiptLogs`, `ReceiptLog`, and borrowed log-topic
  iterators.
- Added stable receipt decode errors and categories.
- Added malformed receipt tests for field counts, status/root shape, log shape,
  topic width, and decode-limit resource exhaustion.
- Added `fuzz/fuzz_targets/receipt.rs`.
- Added `docs/receipts.md`.
- Added a shared internal EIP-2718 prefix classifier used by both transaction
  and receipt envelope decoders to prevent range-classification drift.

## Security Notes

- Receipt decoding remains syntactic. It does not prove transaction execution,
  receipt-trie inclusion, block receipt-root membership, cumulative-gas
  monotonicity, log semantics, or typed receipt/transaction type matching.
- The status/root field accepts only post-Byzantium status `0`/`1` or a
  32-byte pre-Byzantium state root.
- Logs are eagerly shape-checked before the borrowed model is returned. Later
  iteration intentionally re-parses the same bounded RLP bytes.

## Specification Notes

- EIP-658 defines the status-code replacement for the intermediate state root.
- EIP-2718 defines typed receipt envelopes and the legacy receipt field shape.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.24.0`.
- `eth-valkyoth-verify` publishes as `0.17.2` because its published protocol
  dependency range changes.
- The facade crate publishes as `eth` `0.29.0`.
- Other unchanged support crates are not republished.

## Release Gate

- External pentest passed after remediation of the shared EIP-2718 prefix
  classifier finding.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_29_gate.sh
```
