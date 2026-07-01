# Transaction Signature Validation

Status: v0.23.0 pentest remediation ready for retest.

`eth-valkyoth-verify` now exposes decoded transaction signature validation
helpers for the transaction families decoded by `eth-valkyoth-protocol`:

- legacy EIP-155 transactions;
- EIP-2930 access-list transactions;
- EIP-1559 dynamic-fee transactions;
- EIP-4844 blob transactions.

The helpers combine the pieces that were intentionally separate in earlier
releases:

- replay-domain checking against an expected `ChainId`;
- canonical transaction signing-hash construction;
- low-s and scalar checks through the secp256k1 recovery path;
- typed y-parity policy;
- sender recovery from the transaction signing hash;
- optional recovered-sender comparison when the caller has an expected sender.

Callers still provide two Keccak-256 implementations: one for the transaction
signing preimage and one for hashing the recovered public key into an Ethereum
address. Implementations used on key-adjacent paths should clear internal
sponge state on drop; the optional sanitization bridge is the preferred place
to enforce that for concrete hashers.

These helpers do not prove full Ethereum execution validity. They do not check
fork activation, intrinsic gas, fee ordering, account nonce/state, balance,
blob count, blob-hash version bytes, KZG commitments, or protocol typestate
promotion. They return a `ValidatedTransactionSignature`, which records only
the recovered sender and the signing hash that was recovered against. That
proof value is intentionally not publicly constructible; callers must obtain it
through the validation helpers so sender-recovered state cannot be forged
outside `eth-valkyoth-verify`.

The test suite includes external raw mainnet transaction known-answer tests for
EIP-2930, EIP-1559, and EIP-4844. Those fixtures were sourced through
`eth_getRawTransactionByHash` from `ethereum.publicnode.com` and assert the RPC
`from` sender against the crate's independent decode, signing-hash, and
recovery path.

Protocol typestate promotion remains intentionally gated until public proof
constructors can bind proofs to transaction identity instead of allowing
callers to manufacture sender-recovered state tokens directly.
