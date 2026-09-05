# Roadmap Scope And Completeness Review

Baseline: commit `958f47d`, after the September upstream review and published
`v0.55.0`. This is a planning review, not implementation or pentest evidence.

## Findings And Changes

The coverage was broad enough to describe a full Ethereum library/client
family, but release sizes were inconsistent. Several short bullets hid whole
subsystems; long evidence milestones mixed kernel, attribution, persistence
and benchmarking work. The new upstream index added too much to the former
`v0.116.0`. Some planned patches introduced new APIs or substantial algorithms
rather than narrowly compatible fixes.

The revised [release plan](RELEASE_PLAN.md):

- retains all 296 previously planned workstream contracts;
- extracts 98 implementation passes across 50 workstreams;
- promotes all 11 previously planned patch milestones to sequential minors;
- contains 482 total milestones including historical releases and 1.0/RC;
- assigns unpublished minor milestones from `v0.56.0` through `v0.449.0`;
- gives every planned pre-1.0 release an explicit scope and predecessor;
- preserves all original deliverable, verification and exit text, apart from
  translating future version references; broad workstreams become bounded
  completion passes with explicitly named remaining implementation;
- removes the repetitive "deliver the ... release" goal prefix;
- keeps signed tags/pentests per milestone and crates.io checkpoints at every
  fifth minor; published tags, package versions and reports are not changed.

The [complete old-to-new map](roadmap-version-map.json) includes every former
planned assignment, not just changed titles. A range starts at the first
extracted pass; a single reference to a former workstream points to its
completion. Dated evidence keeps its old numbers with a mapping notice.

## Principal Splits

| Former owner | New passes | Reason |
| --- | --- | --- |
| `v0.56.0..=v0.57.0` | `v0.56.0..=v0.61.0` | Fp/Fp2, G1/G2 group operations and charged addition are separate steps. |
| `v0.61.0` | `v0.65.0..=v0.68.0` | Pairing tower, Miller loop and final exponentiation need independent oracles before composition. |
| `v0.68.0..=v0.70.0` | `v0.75.0..=v0.86.0` | Field/point arithmetic, nonce derivation, ECDSA, ECDH/recovery, symmetric primitives and protocol transcripts have different security boundaries. |
| `v0.74.0` | `v0.90.0..=v0.93.0` | Minimal invalidity authority, peer/batch attribution and optional cache/sink behavior must be reviewed separately. |
| `v0.77.0..=v0.82.0` | `v0.96.0..=v0.109.0` | Context issuance, reservation lifecycle, collection modes, API containment, arenas and measurement integrity become individual passes. |
| `v0.104.0..=v0.104.1` | `v0.135.0..=v0.139.0` | Journaling, EIP-7702 state application, system calls and request roots are separated. |
| `v0.108.0..=v0.109.0` | `v0.144.0..=v0.148.0` | Scalar/FFT foundations, single KZG proofs and batch soundness have distinct admission evidence. |
| `v0.116.0` | `v0.155.0..=v0.165.0` | Ten concrete Osaka/current-fork implementation passes precede cross-EIP integration. |
| `v0.177.0` | `v0.243.0..=v0.245.0` | Full BLS message hashing/ciphersuite and aggregate verification precede sync-committee policy. |
| `v0.185.0..=v0.187.0` | `v0.253.0..=v0.263.0` | Discovery, signed DNS trees, RLPx, baseline/modern eth messages and Snap consumption/serving no longer share one implementation tag. |
| `v0.190.0` | `v0.266.0..=v0.268.0` | Pool admission/replacement is separate from reorg, blob and delegation retention. |
| `v0.200.0` | `v0.278.0..=v0.281.0` | A selected successor scheme needs explicit source admission, arithmetic and executable proofs before integration. |
| `v0.221.0..=v0.223.0` | `v0.306.0..=v0.315.0` | Progressive/mutable SSZ, secret BLS signing, batching, cell proofs and reconstruction receive separate stops. |
| `v0.231.0`, `v0.237.0` | `v0.323.0..=v0.325.0`, `v0.331.0..=v0.334.0` | ePBS builder state/timeliness and FOCIL enforcement are executable tasks, not an adapter-only promise. |
| `v0.316.0..=v0.317.0` | `v0.418.0..=v0.423.0` | HTTP, WS/IPC, event serving and diagnostics/GraphQL have independent admission and abuse tests. |
| `v0.336.0` | `v0.442.0..=v0.445.0` | Independent downstream clients and lightweight consumers must work before final acceptance. |

