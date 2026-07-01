# eth 0.12.0 Release Notes

Status: implementation complete; pentest required before tag

## Summary

`0.12.0` adds the first legacy transaction field decoder. The decoder accepts a
canonical legacy transaction RLP list and returns an explicitly unvalidated
borrowed field model. It does not validate signatures, recover senders, enforce
EIP-155 chain binding, validate gas policy, check account state, or prove fork
validity.

## Added

- Added `eth_valkyoth_protocol::decode_legacy_transaction`.
- Added `UnvalidatedLegacyTransaction` with nonce, gas price, gas limit,
  to/create, value, input, and raw bounded signature words.
- Added `LegacyTransactionTo` for call versus contract creation.
- Added `LegacyTransactionField`, `LegacyTransactionDecodeError`, and stable
  error categories/codes for malformed fields, typed-envelope input, wrong
  field count, and resource exhaustion.
- Re-exported the legacy decode errors from `eth::error`.
- Added `UnvalidatedLegacyTransaction::eip155_chain_id` as a panic-free helper
  for callers that need to inspect EIP-155 chain binding later.
- Extended the transaction-envelope fuzz target to also drive legacy transaction
  field decoding.
- Updated the pinned stable Rust toolchain and release-gate compatibility
  matrix through Rust `1.96.1`.

## Security Notes

- Legacy transaction field decoding is syntactic and bounded only.
- Signature fields are decoded as canonical unsigned U256 words but are not
  checked for secp256k1 validity, low-s, sender recovery, or chain binding.
- Callers should use `eip155_chain_id` instead of unchecked arithmetic on the
  raw `v` word; pre-EIP-155 `v` values and oversized values return `None`.
- The `to` field is either empty contract creation or exactly 20 address bytes.
- The borrowed input field is checked against the active allocation limit even
  though the decoder does not allocate, so callers have one policy knob for
  exposed calldata size.
- Typed transaction payloads remain opaque until later milestones.

## Specification Evidence

- EIP-2718 defines legacy transactions as
  `rlp([nonce, gasPrice, gasLimit, to, value, data, v, r, s])`.
- EIP-155 signing and replay-protection rules remain deferred to later
  validation milestones.
- Official EIP sources were checked on 2026-07-01 while preparing this release.

## Still Required Before Tag

- Maintainer pentest must be run for the exact implementation commit.
- Any pentest findings must be fixed and retested.
- A permanent report must be written at `security/pentest/v0.12.0.md`.
- GitHub checks must pass on the final release report commit.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_0_12_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
