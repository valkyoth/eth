# EVM Execution Environment Boundary

Status: `v0.38.0` implementation ready; awaiting pentest before tagging.

This document records the first explicit execution boundary for
`eth-valkyoth-evm`. The crate still does not admit REVM or any other concrete
execution backend. This release defines the input and report contract that
future backends must consume.

## Boundary Shape

An execution request is made from three explicit inputs:

- `ExecutionEnvironment`: active fork validation context plus matching block
  execution context;
- `ExecutionTransaction`: raw transaction bytes bound to a decoded
  EIP-2718/legacy envelope shell through the bounded protocol decoder;
- `StateSnapshot`: caller-provided account and storage view with a stable
  snapshot identifier.

The environment constructor rejects:

- inactive fork contexts;
- fork/block chain ID mismatch;
- fork/block number mismatch;
- fork/block timestamp mismatch.

The transaction constructor uses `decode_transaction_envelope`, so the raw
bytes and envelope shell are derived together under explicit `DecodeLimits`.

## State Snapshot Contract

`StateSnapshot` is intentionally narrow and no-alloc:

- `snapshot_id()` returns the caller-reviewed state identity;
- `account(address)` returns account nonce, balance, and code hash;
- `storage(address, slot)` returns one storage slot value.

The trait does not prescribe storage, caching, database, RPC, proof, or witness
formats. Those are future layers. The first requirement is that any execution
attempt can report which state identity was used.

## Result Model

`ExecutionReport` binds:

- the exact `ExecutionEnvironment`;
- the transaction type domain;
- the state snapshot ID.

`ExecutionResult` is present as the future backend result envelope. It carries
status, gas used, and the report. No function currently performs EVM execution;
`ExecutionError::BackendUnavailable` records that a backend is not admitted by
this crate version.

## Security Notes

- REVM remains rejected by the existing dependency review and runtime
  dependency policy.
- The boundary is `no_std` and uses only first-party workspace crates.
- Later gas estimation and execution backends must accept this boundary rather
  than inventing parallel fork, block, transaction, or snapshot inputs.
