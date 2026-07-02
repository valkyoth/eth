# eth 0.30.0 Release Notes

Status: implementation ready for external pentest

## Summary

`0.30.0` adds syntactic EIP-4895 withdrawal-list decoding.

The decoder is intentionally not a withdrawal-validity or block-validity
engine. It admits one canonical withdrawals list under explicit `DecodeLimits`,
checks each withdrawal entry shape, validates canonical `uint64` indexes,
checks the 20-byte recipient address, and rejects zero Gwei amounts.

## Added

- Added `decode_withdrawals` for canonical EIP-4895 withdrawal lists.
- Added `UnvalidatedWithdrawals`, `UnvalidatedWithdrawal`,
  `WithdrawalIndex`, `WithdrawalValidatorIndex`, and
  `WithdrawalAmountGwei`.
- Added stable withdrawal decode errors and categories.
- Added malformed withdrawal tests for field counts, address width,
  noncanonical integer fields, zero amounts, and decode-limit resource
  exhaustion.
- Added `fuzz/fuzz_targets/withdrawal.rs`.
- Added `docs/withdrawals.md`.

## Security Notes

- Withdrawal decoding remains syntactic. It does not prove consensus-layer
  dequeue correctness, global index monotonicity, header `withdrawals_root`
  matching, trie-root membership, or state-balance application.
- Empty withdrawal lists are accepted syntactically. Fork and payload validity
  rules remain outside this parser.
- Withdrawal entries are eagerly shape-checked before the borrowed model is
  returned. Later iteration intentionally re-parses the same bounded RLP bytes.

## Specification Notes

- EIP-4895 defines withdrawals as RLP lists shaped
  `[index, validator_index, address, amount]`.
- EIP-4895 defines `index` and `validator_index` as `uint64`, `address` as
  20 bytes, and `amount` as a nonzero `uint64` in Gwei.
- EIP-4895 defines `withdrawals_root` as a trie commitment over indexed
  withdrawals; root computation and comparison remain planned for later proof
  releases.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.25.0`.
- `eth-valkyoth-verify` publishes as `0.17.3` because its published protocol
  dependency range changes.
- The facade crate publishes as `eth` `0.30.0`.
- Other unchanged support crates are not republished.

## Release Gate

- External pentest is required before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_30_gate.sh
```
