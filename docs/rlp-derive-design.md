# RLP Derive Evaluation

Status: v0.25.0 public conservative derive surface.

`eth-valkyoth-derive` exposes public `RlpEncode` and `RlpDecode` macros for
reviewed simple structs. The public surface remains intentionally conservative:
generics, enums, unions, transaction derives, and implicit skipped fields are
still rejected.

## Decision

RLP derives must generate code against first-party codec traits, not directly
against ad hoc byte manipulation. The eventual public shape is:

```rust,ignore
#[derive(RlpEncode, RlpDecode)]
#[eth_rlp(crate = "::eth_valkyoth_codec")]
struct Example {
    chain_id: ChainId,
    nonce: Nonce,
}
```

`RlpEncode` must write into caller-provided buffers and return exact byte counts.
`RlpDecode` must take an explicit `DecodeLimits` value, use the same canonical
RLP helpers as hand-written code, and return domain-specific errors rather than
silently accepting malformed input.

## Field Order

Fields are encoded and decoded in Rust declaration order. The derive must not
sort by name, accept duplicate aliases, or infer Ethereum transaction field
order. Ethereum transaction structs should continue to use hand-written
encoders until fork-aware validation typestates are available.

## Skip Policy

Skipping a field is decode-sensitive and must be explicit:

```rust,ignore
#[eth_rlp(skip, default, reason = "derived cache")]
cached_hash: B256,
```

`skip` without `default`, `default` without `skip`, and skip/default without a
non-empty `reason` are rejected in the prototype. This keeps cached or derived
fields visible in review and prevents accidental consensus-field omission.

The private prototype carries the validated reason into its field plan instead
of discarding it after parsing. When public code generation lands, the reason
must remain visible in generated output or diagnostics so review evidence is not
lost during macro expansion.

Repeated `#[eth_rlp(...)]` attributes on the same field are rejected. Attribute
merging or last-write-wins behavior is not allowed for consensus-relevant field
metadata.

## Deferred Public Surface

The first public RLP derive surface shipped in `v0.25.0`. The following remain
intentionally deferred:

- generic structs, until generated trait bounds and borrowed lifetimes are
  specified;
- enums and unions, because RLP field counts and Ethereum domains need explicit
  typestate decisions;
- transaction derives, until decoded, fork-valid, and signer-recovered states
  are separate types;
- default field values other than `Default::default`, until a safe attribute
  grammar exists.

## Security Requirements

Public derives must preserve these invariants:

- decode paths require `DecodeLimits` and cumulative budget accounting;
- integer fields delegate to canonical codec and primitive helpers;
- fixed-width fields reject adjacent scalar lengths;
- direct field encoders must not mutate buffers when they reject before
  writing;
- aggregate derived encoders may leave a partial prefix on later field failure,
  so callers must discard output buffers after any encode error;
- generated transaction code cannot bypass fork or sender validation.

The v0.25.0 implementation includes trybuild compile-fail coverage for
unsupported generics, enums, and ambiguous skipped fields, plus round-trip tests
for named, tuple, and unit structs.
