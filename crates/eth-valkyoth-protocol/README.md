# eth-valkyoth-protocol

Support crate for `eth`: fork-aware `no_std` Ethereum protocol validation
state and transaction envelope shell classification.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.24"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.22.0` support-crate release, shipped with `eth` `0.24.2`, adds the
EIP-7702 set-code transaction validity gate. It checks Prague/Pectra fork
context, non-empty authorization lists, authorization chain/nonce policy,
fee order, caller-computed gas policy, and caller-provided authority account
state without bundling a node or RPC dependency.

The previous `0.21.0` release added no-allocation EIP-7702 set-code
transaction and authorization signing-preimage helpers. The transaction
preimage uses type byte `0x04`; authorization tuple preimages use the EIP-7702
authorization magic byte `0x05` over `rlp([chain_id, address, nonce])`.

The previous `0.20.0` release added unvalidated EIP-7702 set-code transaction
decoding and encoding for type byte `0x04`. It decodes the required
destination address plus authorization tuples shaped as
`[chain_id, address, nonce, y_parity, r, s]`, then re-encodes the borrowed
model without allocation.

Earlier releases added proof-gated transaction typestate transitions for
decoded, canonical, fork-validated, and sender-recovered state tokens. The
proof token fields remain private, so external callers cannot fabricate
validation state before the real validators land. Successful promotion consumes
the previous state token; failed promotion returns the original token with the
validation error.

The crate also provides caller-reviewed `ChainSpec`, `ForkSpec`, `Hardfork`,
and `ValidationContext` types for explicit fork activation context. Use
`ChainSpec::new` only for hand-audited static tables; use `ChainSpec::try_new`
for dynamic, generated, or merged fork entries. Selection APIs reject
wrong-chain entries, duplicate hardforks, and non-monotonic hardfork or
activation ordering before returning a fork context.

This crate retains the earlier EIP-2718 typed envelope classification and
unvalidated transaction models for legacy, EIP-2930 access-list, EIP-1559
dynamic-fee, EIP-4844 blob, and EIP-7702 set-code transactions. It does not
validate signatures, recover senders, enforce transaction chain binding,
account for gas or blob gas, verify KZG commitments/proofs, validate set-code
authorization signatures itself, apply duplicate access-list policy, or execute
transactions.
