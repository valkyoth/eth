# Block Headers

Status: v0.28.0 adds syntactic execution header decoding and hashing.

`eth-valkyoth-protocol` exposes `decode_block_header` for one canonical RLP
execution header. The caller chooses a `HeaderFieldSet`:

- `Legacy`: 15 fields, pre-London;
- `London`: adds `base_fee_per_gas`;
- `Shanghai`: adds `withdrawals_root`;
- `Cancun`: adds `blob_gas_used`, `excess_blob_gas`, and
  `parent_beacon_block_root`;
- `Prague`: adds `requests_hash`.

The decoder enforces exact field counts for the selected field set and fixed
byte widths for address, hash, logs-bloom, nonce, withdrawal-root,
parent-beacon-root, and requests-hash fields. Integer fields reuse the codec
canonical RLP integer path.

The returned `UnvalidatedBlockHeader` is intentionally not a validity proof. It
does not prove ancestry, fork activation, gas accounting, base-fee calculation,
state root, transaction root, receipt root, logs-bloom correctness,
withdrawals root, blob gas accounting, parent beacon root, requests hash, or
consensus-layer commitments.

Header hashing uses the `eth-valkyoth-hash::Keccak256` trait boundary and
hashes the exact canonical RLP header bytes that were decoded. The result is a
`BlockHash` newtype instead of a raw `B256`, preserving domain separation for
later proof and inclusion APIs.

Specification anchors checked for this release:

- EIP-4895 defines the `withdrawals_root` header extension.
- EIP-4844 defines the `blob_gas_used` and `excess_blob_gas` header extension.
- EIP-4788 defines the `parent_beacon_block_root` header extension.
- EIP-7685 defines the `requests_hash` header extension.
