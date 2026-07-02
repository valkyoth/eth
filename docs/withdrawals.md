# Withdrawals

Status: v0.30.0 adds syntactic EIP-4895 withdrawal-list decoding.

`eth-valkyoth-protocol` exposes `decode_withdrawals` for one canonical RLP
withdrawals list. The decoder accepts an explicit `DecodeLimits` policy and
returns `UnvalidatedWithdrawals`.

The decoder supports withdrawals encoded as:

```text
rlp([[index, validator_index, address, amount], ...])
```

Each withdrawal entry has:

- `index`: canonical `uint64` global withdrawal index;
- `validator_index`: canonical `uint64` consensus-layer validator index;
- `address`: 20-byte execution-layer recipient address;
- `amount`: nonzero canonical `uint64` amount in Gwei.

The returned `UnvalidatedWithdrawals` value is intentionally not a validity
proof. It does not prove consensus-layer dequeue correctness, global index
monotonicity, header `withdrawals_root` matching, trie-root membership, or
state-balance application.

Empty withdrawal lists are accepted syntactically. Whether a block can or must
contain withdrawals is a fork and payload-validity rule outside this parser.

Specification anchors checked for this release:

- EIP-4895 defines withdrawals as `[index, validator_index, address, amount]`.
- EIP-4895 defines `amount` as nonzero Gwei.
- EIP-4895 defines `withdrawals_root` as the trie commitment over indexed
  withdrawals; root computation and comparison are planned for later proof
  milestones.
