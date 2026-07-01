# Fuzzing

Status: `v0.22.0` RLP, transaction envelope, legacy transaction decode,
EIP-2930 access-list transaction decode, EIP-1559 dynamic-fee transaction
decode, EIP-4844 blob transaction decode, and transaction encode fuzz/test
baseline, including signing-preimage encoding, plus Ethereum signature parsing
fuzz coverage. EIP-712 domain-safety coverage is currently unit-test based
because it does not parse untrusted bytes.

The fuzz workspace lives under `fuzz/` and is intentionally separate from the
published crates. Live corpus growth and crash artifacts are local generated
state and are ignored by git.

## Targets

| Target | Scope |
| --- | --- |
| `rlp` | Combined exact and partial RLP scalar, integer, list, and bounded integer decoders. |
| `rlp_scalar` | Scalar exact and partial decoders across test, deployment-template, and unbounded limits. |
| `rlp_integer` | Canonical integer payload helpers plus exact and partial integer decoders. |
| `rlp_list` | List exact and partial decoders plus recursive item traversal. |
| `rlp_encode` | Scalar, integer, list-payload, list-header, and decoded-item encode paths. |
| `primitives` | Primitive RLP bridge decoders and canonical integer payload constructors. |
| `transaction_envelope` | EIP-2718 typed envelope classification, legacy RLP-list shell classification, unvalidated legacy/EIP-2930/EIP-1559/EIP-4844 field decoding, and fixed-buffer canonical re-encoding for successfully decoded transaction models. |
| `ethereum_signature` | Ethereum `r || s || y_parity` signature parsing and digest-level sender recovery with a deterministic stub hasher. |
| `decode_limits` | Stateless and accumulator decode-budget APIs. |

Every new parser that accepts untrusted bytes must either extend one of these
targets or add a new target in the same release.

List-recursion fuzz targets drive item iteration to
`MAX_RLP_LIST_TRAVERSAL_DEPTH`, matching the decoder hard cap. The committed
seed corpus includes a 128-level empty-list chain for this path.
The transaction-envelope fuzz target also drives `decode_legacy_transaction`,
`decode_access_list_transaction`, `decode_dynamic_fee_transaction`,
`decode_blob_transaction`, and the matching fixed-buffer encoders when decoding
succeeds, then applies the same recursion limit when it sees a legacy RLP-list
envelope.

## Seed Corpus

Committed seeds live in `fuzz/seed-corpus/<target>/*.hex`. They are hex text so
reviews can see which RLP fixture or adversarial case changed.

Validate committed seeds without writing local corpus files:

```bash
scripts/materialize_fuzz_seeds.py --check
```

Materialize them into `fuzz/corpus/` before running `cargo fuzz`:

```bash
scripts/materialize_fuzz_seeds.py
```

## Running

Install `cargo-fuzz`, then run a target from the repository root:

```bash
cargo fuzz run rlp
cargo fuzz run rlp_integer
```

The release gate only requires that the fuzz workspace builds:

```bash
cargo check --manifest-path fuzz/Cargo.toml
```

Long-running fuzz campaigns are expected before parser-heavy releases, but they
are not run inside normal CI.

## Crash Reproduction

When libFuzzer reports a crash:

1. Keep the artifact under `fuzz/artifacts/<target>/`.
2. Reproduce it locally:

```bash
cargo fuzz run <target> fuzz/artifacts/<target>/<artifact>
```

3. Minimize the case if it is large:

```bash
cargo fuzz tmin <target> fuzz/artifacts/<target>/<artifact>
```

4. Add a deterministic unit or integration test before fixing the parser.
5. Keep the minimized artifact out of git unless it is converted into a reviewed
   hex seed under `fuzz/seed-corpus/<target>/`.
6. Document the finding and retest result in the release pentest report.
