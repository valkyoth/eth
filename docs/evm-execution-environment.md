# EVM Execution Environment Boundary

Status: introduced in `v0.38.0`, extended by bounded gas estimation in
`v0.39.0`, and superseded for transaction admission by the `v0.52.7`
typestate boundary.

This document records the durable environment, request, report, and
gas-estimation contracts. See
[Execution Admission And Host Capabilities](execution-admission-host.md) for
the current transaction and host boundary.

## Boundary Shape

An execution request binds:

- `ExecutionEnvironment`: active fork validation context plus matching block
  execution context;
- `ExecutionReadyTransaction`: exact bytes that passed classification,
  type-specific canonical decoding, transaction-type activation, and signed
  chain binding;
- `StateView`: caller-provided immutable account and original/current storage
  view with a stable snapshot identifier.

The environment constructor rejects inactive fork contexts and mismatched
chain ID, block number, or timestamp. `ExecutionRequest::new` cannot accept an
opaque `TransactionEnvelope` or `ClassifiedEnvelope`.

## State Contract

`StateView` is intentionally narrow and no-alloc:

- `snapshot_id()` returns the caller-reviewed state identity;
- `account(address)` returns account nonce, balance, and code hash;
- `original_storage(address, slot)` returns the transaction-start value;
- `current_storage(address, slot)` returns the current read value.

`StateSnapshot` remains as a compatibility trait. Its blanket `StateView`
implementation uses `storage` for both original and current values because it
has no journal overlay.

The traits do not prescribe databases, caches, RPC, proofs, or witness
formats. Every execution attempt can still report which state identity it
used.

## Result Model

`ExecutionReport` binds:

- the exact `ExecutionEnvironment`;
- the transaction type domain;
- the caller-computed Keccak-256 hash of the exact raw transaction bytes;
- the state snapshot ID.

The EVM boundary does not compute Keccak-256 here. Callers pass a transaction
hash produced by their reviewed hash backend. `ExecutionError::BackendUnavailable`
records that this crate version does not yet expose complete EVM execution.

## Gas Estimation

`GasEstimationPolicy` requires:

- maximum backend attempts from `1` through
  `MAX_GAS_ESTIMATION_ATTEMPTS`;
- a gas cap from `1` through `MAX_GAS_ESTIMATION_GAS_CAP`;
- a deterministic backend-step or caller-enforced timeout guard.

`GasEstimationRequest::try_new` binds that policy to an `ExecutionRequest` and
rejects a gas cap above the selected block gas limit. Reports reject attempt
counts above policy and estimates above the selected cap.

## Security Notes

- Shell-level EIP-2718 classification is never execution authority.
- Transaction admission uses a single conserved bounded decode session.
- The active environment, transaction type, transaction hash, and snapshot
  identity remain explicit audit evidence.
- Host state, journal, access, cryptographic, inspection, and arena powers are
  separate contracts.
- Complete transaction validity and complete execution remain assigned to
  later versioned milestones.
