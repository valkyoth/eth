# Receipts

Status: v0.29.0 adds syntactic legacy and typed receipt decoding.

`eth-valkyoth-protocol` exposes `decode_receipt_envelope` for EIP-2718
envelope classification and `decode_receipt` for one canonical RLP receipt
payload.

The decoder supports:

- legacy receipts encoded as `rlp([status_or_root, cumulative_gas_used,
  logs_bloom, logs])`;
- typed receipts encoded as `type_byte || rlp([status_or_root,
  cumulative_gas_used, logs_bloom, logs])`;
- post-Byzantium status codes `0` and `1`;
- pre-Byzantium 32-byte state-root fields;
- borrowed zero-copy log entries, topics, and data.

The returned `UnvalidatedReceipt` is intentionally not a validity proof. It
does not prove transaction execution, receipt-trie inclusion, block
`receipts_root` membership, log semantics, cumulative-gas monotonicity, or that
a typed receipt matches the transaction type at the same block index.

Specification anchors checked for this release:

- EIP-658 defines the post-Byzantium status-code replacement for the
  intermediate state root.
- EIP-2718 defines typed receipts as `TransactionType || ReceiptPayload` and
  legacy receipts as `rlp([status, cumulativeGasUsed, logsBloom, logs])`.
