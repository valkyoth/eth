# Transaction Signing Hashes

Status: v0.24.1 adds EIP-7702 set-code transaction and authorization signing
hashes.

This release adds canonical signing-preimage encoders for the decoded
transaction families currently admitted by `eth-valkyoth-protocol`, plus
Keccak-256 signing-hash helpers in `eth-valkyoth-verify`.

The crate still does not admit a default Keccak backend. Callers must provide a
hasher that implements `eth_valkyoth_hash::Keccak256` and should verify it with
`KECCAK256_EMPTY` before use.

## Preimage Domains

| Transaction family | Signing preimage |
| --- | --- |
| Legacy EIP-155 | `rlp([nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0])` |
| EIP-2930 | `0x01 || rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList])` |
| EIP-1559 | `0x02 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList])` |
| EIP-4844 | `0x03 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, maxFeePerBlobGas, blobVersionedHashes])` |
| EIP-7702 transaction | `0x04 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, destination, value, data, accessList, authorizationList])` |
| EIP-7702 authorization | `0x05 || rlp([chain_id, address, nonce])` |

The EIP-7702 authorization domain is not a transaction signing hash. It uses
`SetCodeAuthorizationSigningHash` so callers cannot pass it where a
`TransactionSigningHash` is required.

The typed transaction preimages intentionally exclude `y_parity`, `r`, and
`s`. Legacy EIP-155 preimages replace those fields with `chainId`, `0`, and
`0`.

## API Split

`eth-valkyoth-protocol` exposes no-allocation preimage encoders and length
helpers. The caller supplies the output buffer.

`eth-valkyoth-verify` exposes signing-hash helpers that:

- encode the canonical preimage into caller-provided scratch space;
- hash only the bytes written by the encoder;
- return `TransactionSigningHash`, a domain newtype around `B256`;
- return `SetCodeAuthorizationSigningHash` for authorization tuples;
- reject pre-EIP-155 legacy transactions with
  `TransactionSigningHashError::MissingReplayDomain`.

Callers that reuse `scratch` across multiple in-flight or not-yet-broadcast
transactions should zero it after hashing before reusing or releasing the
buffer. Transaction preimages can contain calldata or business-sensitive
payloads even though they are normally public after broadcast.

## Deferred Validation

For full decoded transaction signature validation, use the v0.24.1
`validate_transaction_signature` helpers so replay-domain checks, low-s/y-parity
policy, sender recovery, and optional expected-sender comparison are applied
together.

The helpers also do not perform fee, account-state, fork-validity,
intrinsic-gas, blob-count, blob-version, KZG, data-availability, or EIP-7702
authorization chain/nonce/account-state checks.
