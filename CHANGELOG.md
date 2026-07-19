# Changelog

All notable changes to `eth` are documented here.

## Unreleased

- Corrected `PUSH1..=PUSH32` execution at code EOF to right-zero-pad missing
  immediate bytes, and made jump-destination analysis use the same declared
  instruction width.
- Added exhaustive coverage for all 528 truncated PUSH forms, independent
  client/specification vectors, jump-analysis invariants, a fuzz target, and
  committed seeds.
- Remediated the v0.52.2 pentest by retaining the hidden legacy truncated-PUSH
  error variant and stable code for facade patch-release compatibility while
  keeping it unreachable from production execution.
- Corrected the v0.52.2 support-crate release to `eth-valkyoth-evm-core 0.26.1`
  so facade patch updates preserve public dependency type identity.
- Updated the pinned stable toolchain and full release gate to Rust `1.97.1`,
  extended compatibility checks through Rust `1.97.0`, and made the networked
  release tooling fail when the pin is not the latest official stable Rust.
- Remediated the v0.52.1 pentest by zero-initializing EVM memory, making
  execution contexts one-shot until destructive reset, rolling back warm/cold
  access tracking on failed or reverted stateful runs, validating EIP-712
  identifiers and uniqueness, reserving atomic-looking type names, bounding
  borrowed field/value/array collections and cumulative recursive value work,
  validating fully unwrapped member types even for empty arrays, bounding
  cumulative dynamic hashing with default and caller-selected limits,
  validating schemas once, caching recursive type hashes, and clearing partial
  typed-data output on failure.
- Reworked the facade README opening around a direct crate purpose, early
  install and transaction-envelope example, compact capability status table,
  and a dedicated detailed current-status document.
- Updated the pinned stable toolchain and full release gate to Rust `1.97.0`,
  while retaining all-feature compatibility checks for Rust `1.90.0` through
  `1.96.1`.
- Started `0.52.1` with dependency-free canonical EIP-2537 Fp, Fr, Fp2,
  unrestricted MSM scalar, G1/G2 coordinate, infinity, and complete
  `0x0b..=0x11` precompile-frame parsing.
- Added exact field-modulus, padding, coefficient-order, frame-boundary,
  round-trip, iterator, and malformed-late-item tests plus the
  `bls12381_wire` fuzz target and committed seed corpus.
- Kept curve membership, subgroup checks, mapping, arithmetic, MSM, pairing,
  and all BLS precompile execution fail closed for the assigned later releases.
- Started `0.52.0` with exact EIP-4844 KZG and EIP-2537 BLS12-381 frame,
  output-length, and gas planning while advanced cryptographic execution stays
  fail closed.
- Added official BLS MSM discount-table gas, non-empty variable-frame policy,
  boundary tests, advanced-precompile fuzz coverage, and a concrete
  first-party BLS/KZG release sequence.
- Refreshed optional `k256` to `0.14.0`, optional `sanitization` to `1.2.4`,
  `trybuild` to `1.0.118`, and CI `cargo-deny` to `0.20.2`.
- Remediated the v0.52.0 pentest by correcting `SUB`/`LT`/`GT` stack operand
  order, requiring gas meters for every public precompile executor, removing
  raw unmetered execution exports, and enforcing semantic committed-SBOM drift
  checks in CI and release readiness.
- Remediated the follow-up v0.52.0 review by ignoring unused false-`JUMPI` and
  zero-length memory offsets, rebinding every precompile plan to actual-input
  gas, bounding EIP-712 dependency traversal to 64 types with one visit per
  reachable type, and redacting non-copyable EIP-712 signing values.
- Closed the remaining v0.52.0 retest finding by making `EXTCODECOPY` ignore
  offsets for empty copies and zero-fill code offsets outside the bounded code
  domain without host-width conversion or host invocation.
- Corrected the v0.52.0 verify-crate release from `0.21.1` to `0.22.0` so its
  EIP-712 public API changes cannot enter downstream `^0.21` builds.
- Started `0.37.2` with `docs/core-independence-audit.md`, classifying
  default, optional, dev-only, reference-only, and compile-time dependencies
  that can influence core Ethereum behavior.
- Scheduled `0.37.4` and `0.37.5` so `subtle`, `alloy-rlp`, dev
  `serde_json`, optional `serde`/`serde_json`, and optional `sanitization`
  have explicit follow-up releases before execution work continues.
- Addressed v0.37.2 pentest findings by removing a duplicate release-gate test
  invocation, normalizing dependency-audit taxonomy labels, and checking the
  upstream user-agent release string in metadata validation.
