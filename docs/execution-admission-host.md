# Execution Admission And Host Capabilities

Status: `v0.52.7` pentest findings remediated; awaiting clean retest of the
exact remediation commit.

This document defines the transaction and host boundary that every later
first-party execution machine must consume. It does not claim complete
transaction validity or complete EVM execution.

## Admission Typestates

Execution admission is a one-way promotion:

```text
raw bytes
  -> ClassifiedEnvelope
  -> CanonicallyDecodedTransaction
  -> ForkValidatedTransaction
  -> ExecutionReadyTransaction
  -> ExecutionRequest
```

The tokens have private fields and are not `Clone` or `Copy`.
`ExecutionRequest::new` accepts only `ExecutionReadyTransaction`, so an outer
EIP-2718 classification result cannot be confused with an executable
transaction.

`ClassifiedEnvelope::decode`:

- constructs one non-copyable `DecodeSession`;
- classifies legacy RLP through that same session rather than an unaccounted
  compatibility parser;
- rejects an empty typed payload;
- rejects typed domains without a first-party canonical decoder;
- retains the exact raw bytes and classified envelope.

`try_into_canonical` runs the complete transaction-type decoder for legacy,
EIP-2930, EIP-1559, EIP-4844, or EIP-7702 under the conserved session. Legacy
classification and its required canonical reparse are both charged, so
deployment policy must budget both passes. A failure returns the original
candidate token and a stable error.

`try_into_fork_validated` checks:

- the transaction type is active for the selected modeled hardfork;
- every explicit signed chain ID matches the execution environment.

The current shared `Hardfork` domain does not model Berlin separately.
EIP-2930 admission therefore fails closed before London rather than claiming
historical Berlin support. The complete fork-domain redesign remains assigned
to `v0.63.0`.

This name is deliberately narrower than complete transaction validity. The
stage does not prove sender recovery, intrinsic gas, nonce/account state,
balance, fee ordering, blob/KZG validity, EIP-7702 authority validity, or any
other state-dependent consensus rule. Later validity gates remain mandatory
before state transition execution.

## Host Capability Split

The host owns separate private powers:

- `StateView`: immutable snapshot identity, accounts, and original storage;
- `StateJournal`: an associated exact `StateView`, authoritative current
  storage, reset, private child checkpoints, commit/revert, and writes;
- `AccessTracker`: transaction-global address and slot warmth;
- `CryptoProvider`: reviewed Keccak and recovery operations;
- `Inspector`: external observation of post-transition immutable events;
- `TransactionArena`: destructively resettable memory and iterative frames.

`ExecutionHost::new` requires an `ExecutionRequest` and derives its state and
environment exclusively from that admitted request. Capability fields are
private, and the journal's associated view type must match the request view.
`StateSnapshot` remains as a compatibility read interface for immutable base
state, while current storage can only be read through `StateJournal`.

`ExecutionHost` intentionally gives `AccessTracker` no child checkpoint.
EIP-2929 warmth therefore survives child failure and revert, while state
changes remain controlled by `StateJournal` checkpoints.

`ExecutionHost::with_child` keeps checkpoint tokens private and scopes child
execution in a closure, so nested children must finalize in LIFO order. It
validates the exact frame depth before consuming the checkpoint and poisons
the host after any partial or inconsistent journal/arena finalization. If
frame admission and journal rollback both fail, `BeginChildError` retains both
errors. A poisoned host rejects all later mutable capability operations.

Transaction and child methods return immutable `InspectorEvent` evidence only
after critical transitions complete. Inspectors are not invoked while a
checkpoint or frame is in an externally unrecoverable intermediate state.
Child-event depth is the count of active child frames; the first nested frame
reports one on entry, commit, and revert.

## Bounds And Failure

`BorrowedTransactionArena`:

- accepts caller-owned memory and a compile-time frame capacity;
- clears memory during construction and transaction reset;
- rejects more than 1,024 iterative frames;
- rejects memory capacity above 16 MiB;
- turns depth and memory expansion failures into explicit stable errors;
- uses no host recursion or allocation.

The limits are release ceilings, not recommendations for every deployment.
Operators should select smaller reviewed capacities where appropriate.

## Verification

The release includes:

- compile-fail coverage proving `ClassifiedEnvelope` cannot construct an
  `ExecutionRequest`;
- canonical transaction fixtures for all five admitted transaction domains;
- fork/type activation and signed-chain mismatch matrices;
- empty and unknown typed-envelope rejection;
- a nested child-revert test proving state rolls back while warmth survives;
- nested child finalization proving journal checkpoints complete in LIFO order;
- journal-authoritative current-storage coverage after a write;
- poisoned-host coverage after partial journal finalization;
- frame-capacity rejection tests with successful and failed journal cleanup;
- post-transition inspector dispatch coverage;
- legacy classification/reparse accounting under one decode ledger;
- iterative depth and memory-capacity failure tests;
- an execution-admission fuzz target with committed seeds;
- `no_std`, strict Clippy, workspace, MSRV, and release-gate coverage.
