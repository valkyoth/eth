# eth 0.14.0 Release Notes

Status: implementation ready for pentest

## Summary

`0.14.0` adds unvalidated EIP-1559 dynamic-fee transaction field decoding for
typed transaction byte `0x02`. The decoder accepts
`0x02 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas,
gas_limit, destination, amount, data, access_list, signature_y_parity,
signature_r, signature_s])` and returns a borrowed field model.

The model is intentionally syntactic. It does not validate signatures, recover
senders, enforce chain binding, check `max_fee_per_gas >=
max_priority_fee_per_gas`, account for gas, validate account state, apply a
duplicate access-list policy, or prove fork validity.

## Added

- Added `eth_valkyoth_protocol::decode_dynamic_fee_transaction`.
- Added `UnvalidatedDynamicFeeTransaction` with chain ID, nonce, max priority
  fee per gas, max fee per gas, gas limit, to/create, value, input data, access
  list, y parity, and raw canonical U256 signature words.
- Added `DynamicFeeTransactionTo` as the EIP-1559 call/create target domain.
- Added `DynamicFeeTransactionField`, `DynamicFeeTransactionDecodeError`, and
  stable error categories/codes for wrong type, wrong field count, malformed
  fields, invalid address/storage-key lengths, invalid y parity, and resource
  exhaustion.
- Re-exported the dynamic-fee transaction decode errors from `eth::error`.
- Extended the transaction-envelope fuzz target to also drive EIP-1559
  dynamic-fee transaction decoding.

## Security Notes

- Dynamic-fee transaction field decoding is syntactic and bounded only.
- Access lists use the same borrowed EIP-2930 access-list model and eager
  validation behavior introduced in v0.13.0.
- EIP-2930 and EIP-1559 transaction decoders share the same internal scalar,
  list, address-target, chain-id, and U256 field helpers to avoid validation
  drift as later typed transaction formats are added.
- Whole-payload RLP failures are attributed to the `Payload` field variant for
  both access-list and dynamic-fee transactions instead of being bucketed under
  `AccessList`.
- Dynamic-fee access-list error mapping uses a narrower internal error type so
  future access-list decode changes must be mapped deliberately.
- The decoder intentionally does not reject `max_fee_per_gas` values below
  `max_priority_fee_per_gas`; fee-order validation belongs in a later validation
  state with fork and block context.
- Signature `r` and `s` are decoded as canonical unsigned U256 words but are not
  checked for secp256k1 scalar validity or low-s policy.
- `signature_y_parity` accepts only `0` and `1`.
- The borrowed input field is checked against the active allocation limit even
  though the decoder does not allocate, so callers have one policy knob for
  exposed calldata size.

## Specification Evidence

- EIP-1559 defines transaction type `0x02` and the dynamic-fee payload shape.
- EIP-1559 inherits the access-list structure from EIP-2930.
- Official EIP sources were checked on 2026-07-01 while preparing this release.

## Release Gate

- Initial pentest findings were remediated before retest:
  - shared transaction field helpers replaced duplicated validation code;
  - malformed whole-payload tests were added for EIP-2930 and EIP-1559;
  - dynamic-fee negative-path tests now cover invalid `to`, malformed
    access-list entry shape, invalid storage-key length, and invalid address
    length;
  - dynamic-fee access-list error mapping is exhaustive over the narrower
    internal access-list decode error.
- Pentest is required before the release report commit.
- Permanent report path after pentest: `security/pentest/v0.14.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_0_14_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