- Started `0.37.1` with a safe upstream advisory checker for latest REVM
  registry metadata, Rust `1.90.0`-compatible REVM lines, and official
  Ethereum source drift.
- Added `docs/ethereum-upstream-check.md` and the `0.37.1` release gate so
  future fork/spec movement is visible before execution adapter work continues.
- Started `0.37.0` with the REVM dependency admission review, recording that
  current REVM candidates are not admitted because they fail cargo-deny/MSRV
  policy, and adding a code-visible review result in `eth-valkyoth-evm`.
- Expanded the roadmap for the first-party-core goal: core dependency
  independence audit, signature/crypto backend boundaries, native EVM
  execution, full block/state validity, trie-root construction, blob/KZG
  boundaries, and full execution fixture admission are now explicitly
  versioned before RPC/Reth/node work.
- Started `0.36.0` with a dev-only differential structural RLP harness against
  `alloy-rlp`, a documented runner command, mismatch reporting, and release
  gate coverage for the independent reference path.
- Addressed v0.36.0 pentest findings by making the differential `--check`
  command compile the real test target and adding `rlp_differential` fuzz
  coverage against `alloy-rlp`.
- Started `0.35.0` with the first external Ethereum execution fixture harness
  for pinned `ethereum/tests` `RLPTests`, explicit unsupported fixture groups,
  and dependency-version planning for the codec conformance package change.
- Started `0.34.0` with refreshed official Ethereum source pins, required
  spec-lock validation, and a reproducible external reference-store sync
  workflow for `/home/eldryoth/Work/test/eth`.
- Started `0.33.0` with account and storage Merkle Patricia Trie inclusion
  proof verification in `eth-valkyoth-verify`.
- Added `AccountTrieRoot`, `StorageTrieRoot`, `StorageSlotKey`,
  `VerifiedAccountInclusion`, and `VerifiedStorageInclusion` domain/result
  types.
- Reused the bounded MPT proof walker for exact account and storage value
  membership at `keccak256(address)` and `keccak256(slot_key)`.
- Addressed v0.33.0 pentest findings by adding real-Keccak fuzz coverage for
  transaction, receipt, account, and storage proof verification, expanding
  account/storage negative-path tests for wrong-root, absent, and depth-cap
  errors, and documenting storage-root composition at the call site.
- Started `0.32.0` with transaction and receipt Merkle Patricia Trie inclusion
  proof verification in `eth-valkyoth-verify`.
- Added transaction and receipt root domains, successful proof result tokens,
  malformed/absent/wrong-root proof error categories, and fixed proof-walk
  depth enforcement.
- Started `0.31.0` with bounded syntactic Merkle Patricia Trie node decoding in
  `eth-valkyoth-verify`.
- Added branch, extension, leaf, compact-path, hash-reference, inline-reference,
  and proof-node-list types with cumulative proof-node and byte accounting.
- Added `mpt_node` fuzz coverage and committed malformed-node seed fixtures.
- Addressed v0.31.0 pentest findings by enforcing the canonical inline child
  `< 32` encoded-byte rule and storing decoded branch children/value instead
  of reparsing them on access.
- Started `0.24.1` with EIP-7702 set-code transaction signing-preimage and
  signing-hash helpers for the type `0x04` transaction domain.
- Added EIP-7702 authorization tuple signing-preimage and signing-hash helpers
  for the `0x05 || rlp([chain_id, address, nonce])` authorization domain.
- Added decoded set-code transaction signature validation and separate
  authorization tuple signer recovery APIs with low-s, scalar, and y-parity
  policy.
- Added set-code authorization signature fuzz coverage with input-selected
  scratch-buffer lengths.
- Started `0.24.0` with unvalidated EIP-7702 set-code transaction field
  decoding and no-allocation canonical encoding for type byte `0x04`.
- Added bounded authorization-list tuple decoding, including chain ID,
  authorization address, nonce, y parity, `r`, and `s` domains, while
  explicitly deferring empty-list, authorization-signature, fee, fork, and
  account-state validation.
- Added set-code replay-domain checks and fail-closed decoded signature
  validation handling until the EIP-7702 signing-hash path is admitted.
- Extended transaction-envelope fuzz coverage and seed corpus entries for
  set-code transaction decode and encode paths.
- Addressed v0.24.0 pentest findings by keeping the EIP-7702 authorization
  signing magic out of the public API until the reviewed v0.24.1 helper lands
  and by adding authorization tuple sub-field decode diagnostics.
- Started `0.23.0` with decoded transaction signature validation helpers that
  combine replay-domain checks, transaction signing hashes, low-s/y-parity
  policy, sender recovery, and optional expected-sender comparison.
