# Truncated PUSH Semantics

Status: implemented in `v0.52.2`; full opcode and state-transition conformance
remains assigned to later releases.

## Consensus Rule

For `PUSH1` through `PUSH32`, bytes missing because code ends before the
declared immediate width are read as zero bytes on the right. The program
counter still advances by the opcode byte plus the full declared immediate
width. A truncated PUSH is therefore valid EVM bytecode and is not an
exceptional halt.

Execution and jump-destination analysis use the same instruction-advance
helper. Bytes physically present after a PUSH opcode are data up to the
declared width and cannot become `JUMPDEST` targets, even when the immediate is
truncated by code EOF.

## Conformance Sources

The implementation is checked against three independent descriptions:

- the Ethereum Yellow Paper instruction-width function and implicit `STOP`
  bytes after the end of code;
- the pinned `execution-specs` `buffer_read` instruction implementation,
  which returns zero for bytes beyond the buffer and advances by the complete
  PUSH width;
- Geth's `opPush1`, `opPush2`, and generated `makePush` handlers, which
  left-shift available bytes when the immediate is short, producing the same
  right-zero-padded word.

Exact Ethereum source revisions are locked in `spec-lock.toml` and materialized
by `scripts/sync_spec_sources.py`. Upstream drift is checked separately by
`scripts/check_ethereum_upstream.py`; an upstream update is not silently
treated as reviewed consensus behavior.

## Verification Boundary

The exhaustive test covers every declared PUSH width and every possible
truncated payload length, for 528 truncated forms. It checks the resulting
word, gas, step count, and declared program-counter advance. Separate tests
check that jump analysis skips every available immediate byte and rejects
jumps into those bytes.

The `truncated_push` fuzz target compares execution with an independent padded
word construction and preserves the jump-destination invariant. Its committed
seed corpus includes an empty `PUSH32` and a partially present `PUSH2`.

This release corrects only PUSH-at-EOF semantics. It does not expand the
interpreter's admitted opcode set or claim complete EVM conformance.
