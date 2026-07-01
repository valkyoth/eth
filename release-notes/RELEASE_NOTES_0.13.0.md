# eth 0.13.0 Release Notes

Status: release candidate ready; waiting for final GitHub checks before tag

## Summary

`0.13.0` adds unvalidated EIP-2930 access-list transaction field decoding for
typed transaction byte `0x01`. The decoder accepts
`0x01 || rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
signatureYParity, signatureR, signatureS])` and returns a borrowed field model.

The model is intentionally syntactic. It does not validate signatures, recover
senders, enforce chain binding, account for gas, validate account state, apply a
duplicate access-list policy, or prove fork validity.

## Added

- Added `eth_valkyoth_protocol::decode_access_list_transaction`.
- Added `UnvalidatedAccessListTransaction` with chain ID, nonce, gas price, gas
  limit, to/create, value, input data, access list, y parity, and raw canonical
  U256 signature words.
- Added borrowed `AccessList`, `AccessListEntry`, and
  `AccessListStorageKeys` iterators.
- Added `AccessListTransactionTo` and `SignatureYParity` domain types.
- Added `AccessListTransactionField`, `AccessListTransactionDecodeError`, and
  stable error categories/codes for wrong type, wrong field count, malformed
  fields, invalid address/storage-key lengths, invalid y parity, and resource
  exhaustion.
- Re-exported the access-list transaction decode errors from `eth::error`.
- Extended the transaction-envelope fuzz target to also drive EIP-2930
  access-list transaction decoding.

## Security Notes

- Access lists are decoded through the bounded RLP list machinery before the
  unvalidated transaction model is returned.
- Access-list entry and storage-key iterator types are exported so downstream
  callers can name the vetted borrowed walkers in their own APIs.
- Access-list addresses must be exactly 20 scalar bytes.
- Access-list storage keys must be exactly 32 scalar bytes.
- The borrowed access-list model is eager-validated at decode time. Iterating
  entries or storage keys later intentionally re-parses the same bounded bytes
  instead of allocating decoded storage.
- Duplicate access-list entries and duplicate storage keys are accepted by this
  syntactic decoder. EIP-2930 allows duplicates and charges them multiple times;
  any deployment-specific duplicate policy belongs in a later validation layer.
- Signature `r` and `s` are decoded as canonical unsigned U256 words but are not
  checked for secp256k1 scalar validity or low-s policy.
- `signatureYParity` accepts only `0` and `1`.
- The borrowed input field is checked against the active allocation limit even
  though the decoder does not allocate, so callers have one policy knob for
  exposed calldata size.

## Specification Evidence

- EIP-2930 defines transaction type `0x01` and the access-list payload shape.
- EIP-2930 defines the access list as address plus storage-key tuples and allows
  duplicate addresses and storage keys.
- Official EIP sources were checked on 2026-07-01 while preparing this release.

## Release Gate

- Initial pentest findings were remediated and committed.
- Maintainer retest found no remaining findings for the release candidate.
- Permanent report: `security/pentest/v0.13.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_0_13_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
