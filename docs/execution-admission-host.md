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
- rejects an empty typed payload;
- rejects typed domains without a first-party canonical decoder;
- retains the exact raw bytes and classified envelope.

`try_into_canonical` runs the complete transaction-type decoder for legacy,
EIP-2930, EIP-1559, EIP-4844, or EIP-7702 under the conserved session. A
failure returns the original candidate token and a stable error.

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

The host exposes separate powers:

- `StateView`: immutable snapshot identity, accounts, original storage, and
  current storage;
- `StateJournal`: reset, child checkpoints, commit/revert, and writes;
- `BlockEnvironment`: immutable admitted fork and block context;
- `AccessTracker`: transaction-global address and slot warmth;
- `CryptoProvider`: reviewed Keccak and recovery operations;
- `Inspector`: observation-only lifecycle events with no consensus decision;
- `TransactionArena`: destructively resettable memory and iterative frames.

`StateSnapshot` remains as a compatibility read interface. Its blanket
`StateView` implementation returns the same value for original and current
storage because it has no journal overlay.

`ExecutionHost` intentionally gives `AccessTracker` no child checkpoint.
EIP-2929 warmth therefore survives child failure and revert, while state
changes remain controlled by `StateJournal` checkpoints.

`ExecutionHost::begin_child` reports checkpoint creation, frame rejection, and
frame-rejection cleanup separately through `BeginChildError`. If frame
admission and journal rollback both fail, the error retains both
`HostCapabilityError` values and marks journal consistency as unknown. A
caller must abort rather than retry that transaction.

Inspector child-event depth is the count of active child frames. The first
nested frame reports one on entry, commit, and revert.

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
- frame-capacity rejection tests with successful and failed journal cleanup;
- iterative depth and memory-capacity failure tests;
- an execution-admission fuzz target with committed seeds;
- `no_std`, strict Clippy, workspace, MSRV, and release-gate coverage.
