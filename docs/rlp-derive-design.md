# RLP Derive Evaluation

Status: v0.16.1 design and non-exported prototype.

`eth-valkyoth-derive` does not expose public `RlpEncode` or `RlpDecode` macros
yet. The v0.16.1 release records the API decision and keeps the prototype
private to tests until the codec traits and transaction validation typestates
are stable enough for a public derive contract.

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

## Deferred Public Surface

The following remain intentionally deferred:

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
- output buffers are not mutated on encode errors;
- generated transaction code cannot bypass fork or sender validation.

The v0.16.1 prototype lives in derive-crate tests and enforces the first API
rules: declaration order, explicit skip/default reason, and rejection of
generics, enums, unions, and ambiguous field attributes.