- Added valid-signature coverage for legacy EIP-155, EIP-2930, EIP-1559, and
  EIP-4844 decoded transactions plus wrong-chain, wrong-sender, high-s,
  malformed-scalar, and signing-hash construction failure tests.
- Addressed v0.23.0 pentest findings by making validated signature results
  crate-private to construct and adding external raw mainnet transaction KATs
  for typed sender recovery.
- Started `0.21.0` with EIP-712 domain-safety checks for required `chainId`
  and `verifyingContract` fields.
- Added EIP-712 signing digest framing with the EIP-191 `0x1901` prefix and a
  domain-gated sender recovery helper.
- Started `0.14.0` with unvalidated EIP-1559 dynamic-fee transaction field
  decoding for typed transaction byte `0x02`.
- Added max-priority-fee and max-fee field domains while explicitly deferring
  fee-order validation to later validation state.
- Extended transaction-envelope fuzz coverage to drive EIP-1559 dynamic-fee
  transaction decoding.
- Started `0.13.0` with unvalidated EIP-2930 access-list transaction field
  decoding for typed transaction byte `0x01`.
- Added bounded borrowed access-list entry and storage-key iteration plus stable
  access-list transaction decode error codes/categories.
- Extended transaction-envelope fuzz coverage to drive EIP-2930 access-list
  transaction decoding.
- Addressed v0.13.0 pentest findings by re-exporting access-list iterator
  types and documenting the eager-validation plus zero-copy re-parse model.
- Started `0.12.0` with unvalidated legacy transaction field decoding for the
  EIP-2718 legacy transaction field list.
- Added stable legacy transaction decode error codes/categories, call/create
  target modeling, bounded calldata exposure, and signature U256 width checks.
- Added a panic-free nonzero `eip155_chain_id` helper and direct
  signature-field caveats after v0.12.0 pentest feedback.
- Extended transaction-envelope fuzz coverage to drive legacy transaction field
  decoding before later typed transaction decoders land.
- Updated the pinned stable Rust toolchain and compatibility evidence through
  Rust `1.96.1`.
- Started `0.9.0` with canonical no-allocation RLP encoding helpers for
  scalar byte strings, Ethereum integer payloads, list payloads, and decoded
  RLP items.
- Added decode-then-encode canonicality tests plus scalar, integer, list,
  long-payload, output-buffer, and noncanonical-input regression coverage.
- Hardened raw list-payload encoding by validating concatenated child items
  under explicit `DecodeLimits` before returning a length or emitting a list
  header.
- Addressed v0.9.0 pentest findings by making encode errors leave output
  buffers unchanged, expanding encode fuzz coverage to exact-size output
  buffers, documenting sealed decoded value construction, and hardening
  long-form length invariants.
- Added fuzz coverage for RLP encoding length helpers, raw payload encoders,
  and decoded scalar, integer, list, and item re-encoding paths.
- Refreshed pinned execution-apis and consensus-specs revisions after checking
  official Ethereum sources for v0.9.0 codec work.
- Started `0.8.0` with canonical RLP integer decoding layered on top of
  scalar decoding, including exact and partial entry points.
- Added integer-specific rejection for single-byte zero and leading-zero
  payloads, preserving Ethereum's zero-as-empty-byte-array rule.
- Added bounded `u64`, `u128`, and unsigned 256-bit byte conversion helpers
  for canonical RLP integers.
- Added primitive constructors for canonical RLP integer payloads on the
  integer domain newtypes and `Wei`.
- Added fuzz coverage for exact and partial RLP integer decoding plus bounded
  integer conversion helpers.
- Addressed v0.8.0 pentest findings by documenting duplicated canonical
  integer logic, adding maximum-width integer and Wei boundary tests, clarifying
  U256 copy invariants, cross-referencing duplicated constants, and documenting
  Chain ID 0 domain validation requirements.
- Refreshed the pinned EIPs revision after checking official Ethereum sources
  for v0.8.0 parser work.
- Started `0.7.0` with bounded canonical RLP list decoding, including short
  and long list headers, nested traversal, list item-count enforcement, nesting
  depth enforcement, and adversarial malformed-list tests.
- Added no-allocation immediate-child iteration for decoded RLP lists through
  `RlpList::items`, `RlpListItems`, and `RlpItem`.
- Added `RlpItem::header_len`, `RlpItem::as_scalar`, `RlpItem::as_list`, and
  fused iterator behavior for `RlpListItems`.
- Split scalar and list RLP tests into separate modules and added official
  nested-list fixtures plus deeper canonical nesting budget regression coverage.
- Added fuzz coverage for exact and partial RLP list decoding paths, including
  immediate child iteration on successfully decoded lists.
