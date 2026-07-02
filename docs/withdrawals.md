# Withdrawals

Status: v0.30.0 adds syntactic EIP-4895 withdrawal-list decoding.

`eth-valkyoth-protocol` exposes `decode_withdrawals` for one canonical RLP
withdrawals list. The decoder accepts an explicit `DecodeLimits` policy and
returns `UnvalidatedWithdrawals`.

The decoder supports withdrawals encoded as:

```text
rlp([[index, validator_index, address, amount], ...])
```

Each admitted withdrawal entry has:

- `index`: canonical `uint64` global withdrawal index;
- `validator_index`: canonical `uint64` consensus-layer validator index;
- `address`: 20-byte execution-layer recipient address;
- `amount`: nonzero canonical `uint64` amount in Gwei.

This decoder rejects zero amounts at decode time. That is a deliberate
field-domain admission rule: `WithdrawalAmountGwei` represents an EIP-4895
withdrawal amount, and EIP-4895 defines the amount as nonzero. This is still
not a full execution-layer block-validity claim. A future fork, test network,
or diagnostic tool that needs to inspect zero-amount artifacts should keep the
raw RLP bytes or use a fork-specific decoder instead of treating
`UnvalidatedWithdrawals` as a universal payload container.

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
