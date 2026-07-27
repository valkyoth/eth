# Release Notes - eth v0.52.7

Status: pentest findings remediated; awaiting clean retest.

## Summary

This release replaces opaque transaction-shell execution admission with
non-forgeable typestates and separates EVM host powers into explicit,
auditable capabilities.

## Added

- `ClassifiedEnvelope`, `CanonicallyDecodedTransaction`,
  `ForkValidatedTransaction`, and `ExecutionReadyTransaction`.
- Canonical in-session decoding for legacy, EIP-2930, EIP-1559, EIP-4844, and
  EIP-7702 execution admission.
- Fail-closed rejection for empty or unsupported typed envelopes.
- Transaction-type activation and signed-chain checks before execution-ready
  promotion.
- `StateView`, `StateJournal`, `BlockEnvironment`, `AccessTracker`,
  `CryptoProvider`, optional `Inspector`, and `TransactionArena`.
- Allocation-free `BorrowedTransactionArena` with destructive reset,
  iterative frames, and explicit memory/depth failures.
- Compile-fail, fork matrix, rollback/warmth, arena-bound, and fuzz coverage.
- `BeginChildError` distinguishes checkpoint failure, frame rejection after
  successful cleanup, and simultaneous frame rejection plus journal rollback
  failure while retaining both underlying errors.

## Changed

- `ExecutionRequest::new` now accepts only an
  `ExecutionReadyTransaction`; the old decoded-envelope-shell constructor is
  removed.
- `StateSnapshot` remains compatible through a blanket `StateView`
  implementation.
- The live tooling freshness gate now audits Action refs across every workflow
  and verifies every checkout use against the latest tag and exact upstream
  commit; it also checks the active cargo-fuzz CLI release.

## Security Properties

- Outer EIP-2718 classification alone cannot authorize execution.
- Type-specific canonical decoding conserves one non-copyable decode session.
- Unknown typed domains fail closed before execution admission.
- Fork admission binds the transaction type and explicit signed chain to the
  selected execution environment.
- Journal rollback cannot cool transaction-global EIP-2929 accesses.
- Host recursion and unbounded transaction-memory growth are not admitted.
- Inspectors observe immutable lifecycle events and cannot return consensus
  decisions.
- Inspector depth is documented as a one-based active child-frame count.
- A frame-capacity failure cannot shadow a simultaneous journal cleanup
  failure; callers receive a distinct fatal consistency error with both
  causes.

The fork-validation typestate is intentionally limited. Sender recovery,
intrinsic gas, nonce/account state, balances, fees, blob/KZG rules, EIP-7702
authority rules, and complete consensus validity remain separate required
gates. Because the current shared hardfork domain does not model Berlin,
EIP-2930 admission fails closed before London until the fork-domain redesign
assigned to `v0.63.0`.

## Versioning

- `eth-valkyoth-evm` advances from `0.10.2` to `0.11.0`.
- `eth` advances from `0.52.6` to `0.52.7`.
- All other support crates remain unchanged.

## Pentest

The initial independent review reported two Low findings: child-frame rejection
could be shadowed by a rollback failure, and inspector depth documentation
incorrectly called a one-based count zero-based. Both are remediated. Tagging
remains blocked until a clean retest and permanent exact-commit report at
`security/pentest/v0.52.7.md`.