- Addressed v0.7.0 pentest findings by making nested list iteration
  re-counting use the original decode limits, deepening list iterator fuzz
  coverage, documenting the RLP traversal hard cap and partial-decoder slicing
  contract, and clarifying long-list/string prefix constants.
- Addressed v0.7.0 pentest re-test findings by making `RlpList` equality
  ignore private decode policy, documenting independent iterator recount
  budgets, and simplifying list bounds-check expressions.
- Aligned every public workspace crate to `0.7.0` publication so crates.io
  receives corrected `MIT OR Apache-2.0` license metadata for all packages.
- Refreshed pinned official Ethereum source revisions for v0.7.0 parser work.
- Corrected the public crate license metadata and repository license files to
  `MIT OR Apache-2.0`.
- Started `0.6.0` with a dependency and tooling refresh before RLP scalar
  decoder work: updated `quote` to `1.0.46`, updated optional `sanitization`
  support to `1.2.2`, confirmed GitHub tooling is current, and added the
  v0.6 release gate.
- Added canonical RLP scalar byte-string decoding with exact-consumption,
  malformed length, list-prefix rejection, and budget enforcement tests.
- Added official scalar RLP example fixtures and long-length overflow coverage.
- Added fuzz coverage for exact and partial RLP scalar decoding paths.
- Refreshed pinned official Ethereum source revisions for v0.6.0 parser work.
- Addressed v0.6.0 pentest findings by gating codec test fixtures, renaming
  ambiguous decode-limit and partial-decoder APIs, adding hardened-only
  sanitization builds, and requiring explicit trusted-RPC acknowledgment.
- Started `0.5.0` by extending the decode-budget model with proof-node and
  cumulative item budgets, checked length and range helpers, and adversarial
  tests for overflow and limit rejection.
- Addressed v0.5.0 pentest findings for enum sanitization residual bytes,
  sanitization hardening evidence, spec-source pinning, decode limit naming,
  production-template fuzzing, hash timing documentation, typestate dead code,
  non-exhaustive public errors, TryFrom transaction type documentation, and
  skipped-field generic derive bounds.
- Addressed v0.5.0 follow-up pentest findings by making
  `SecureSanitizeOnDrop` struct-only and documenting downstream
  `HARDENED_MODE` assertion patterns.
- Started `0.4.0` by adding independent support-crate version planning,
  release-plan validation, and a crate version matrix to avoid unnecessary
  crates.io publishes.
- Added stable error codes, messages, categories, and formatting for codec,
  protocol, fork, feature, resource, and verification failures.
- Addressed v0.4.0 pentest findings for typestate token creation, address
  comparison timing, decode-limit API naming, sanitization skip acknowledgement,
  typed-envelope classification, best-effort sanitization visibility, and fuzz
  bootstrap coverage for all decode-budget APIs.
- Added crate-local READMEs for published support crates that point users to
  the `eth` facade crate.
- Added workspace packaging verification to local checks.
- Fixed facade crate docs to include a packaged README instead of a workspace
  root path.
- Initialized the `eth` Rust workspace.
- Added first-party `no_std` crate boundaries.
- Added security, supply-chain, modularity, implementation, and release plans.
- Added local check and release-gate scripts.
- Expanded the release plan into smaller milestone tags with explicit exit
  criteria and mandatory pentest-before-tag readiness checks.
- Added a spec-source policy requiring current official Ethereum sources,
  pinned revisions, and local fixture evidence before consensus-sensitive work.
- Addressed v0.1.0 pentest release-gate findings for CI pinning, advisory
  policy, release readiness, lints, and metadata validation.
- Added explicit secret-handling policy and hardened current placeholder
  primitives/protocol helpers flagged during pentest.
- Added advisory checks for pinned CI tools and GitHub Actions currency.
- Started `0.2.0` by moving support crates to the `eth-valkyoth-*` namespace
  and adding a crates.io release-order helper.
- Added release-readiness negative tests for missing or stale release evidence.
- Addressed v0.2.0 pentest findings for constant-time equality, decode-limit
  enforcement, fork activation semantics, typestate direction, advisory policy,
  deterministic release gates, and RPC trust-model defaults.
- Implemented `0.3.0` domain newtypes with explicit wei and transaction type
  primitives, conversion coverage, and the v0.3 release gate.
- Added optional `eth-valkyoth-sanitization` and `eth-valkyoth-derive` support
  crates outside the default `eth` feature set.
- Addressed v0.3.0 pentest findings for constant-time primitive equality,
  cumulative decode allocation accounting, enum sanitization acknowledgement,
  typed transaction disambiguation, and release/tooling gates.
