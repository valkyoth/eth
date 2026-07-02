# Transaction Signature Validation

Status: v0.24.2 keeps set-code transaction signature validation and
authorization tuple signature validation separate from the protocol validity
gate.

`eth-valkyoth-verify` now exposes decoded transaction signature validation
helpers for the transaction families decoded by `eth-valkyoth-protocol`:

- legacy EIP-155 transactions;
- EIP-2930 access-list transactions;
- EIP-1559 dynamic-fee transactions;
- EIP-4844 blob transactions;
- EIP-7702 set-code transactions.

EIP-7702 authorization tuple signatures are validated through
`validate_set_code_authorization_signature`, not through the transaction
signature helper. The authorization signing hash uses
`SetCodeAuthorizationSigningHash` to keep the `0x05` authorization domain
separate from the `0x04` transaction sender domain.

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
blob count, blob-hash version bytes, KZG commitments, EIP-7702 authorization
chain policy, authorization nonce/account-state policy, delegation indicator
state, or protocol typestate promotion. They return a
`ValidatedTransactionSignature`, which records only the recovered sender and the
signing hash that was recovered against. That proof value is intentionally not
publicly constructible; callers must obtain it through the validation helpers so
sender-recovered state cannot be forged outside `eth-valkyoth-verify`.

The test suite includes external raw mainnet transaction known-answer tests for
EIP-2930, EIP-1559, EIP-4844, and EIP-7702. Those fixtures were sourced through
`eth_getRawTransactionByHash` from `ethereum.publicnode.com` or
`ethereum-rpc.publicnode.com` and assert the RPC `from` sender against the
crate's independent decode, signing-hash, and recovery path. The EIP-7702
fixture also validates its embedded authorization tuple through the separate
`0x05` authorization domain.

Protocol typestate promotion remains intentionally gated until public proof
constructors can bind proofs to transaction identity instead of allowing
callers to manufacture sender-recovered state tokens directly.
