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
- Request-bound private `StateJournal`, `AccessTracker`, `CryptoProvider`, and
  `TransactionArena` capabilities with post-transition `InspectorEvent`
  evidence.
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
- `StateJournal` is associated with its exact immutable view and owns all
  current-storage reads.
- Child checkpoint tokens are replaced by closure-scoped `with_child`
  execution with exact-depth/LIFO checks and fail-closed RAII host poisoning,
  armed before checkpoint creation and covering every journal/arena call plus
  child execution before finalization.
- Transaction reset is pessimistically poisoned until journal, access, and
  arena resets all complete; every direct mutable root capability is guarded
  against backend error and unwind.
- The live tooling freshness gate now audits Action refs across every workflow
  through semantic YAML traversal, including flow mappings and reusable
  workflows, and verifies every checkout use against the latest tag and exact
  upstream commit using the parsed value rather than comments; it also checks
  the active cargo-fuzz CLI release.
- Checkout identity must use canonical lowercase spelling, while Docker
  actions, job containers, and service containers require immutable SHA-256
  image digests.

## Security Properties

- Outer EIP-2718 classification alone cannot authorize execution.
- Classification and type-specific canonical decoding conserve one
  non-copyable decode session, including both legacy parser passes.
- Unknown typed domains fail closed before execution admission.
- Fork admission binds the transaction type and explicit signed chain to the
  selected execution environment.
- Journal rollback cannot cool transaction-global EIP-2929 accesses.
- Host recursion and unbounded transaction-memory growth are not admitted.
- Inspectors receive immutable lifecycle events only after critical
  transitions complete and cannot return consensus decisions.
- Inspector depth is documented as a one-based active child-frame count.
- A frame-capacity failure cannot shadow a simultaneous journal cleanup
  failure; callers receive a distinct fatal consistency error with both
  causes.
- Host state and environment cannot diverge from `ExecutionRequest` report
  provenance, and stale compatibility-snapshot storage cannot override a
  journal write.
- Partial journal/arena finalization or panic unwinding poisons the host and
  blocks continued execution.

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

The first independent review reported two Low findings, both remediated. A
follow-up review reported three High and two Medium findings covering child
lifecycle atomicity, request provenance, stale current storage, classification
accounting, and inspector control flow. A second follow-up reported two Medium
findings covering unwind poisoning and flow-style YAML Action-pin bypasses. A
third follow-up reported two Medium findings covering guard arming before
backend calls and comment-based checkout freshness. All findings are
remediated. A fourth follow-up reported three Medium findings covering
transaction/root-mutation unwinding, checkout casing, and mutable container
references; all three are remediated. Tagging remains blocked until a clean
retest and permanent exact-commit report at `security/pentest/v0.52.7.md`.