The map also covers smaller splits in text/serde, public SDK, IPC, remote
signers, bundlers/paymasters, ENS/signature standards, storage migration,
Engine, gossip, custody, Beacon API, Keymanager and distributed signing.

## Dependency Corrections

The cryptography inventory explicitly includes AES-GCM/GHASH and HKDF for
discovery, not just RLPx AES-CTR. The
[official discv5 wire specification](https://github.com/ethereum/devp2p/blob/master/discv5/discv5-wire.md)
uses both AES-CTR header masking and AES-GCM message authentication.

SHA-512/HMAC-SHA512 and password KDFs now have an admission pass before
mnemonic/derivation/keystore consumers. Full BLS message hashing is distinct
from EIP-2537 field-to-curve precompiles: the ciphersuite pass must use pinned
consensus requirements and [hash-to-curve vectors](https://www.rfc-editor.org/rfc/rfc9380.html).
Existing group/precompile arithmetic alone is not signature verification.

The scope rules distinguish a preceding implementation baseline from a
forward normative requirement. No consumer may claim production completion
using a test double or a future implementation. Any newly discovered concrete
prerequisite is moved or split before that consumer is started. Request-only
RPC servers explicitly leave subscription service completion to the following
event workstream rather than claiming it early.

## Product Coverage

These are required build/use cases, not claims about today's code:

| Consumer/product | Primary implementation and acceptance owners |
| --- | --- |
| no_std protocol, arithmetic, codec and verification library | Foundations, `v0.56.0..=v0.128.0`; independent packaged consumer at `v0.125.0` and final `v0.444.0`. |
| Execution library, simulation, proofs and historical forks | `v0.129.0..=v0.173.0`, witness work `v0.273.0..=v0.285.0`, Ethash/genesis-to-Merge `v0.411.0..=v0.413.0`. |
| Providers, wallets, account abstraction, ABI/contracts and application standards | `v0.174.0..=v0.218.0`, live-client evidence and final lightweight-consumer gate. |
| Custom storage/runtime, archive/pruned node, indexing and recovery | `v0.219.0..=v0.232.0`, production adapters/tooling `v0.414.0..=v0.433.0`. |
| Light client and proof-backed applications | `v0.233.0..=v0.250.0` and final packaged light/stateless consumer. |
| Execution P2P, txpool, full/snap/history sync | `v0.252.0..=v0.272.0`, followed by executable/Hive/public-sync gates. |
| Complete execution client built by users or shipped by this project | `v0.414.0..=v0.433.0`, external build/runtime/database replacement at `v0.442.0`. |
| Beacon transition, fork choice, networking, DA and services | `v0.304.0..=v0.364.0`, whole-product assurance through `v0.406.0`. |
| Validator, slashing protection, key lifecycle and external custody | `v0.365.0..=v0.382.0`, signed-duty and long-running independent-client evidence. |
| Builder/ePBS/FOCIL interoperability | Dedicated state/fork-choice passes plus block production, Builder/Beacon/Keymanager/Engine API and mixed-client acceptance. |
| Integrated node, private networks and operable executables | `v0.434.0..=v0.441.0`, independent beacon/validator embedding at `v0.443.0`. |
| Platforms, auditing, formal evidence and final production stability | Continuing per-pass gates, platform gate `v0.286.0`, component/full-stack audits and final `v0.445.0..=v0.449.0`, then exact RC/1.0. |

The objective remains first-party Ethereum functionality with optional
integration dependencies and independently replaceable client roles. A general
Rust library must remain usable without adopting this repository's binaries,
runtime, storage engine or signer. It must also keep small no_std consumers
practical rather than treating them as accidental subsets of a server.

No static document can certify all future Ethereum proposals or arbitrary
ecosystem applications. Completeness is checked against pinned adopted rules,
explicit standard/application support and real consumer evidence. Newly
admitted requirements receive versions, including maintenance lanes; 1.0 is
blocked by gaps in advertised support, not authorized by reaching a version
number. Every pass still undergoes source review and may be split again when
its concrete implementation inventory exceeds one reviewable boundary.

## Review Evidence

The reslicing operation checked that every original planned Deliverables,
Verification and Exit criteria block survived after version-reference
translation. Existing historical renumbering notes were preserved verbatim;
no release report/tag or published crate version was rewritten.

Automated checks cover section order/content, version ordering, exact-commit
pentest stops, the complete migration map and all predecessor edges. Tests
reject missing scope, missing mapped milestones, duplicate mapping, incorrect
completion titles and forward/self prerequisites. README copies, active
roadmap references and documentation links are checked as well.

No Ethereum implementation changed in this planning pass. Its tests validate
the roadmap and release-tooling safeguards, not future Ethereum correctness.
