# Advanced Precompile Backend Admission

Status: `v0.52.0` fixes the EIP-4844 KZG and EIP-2537 BLS12-381 planning
contracts and assigns every remaining execution path to a concrete first-party
release.

This document is the release-blocking admission policy for cryptographic
precompiles at addresses `0x0a..=0x11`. These precompiles process public EVM
calldata, but malformed inputs, incorrect subgroup handling, gas mistakes, and
backend drift are consensus and denial-of-service risks.

## Decision

The production path will be first-party, dependency-free, `no_std`, and
allocation-free within explicit input limits. An external implementation may
be used as a test oracle or optional accelerator only after separate dependency
review; it cannot become the sole implementation behind a native support
claim.

`v0.52.0` implements frame and gas planning, not curve or proof execution.
Every descriptor continues to report `RequiresCryptoBackend` and execution
must fail closed until its assigned release passes vectors, fuzzing, review,
and pentest.

| Domain | Current planning contract | First-party execution releases |
| --- | --- | --- |
| EIP-4844 KZG point evaluation | Exact 192-byte input, 64-byte output, fixed 50,000 gas | `v0.77.0..=v0.81.0` |
| EIP-2537 G1 add | Exact 256-byte input, 128-byte output, fixed 375 gas | `v0.52.1` and `v0.52.11` |
| EIP-2537 G1 MSM | Non-empty `160 * k` input, 128-byte output, official discount gas | `v0.52.1`, `v0.52.13`, and `v0.52.14` |
| EIP-2537 G2 add | Exact 512-byte input, 256-byte output, fixed 600 gas | `v0.52.1` and `v0.52.12` |
| EIP-2537 G2 MSM | Non-empty `288 * k` input, 256-byte output, official discount gas | `v0.52.1`, `v0.52.13`, and `v0.52.14` |
| EIP-2537 pairing | Non-empty `384 * k` input, 32-byte output, `37,700 + 32,600 * k` gas | `v0.52.12`, `v0.52.13`, `v0.52.16`, and `v0.52.17` |
| EIP-2537 map Fp to G1 | Exact 64-byte input, 128-byte output, fixed 5,500 gas | `v0.52.15` |
| EIP-2537 map Fp2 to G2 | Exact 128-byte input, 256-byte output, fixed 23,800 gas | `v0.52.15` |

## Shared Admission Checklist

Every native or optional cryptographic backend must satisfy all applicable
items before its descriptor changes from a fail-closed state:

- Pin the governing EIP and official execution-spec/test revision.
- Check exact algorithm identity and domain separation; similarly named
  algorithms are not interchangeable.
- Reject non-canonical fields, malformed points, invalid encodings, and
  forbidden empty inputs before arithmetic execution.
- Enforce all required on-curve and subgroup checks, including infinity rules.
- Charge or reserve gas before expensive validation or arithmetic is reached.
- Recompute content-dependent gas from the actual execution input and reject
  plans whose recorded cost no longer matches those bytes.
- Use checked arithmetic for frame counts, gas, offsets, and output lengths.
- Keep runtime and memory bounded by the public input and release limits.
- Include official positive and negative vectors plus an independent oracle.
- Fuzz parser, frame, gas, error, and output-buffer boundaries.
- Run dependency, license, advisory, MSRV, `no_std`, clippy, and package checks.
- Complete a release-specific pentest and remediate every blocking finding.

The same principles apply to already admitted hash, secp256k1, ModExp, BN254,
and BLAKE2F paths. Existing native implementations remain covered by their
release vectors. Caller-provided or optional backends remain subject to their
documented boundary checks and cannot weaken transaction or precompile rules.

## KZG Release-Blocking Vectors

The KZG point-evaluation implementation cannot be admitted until tests cover:

- official EIP-4844 valid proof fixtures;
- exact 192-byte framing and 64-byte return constants;
- commitment-to-versioned-hash agreement and mismatch rejection;
- canonical `z` and `y` values strictly below the BLS scalar-field modulus;
- malformed, non-canonical, infinity, and invalid proof encodings;
- valid and invalid proof results under the pinned trusted setup;
- trusted-setup identity, provenance, fingerprint, and load failure behavior;
- fixed gas charged before proof verification;
- differential results against an independent implementation;
- fuzzing of frames, fields, setup selection, and output-buffer behavior.

The production trusted setup must not be downloaded implicitly at runtime.
Its source, exact bytes or canonical representation, digest, embedding/loading
policy, and update procedure must be reviewable and reproducible.

## BLS12-381 Release-Blocking Vectors

The seven EIP-2537 implementations cannot be admitted until tests cover:

- exact fixed lengths and non-empty multiple-of-item requirements;
- all 64-byte Fp encodings, including the required zero high 16 bytes;
- field elements equal to or above the modulus;
- G1/G2 infinity encodings, curve membership, and malformed coordinates;
- subgroup checks for MSM inputs and every pairing input;
- the EIP rule that G1/G2 addition does not require subgroup membership;
- scalar values across zero, subgroup-order, and full 256-bit boundaries;
- one, two, 128, and more-than-128 MSM discount cases;
- empty, partial, and multi-item pairing cases;
- canonical 32-byte zero/one pairing outputs;
- official map-to-curve vectors and exceptional inputs;
- official positive and negative vectors from execution-spec-tests;
- differential results against at least one independent implementation;
- parser/gas fuzzing before arithmetic and arithmetic fuzzing once executable.

## Sanitization Contract

EVM precompile calldata, public curve points, commitments, proofs, and
intermediate verification values are public, so the precompile-only path does
not claim that clearing them provides secret erasure. It must still avoid
uninitialized data exposure and clear caller-visible output on documented
error paths where partial output would violate the API contract.

If a reusable BLS/KZG backend is later exposed to signing keys, secret scalars,
authorization credentials, or key-adjacent hardware state, that API must add a
separate secret-bearing type boundary. Software scratch state must implement
the project sanitization contract, use the optional
`eth-valkyoth-sanitization` bridge where appropriate, and document residual
copies and compiler/platform limitations. A precompile backend is not
automatically approved for secret-key operations.

## Verification Commands

Current planning-boundary checks:

```bash
cargo test -p eth-valkyoth-evm-core precompile
cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings
cargo clippy --manifest-path fuzz/Cargo.toml --bin advanced_precompile_plan -- -D warnings
cargo deny check
```

Once arithmetic lands, each release gate must add the relevant official-vector
runner, differential command, bounded fuzz target, and release-mode gas/CPU
evidence before changing any descriptor to native execution.

## Normative Sources

- EIP-4844, shard blob transactions and point-evaluation precompile:
  <https://eips.ethereum.org/EIPS/eip-4844>
- EIP-2537, BLS12-381 precompiles:
  <https://eips.ethereum.org/EIPS/eip-2537>
- Ethereum execution specifications and generated tests:
  <https://github.com/ethereum/execution-specs>
