<p align="center">
  <b>no_std-first Ethereum protocol building blocks for Rust.</b><br>
  Explicit domains, bounded decode policy, constant-time primitives, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">Crates.io</a>
  |
  <a href="https://docs.rs/eth">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/eth">
    <img src="https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth

`eth` is a `no_std`-first Rust workspace for Ethereum execution-layer protocol
building blocks.

The project target is a production-ready Ethereum crate at `1.0.0`, reached
through small releases with explicit security, conformance, and dependency
evidence. The first implementation work is intentionally conservative:
explicit domains, bounded decode policy, stable crate boundaries, and security
documentation before RPC, signer, REVM, Reth, or P2P adapters become real
dependencies.

## Current Status

Status: `v0.52.1` adds canonical dependency-free EIP-2537 BLS12-381 field,
scalar, point-coordinate, and precompile-frame parsing while keeping arithmetic
and curve/subgroup validation fail closed.
The optional `evm-core` feature now exposes dependency-free no_std word, stack,
memory, opcode, program-counter, fork, gas schedule, opcode-table, host-state,
warm/cold access, historical fork identifiers, opcode-introduction metadata,
call/create planning domains, return-data ranges, journal checkpoint policy,
fork-aware precompile descriptors, bounded precompile input/gas policies, and
bounded interpreter domains for hard-capped basic stack/control-flow bytecode
plus explicit state reads. Identity, SHA-256, RIPEMD-160, bounded ModExp,
BN254 add/mul, BN254 pairing frames, and BLAKE2F execute without
dependencies; ECRECOVER executes through explicit caller-provided secp256k1
and Keccak boundaries. KZG and BLS cryptographic precompiles remain fail-closed
execution descriptors. Canonical BLS wire parsing is complete; curve,
subgroup, arithmetic, mapping, MSM, and pairing execution remain assigned to
`v0.52.2..=v0.52.9`, with KZG work assigned to `v0.61.0..=v0.61.5`.

Implemented now:

- Rust workspace pinned to stable `1.97.0`.
- MSRV policy for Rust `1.90.0` through `1.97.0`.
- `no_std` facade and focused first-party crates.
- Explicit primitive domains for chain, block, gas, nonce, timestamp, address,
  hash, wei, and transaction type values.
- Constant-time equality composition for fixed-width hash and wei values.
- Bounded decode limits plus stateful cumulative allocation, item, and proof-node
  accounting.
- Canonical RLP scalar, list, and integer decoding plus no-allocation canonical
  encoding helpers.
- No-allocation primitive RLP encode and exact-decode helpers for chain, block,
  gas, nonce, timestamp, address, hash, and wei values.
- EIP-2718 transaction envelope shell classification for typed and legacy
  transaction bytes.
- Unvalidated legacy transaction field decoding for nonce, gas price, gas
  limit, to/create, value, input, and signature words.
- Unvalidated EIP-2930 access-list transaction field decoding, including
  bounded borrowed access-list entry and storage-key iteration.
- Unvalidated EIP-1559 dynamic-fee transaction field decoding for max priority
  fee, max fee, gas limit, destination/create, value, calldata, access list, and
  signature words.
- Unvalidated EIP-4844 blob transaction field decoding for blob fee, required
  call target, blob versioned hash list, calldata, access list, and signature
  words.
- Unvalidated EIP-7702 set-code transaction field decoding for destination,
  calldata, access list, authorization list, and signature words.
- No-allocation canonical transaction envelope encoding for admitted
  unvalidated legacy, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transaction
  domains.
- Explicit caller-provided `ChainSpec`, `ForkSpec`, `Hardfork`, and
  `ValidationContext` APIs for fork activation selection, including
  fail-closed checks for duplicate forks, wrong-chain entries, and
  non-monotonic fork or activation ordering.
- Unvalidated execution block header decoding for legacy, London, Shanghai,
  Cancun, and Prague field sets, plus block header hashing through the Keccak
  trait boundary and a distinct `BlockHash` domain newtype.
- Unvalidated legacy and EIP-2718 typed receipt decoding, including
  status/root policy, 256-byte logs bloom, borrowed log entries, topics, and
  data.
- Unvalidated EIP-4895 withdrawal-list decoding, including global withdrawal
  indexes, validator indexes, recipient addresses, and nonzero Gwei amounts.
- Bounded syntactic MPT node decoding for branch, extension, and leaf nodes,
  including compact-path validation, eager inline child shape checks, and
  cumulative proof-node byte/count accounting.
- Transaction and receipt MPT inclusion proof verification at
  `rlp(transaction_index)` over the caller-provided Keccak-256 trait boundary,
  with distinct transaction and receipt root domains.
- Account and storage MPT inclusion proof verification at
  `keccak256(address)` and `keccak256(slot_key)` over the caller-provided
  Keccak-256 trait boundary, with distinct account and storage root domains.
- Proof-gated transaction typestate transitions for decoded, canonical,
  fork-validated, and sender-recovered state tokens.
- Replay-domain validation for legacy EIP-155 and typed transaction chain IDs
  before sender recovery results are accepted.
- Canonical transaction signing-preimage encoding and signing-hash helpers for
  legacy EIP-155, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 decoded
  transaction domains.
- EIP-7702 authorization tuple signing-hash and signer recovery helpers, kept
  separate from transaction signing hashes with explicit domain newtypes.
- EIP-7702 set-code transaction validity gate for Prague/Pectra fork context,
  non-empty authorization lists, fee order, caller-computed gas policy, and
  caller-provided authority account-state checks. Per-authorization failures
  are counted as skipped tuples instead of rejecting the whole transaction.
- Optional `evm` feature exposing explicit no-std execution environment,
  transaction input, state snapshot, result report, and hard-ceiling
  gas-estimation policy types without admitting an EVM backend.
- Optional `evm-core` feature exposing first-party no_std EVM word, stack,
  memory, opcode, program-counter, fork, gas schedule, opcode-table,
  host-state traits, warm/cold access tracking, and basic bounded interpreter
  domains with bytecode, step, gas, state-access, call/create, and precompile
  ceilings as the native execution foundation.
- Digest-level secp256k1 sender recovery through a caller-provided backend
  boundary, with low-s rejection, Ethereum y-parity policy, and caller-provided
  Keccak-256 public-key hashing.
- Decoded transaction signature validation helpers that combine replay-domain
  checks, signing hashes, low-s/y-parity policy, sender recovery, and optional
  expected-sender comparison.
- External raw mainnet transaction KATs for EIP-2930, EIP-1559, EIP-4844, and
  EIP-7702 sender recovery.
- EIP-712 domain-safety checks for required `chainId` and
  `verifyingContract` fields, plus a domain-gated sender recovery helper.
- No-allocation EIP-712 typed-data encoder for caller-provided schemas and
  borrowed values, including `encodeType`, `encodeData`, `hashStruct`, domain
  separator construction, and typed-data signing digest construction.
- Optional `eip712-json` parser boundary for JSON-RPC typed-data payloads with
  duplicate-key rejection, explicit parser limits, and no default dependency
  impact.
- Optional `keccak-tiny` software backend using reviewed `tiny-keccak`,
  disabled by default and covered by Keccak-256 KATs.
- Core dependency independence audit covering default `subtle`, optional
  `k256`, `tiny-keccak`, `serde`, `serde_json`, and `sanitization` paths, and
  dev/reference `alloy-rlp` usage.
- Runtime dependency policy gate proving reference crates and optional
  backend/parser crates do not enter the default facade graph.
- Optional parser and sanitization policy gate proving `serde`, `serde_json`,
  `eth-valkyoth-sanitization`, and `sanitization` enter only through explicit
  opt-in features.
- Public `RlpEncode`/`RlpDecode` traits and derive macros for reviewed simple
  structs, with bounded decode and trybuild compile-fail coverage.
- Caller-provided Keccak-256 trait boundary with no default hash
  implementation dependency.
- RLP fuzz harness with committed hex seed corpus and crash reproduction docs.
- Stable error codes, messages, categories, and formatting for codec,
  protocol, fork, feature, resource, and verification failures.
- Optional sanitization and derive support crates outside the default feature
  set.
- MIT OR Apache-2.0 license.
- Security, modularity, supply-chain, implementation, and release planning docs.
- Local check, release-gate, dependency-policy, SBOM, and pentest evidence.
- Safe upstream advisory checking for REVM and official Ethereum source drift.
- Independent support-crate release planning for crates.io push limits.
- Pinned official Ethereum source revisions plus a reproducible external
  reference-store sync process for `/home/eldryoth/Work/test/eth`.
- Execution fixture harness for pinned `ethereum/tests` `RLPTests`, with a
  pass/fail report and explicit unsupported fixture list.
- Differential structural RLP harness against `alloy-rlp`, with accumulated
  mismatch reporting.

Not implemented yet:

- No RPC transport.
- No signer or local key storage.
- No stateful or production-valid EVM execution adapter.
- No Reth or P2P integration.
- No block parser yet.
- Identity, SHA-256, RIPEMD-160, ECRECOVER, bounded ModExp, BN254 add/mul,
  BN254 pairing execution, G2 subgroup validation, BLAKE2F execution, the
  Fp6/Fp12 tower foundation, validated tuple streaming, line-function
  foundation, Miller-loop accumulator with sparse line-factor multiplication
  evidence, optimized bounded final exponentiation, Frobenius Q1/-Q2 point
  mapping, and the projective post-loop line carrier are implemented; non-empty
  pairing result admission now returns canonical EIP-197 zero/one words, while
  BLAKE2F now follows EIP-152 input and output rules. `v0.52.1` adds canonical
  dependency-free BLS12-381 Fp, Fr, Fp2, unrestricted MSM scalar, G1/G2
  coordinate, infinity, and precompile-frame parsing. These point values are
  wire-valid only until the curve and subgroup releases land in
  `v0.52.2..=v0.52.9`; KZG execution remains scheduled through `v0.61.5`.
- No ABI/contract helper surface yet; scheduled for `v0.70.0` through
  `v0.78.0`.
- No consensus/Engine API support yet; scheduled for `v0.79.0` through
  `v0.85.0`.
- No P2P, txpool, sync, mining, builder, or validator-adjacent boundary yet;
  scheduled for `v0.86.0` through `v0.92.0`.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned toolchain | Rust `1.97.0` |
| Default target | `no_std` |
| Default runtime dependencies | protocol-core support crates plus reviewed `subtle` exception |
| Optional hardening dependencies | `sanitization` and proc-macro tooling behind opt-in crates/features |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | protocol-core only |
| Network/signing defaults | none |
| Release evidence | local gates, cargo-deny, cargo-audit, SBOM, pentest report |
| Formal verification | Kani harness planned for `v0.94.0` as extra assurance |
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready Ethereum execution-layer toolkit |

## Install

```toml
[dependencies]
eth = "0.52.1"
```

For optional sanitization support:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["sanitization"] }
```

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `std` | no | Enables `std` support in admitted core crates. |
| `evm` | no | Explicit no_std EVM execution environment, snapshot, result, and bounded gas-estimation boundary. |
| `evm-core` | no | Dependency-free native EVM core domains, gas-metered basic opcode execution, explicit bounded state-access traits, and precompile planning. |
| `rpc` | no | Future explicit RPC trust-policy boundary. |
| `eip712-json` | no | Enables the optional `std` JSON-RPC EIP-712 typed-data parser boundary. |
| `keccak-tiny` | no | Enables the optional reviewed `tiny-keccak` software backend. |
| `secp256k1-k256` | no | Enables the optional reviewed `k256` sender-recovery adapter. |
| `sanitization` | no | Re-exports optional secret sanitization bridge APIs. |
| `signer` | no | Future signer isolation boundary. |
| `reth` | no | Future Reth integration boundary. |
| `testkit` | no | Test fixtures, conformance helpers, and adversarial inputs. |

Default builds do not enable networking, signing, local key storage, Reth, P2P,
REVM, or concrete production EVM execution. The optional `evm` and `evm-core`
features provide boundary and native core execution primitives only.

Optional reviewed software Keccak backend:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["keccak-tiny"] }
```

```rust
use eth::hash::{KECCAK256_ABC, TinyKeccak256, hash_one};

let digest = hash_one(TinyKeccak256::default(), b"abc");
assert_eq!(<[u8; 32]>::from(digest), KECCAK256_ABC);
```

Optional reviewed secp256k1 recovery adapter:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["secp256k1-k256"] }
```

Optional bounded EVM gas-estimation boundary:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["evm"] }
```

```rust
use eth::codec::DecodeLimits;
use eth::evm::{
    BlockExecutionContext, ExecutionEnvironment, ExecutionRequest, ExecutionTransaction,
    GasEstimationPolicy, GasEstimationRequest, GasEstimationStatus,
    GasEstimationTermination, SnapshotAccount, SnapshotError, StateSnapshot,
};
use eth::primitives::{Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei};
use eth::protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};

struct Snapshot;

impl StateSnapshot for Snapshot {
    fn snapshot_id(&self) -> B256 {
        B256::from_bytes([0_u8; 32])
    }

    fn account(&self, _address: Address) -> Result<Option<SnapshotAccount>, SnapshotError> {
        Ok(Some(SnapshotAccount {
            nonce: Nonce::new(0),
            balance: Wei::from_u128(0),
            code_hash: B256::from_bytes([0_u8; 32]),
        }))
    }

    fn storage(&self, _address: Address, _slot: B256) -> Result<B256, SnapshotError> {
        Ok(B256::from_bytes([0_u8; 32]))
    }
}

let context = ValidationContext {
    fork: ForkSpec {
        chain_id: ChainId::new(1),
        hardfork: Hardfork::Prague,
        activation: ForkActivation::BlockAndTimestamp {
            activation_block: BlockNumber::new(10),
            activation_timestamp: UnixTimestamp::new(20),
        },
    },
    block_number: BlockNumber::new(12),
    timestamp: UnixTimestamp::new(22),
};
let block = BlockExecutionContext {
    chain_id: ChainId::new(1),
    block_number: BlockNumber::new(12),
    timestamp: UnixTimestamp::new(22),
    beneficiary: Address::from_bytes([0_u8; 20]),
    gas_limit: Gas::new(30_000_000),
    base_fee_per_gas: Wei::from_u128(1_000_000_000),
    prev_randao: B256::from_bytes([0_u8; 32]),
};
let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};

let environment = match ExecutionEnvironment::try_new(context, block) {
    Ok(environment) => environment,
    Err(error) => return Err(error.message()),
};
let transaction = match ExecutionTransaction::decode(&[0xc0], limits) {
    Ok(transaction) => transaction,
    Err(error) => return Err(error.message()),
};
let snapshot = Snapshot;
let execution = ExecutionRequest::new(environment, transaction, &snapshot);
let policy = match GasEstimationPolicy::try_new(
    8,
    Gas::new(50_000),
    GasEstimationTermination::BackendStepLimit {
        max_backend_steps: 1_000,
    },
) {
    Ok(policy) => policy,
    Err(error) => return Err(error.message()),
};
let request = match GasEstimationRequest::try_new(execution, policy) {
    Ok(request) => request,
    Err(error) => return Err(error.message()),
};
let report = match request.report(
    B256::from_bytes([0_u8; 32]),
    GasEstimationStatus::BackendUnavailable,
    0,
    None,
) {
    Ok(report) => report,
    Err(error) => return Err(error.message()),
};

assert_eq!(report.policy.gas_cap(), Gas::new(50_000));
# Ok::<(), &'static str>(())
```

Optional native EVM core domains:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["evm-core"] }
```

State access uses explicit host-state traits and caller-provided fixed-capacity
warm/cold access sets. Frontier through Istanbul use explicit flat historical
state-read pricing for the currently executable subset; Berlin and later use
warm/cold access accounting. See
[`docs/evm-fork-matrix.md`](docs/evm-fork-matrix.md) for the current native EVM
fork and opcode support matrix.

EIP-2537 wire parsers validate exact frame lengths, zero padding, field bounds,
coefficient order, and the unique all-zero infinity encoding without allocating.
Returned G1/G2 values are canonical wire coordinates, not yet proof of curve or
subgroup membership. See
[`docs/bls12-381-wire-encodings.md`](docs/bls12-381-wire-encodings.md).

```rust
use eth::evm_core::{EVM_BLS12381_G1_POINT_BYTES, EvmBls12381G1Point};

let encoded = [0_u8; EVM_BLS12381_G1_POINT_BYTES];
let point = EvmBls12381G1Point::try_from_be_bytes(&encoded)?;
assert!(point.is_infinity());
# Ok::<(), eth::error::EvmCoreError>(())
```

```rust
use eth::evm_core::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmExecution, EvmFork, EvmOpcode, EvmStack,
    EvmWord, ExecutionLimits, OpcodeClass, OpcodeTable,
};

let mut stack = EvmStack::<16>::try_new()?;
stack.push(EvmWord::ZERO)?;

let table = OpcodeTable::try_new(EvmFork::CANCUN)?;
let add = table.instruction(EvmOpcode::ADD)?;
assert_eq!(add.class, OpcodeClass::Arithmetic);

let mut memory = [0_u8; 0];
let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
let report = execution.run(
    &[0x60, 0x02, 0x60, 0x03, 0x01, 0x00],
    ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT, EVM_DEFAULT_GAS_LIMIT, EvmFork::CANCUN)?,
)?;
assert_eq!(report.stack_len, 1);
assert_eq!(report.gas_used.get(), 9);
# Ok::<(), eth::error::EvmCoreError>(())
```

Precompiles are explicit and fork-aware. Identity, SHA-256, RIPEMD-160,
bounded ModExp, BN254 add/mul, BN254 pairing frames, BLAKE2F, and ECRECOVER
can execute now; ECRECOVER requires caller-provided secp256k1 and Keccak
backends. ModExp uses a first-party no-alloc engine with an explicit release
operand cap. BN254 add/mul uses first-party fixed-size field arithmetic with
canonical field and point validation. BN254 pairing validates bounded frames,
G2 curve membership, and G2 subgroup membership, streams validated tuples into
the internal Miller-loop accumulator, executes empty input as one, and returns
canonical EIP-197 zero/one output words for non-empty valid frames. BLAKE2F
executes the EIP-152 compression function with exact 213-byte input parsing,
final-flag validation, and round-count gas.
Dispatcher-facing identity, hash, ECRECOVER, ModExp, BN254 add/mul, BN254
pairing, and BLAKE2F execution is available only through plans that charge the
supplied gas meter on every call before output mutation or expensive work.
KZG and BLS cryptographic precompiles expose exact fork, frame, output, and gas
plans and return a backend-unavailable error until their first-party arithmetic
releases are admitted. BLS MSM and pairing plans reject empty and partial item
lists and apply the official EIP-2537 gas schedule.

```rust
use eth::evm_core::{
    EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry,
};

let registry = EvmPrecompileRegistry::try_new(EvmFork::CANCUN)?;
let descriptor = registry.descriptor(EvmPrecompileKind::Identity)?;
let plan = EvmPrecompilePlan::try_new(descriptor, b"eth")?;
let mut output = [0_u8; 3];
let mut gas = EvmGasMeter::try_new(EvmGas::new(18))?;

assert_eq!(plan.execute_identity(&mut gas, b"eth", &mut output)?, 3);
assert_eq!(gas.used(), EvmGas::new(18));
assert_eq!(&output, b"eth");
# Ok::<(), eth::error::EvmCoreError>(())
```

## Primitive Domains

Use explicit Ethereum domains instead of unqualified integers and byte arrays:

```rust
use eth::primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, TransactionType, Wei,
};

let chain = ChainId::new(1);
let block = BlockNumber::new(19_000_000);
let gas = Gas::new(21_000);
let nonce = Nonce::new(7);
let address = Address::from([0x11_u8; 20]);
let hash = B256::from([0x22_u8; 32]);
let value = Wei::from_u128(1_000_000_000_000_000_000);
let tx_type = TransactionType::try_new_typed(2);

assert_eq!(u64::from(chain), 1);
assert_eq!(u64::from(block), 19_000_000);
assert_eq!(u64::from(gas), 21_000);
assert_eq!(u64::from(nonce), 7);
assert_eq!(<[u8; 20]>::from(address), [0x11_u8; 20]);
assert_eq!(<[u8; 32]>::from(hash), [0x22_u8; 32]);
assert_eq!(value.to_be_bytes()[31], 0);
assert_eq!(tx_type.map(u8::from), Ok(2));
```

Primitive domains bridge directly to the bounded codec without allocation:

```rust
use eth::codec::DecodeLimits;
use eth::primitives::{Address, ChainId, Wei};

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 4,
    max_nesting_depth: 4,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 4,
};

let chain = ChainId::new(1);
let mut encoded_chain = [0_u8; 8];
let written = chain.encode_rlp(&mut encoded_chain)?;
assert_eq!(encoded_chain.get(..written), Some([0x01].as_slice()));
assert_eq!(ChainId::try_from_rlp(&[0x01], limits)?, chain);

let value = Wei::from_u128(1024);
let mut encoded_value = [0_u8; 8];
let written = value.encode_rlp(&mut encoded_value)?;
assert_eq!(encoded_value.get(..written), Some([0x82, 0x04, 0x00].as_slice()));
assert_eq!(Wei::try_from_rlp(&[0x82, 0x04, 0x00], limits)?, value);

let address = Address::from([0x11_u8; 20]);
let mut encoded_address = [0_u8; 21];
let written = address.encode_rlp(&mut encoded_address)?;
assert_eq!(written, 21);
assert_eq!(Address::try_from_rlp(&encoded_address, limits)?, address);
# Ok::<(), eth::primitives::PrimitiveRlpError>(())
```

## Transaction Decode

Transaction decoders return explicitly unvalidated borrowed field models. They
classify and bound wire data, but do not validate signatures from the full
transaction, check account state, or prove fork validity:

```rust
use eth::codec::DecodeLimits;
use eth::primitives::{Gas, Nonce, Wei};
use eth::protocol::{
    DynamicFeeTransactionTo, SignatureYParity, decode_dynamic_fee_transaction,
    encode_dynamic_fee_transaction,
};

let dynamic_fee_tx = [
    0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80,
    0xc0, 0x01, 0x01, 0x02,
];

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};
let tx = decode_dynamic_fee_transaction(&dynamic_fee_tx, limits)?;

assert_eq!(tx.chain_id.get(), 1);
assert_eq!(tx.nonce, Nonce::new(2));
assert_eq!(tx.max_priority_fee_per_gas, Wei::from_u128(3));
assert_eq!(tx.max_fee_per_gas, Wei::from_u128(4));
assert_eq!(tx.gas_limit, Gas::new(21_000));
assert_eq!(tx.to, DynamicFeeTransactionTo::Create);
assert_eq!(tx.value, Wei::from_u128(5));
assert_eq!(tx.access_list.address_count(), 0);
assert_eq!(tx.access_list.storage_key_count(), 0);
assert_eq!(tx.y_parity, SignatureYParity::Odd);

let mut encoded = [0_u8; 32];
let written = encode_dynamic_fee_transaction(&tx, &mut encoded)?;
assert_eq!(encoded.get(..written), Some(dynamic_fee_tx.as_slice()));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Replay Domain Checks

Replay-domain helpers reject wrong-chain transactions before sender recovery
results are trusted:

```rust
use eth::codec::DecodeLimits;
use eth::primitives::ChainId;
use eth::protocol::decode_dynamic_fee_transaction;
use eth::verify::{VerifyError, require_dynamic_fee_replay_domain};

let dynamic_fee_tx = [
    0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80,
    0xc0, 0x01, 0x01, 0x02,
];

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};
let tx = decode_dynamic_fee_transaction(&dynamic_fee_tx, limits)?;

require_dynamic_fee_replay_domain(ChainId::new(1), &tx)?;
assert_eq!(
    require_dynamic_fee_replay_domain(ChainId::new(5), &tx),
    Err(VerifyError::WrongChain)
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Transaction Signing Hashes

Decoded transaction domains can be converted into canonical signing hashes
without admitting a default hash backend:

```rust
use eth::hash::Keccak256;
use eth::primitives::B256;
use eth::protocol::decode_dynamic_fee_transaction;
use eth::verify::dynamic_fee_transaction_signing_hash;
use eth::codec::DecodeLimits;

struct PlatformKeccak {
    output: B256,
}

impl Keccak256 for PlatformKeccak {
    fn update(&mut self, input: &[u8]) {
        let _ = input;
    }

    fn finalize(self) -> B256 {
        self.output
    }
}

let dynamic_fee_tx = [
    0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80,
    0xc0, 0x01, 0x01, 0x02,
];
let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};
let tx = decode_dynamic_fee_transaction(&dynamic_fee_tx, limits)?;
let mut scratch = [0_u8; 64];
let signing_hash = dynamic_fee_transaction_signing_hash(
    &tx,
    &mut scratch,
    PlatformKeccak {
        output: B256::from([0x44_u8; 32]),
    },
)?;

assert_eq!(signing_hash.to_b256(), B256::from([0x44_u8; 32]));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The example hasher is illustrative only. Production hashers must compute
Ethereum Keccak-256. For full decoded transaction signature validation, use
`validate_transaction_signature` or the type-specific validation helpers so
replay-domain checks, signing-hash construction, low-s/y-parity policy, sender
recovery, and optional expected-sender comparison are applied together. Callers
that reuse the scratch buffer across multiple in-flight transactions should
zero it after hashing before reusing or releasing it.

EIP-7702 authorization tuples use a separate signing-hash domain:

```rust
use eth::hash::Keccak256;
use eth::primitives::{Address, B256, Nonce};
use eth::protocol::{SetCodeAuthorization, SetCodeAuthorizationChainId, SignatureYParity};
use eth::verify::set_code_authorization_signing_hash;

struct PlatformKeccak {
    output: B256,
}

impl Keccak256 for PlatformKeccak {
    fn update(&mut self, input: &[u8]) {
        let _ = input;
    }

    fn finalize(self) -> B256 {
        self.output
    }
}

let mut chain_id = [0_u8; 32];
if let Some(last) = chain_id.last_mut() {
    *last = 1;
}
let authorization = SetCodeAuthorization {
    chain_id: SetCodeAuthorizationChainId::from_be_bytes(chain_id),
    address: Address::from([0x11_u8; 20]),
    nonce: Nonce::new(7),
    y_parity: SignatureYParity::Even,
    r: [0_u8; 32],
    s: [0_u8; 32],
};
let mut scratch = [0_u8; 128];
let authorization_hash = set_code_authorization_signing_hash(
    authorization,
    &mut scratch,
    PlatformKeccak {
        output: B256::from([0x55_u8; 32]),
    },
)?;

assert_eq!(authorization_hash.to_b256(), B256::from([0x55_u8; 32]));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## EIP-712 Typed Data

EIP-712 signing paths can build the structured-data digest from reviewed
borrowed type descriptors and values without adding a concrete Keccak backend
to the default graph:

```rust
use eth::hash::Keccak256;
use eth::primitives::{Address, B256, ChainId};
use eth::verify::{
    Eip712DomainData, Eip712Field, Eip712StructType, Eip712Value,
    Eip712ValueKind, eip712_typed_data_signing_digest,
};

let types = [Eip712StructType {
    name: "Permit",
    fields: &[
        Eip712Field { name: "owner", type_name: "address" },
        Eip712Field { name: "spender", type_name: "address" },
        Eip712Field { name: "value", type_name: "uint256" },
    ],
}];
let values = [
    Eip712Value {
        name: "owner",
        value: Eip712ValueKind::Address(Address::from([0x11_u8; 20])),
    },
    Eip712Value {
        name: "spender",
        value: Eip712ValueKind::Address(Address::from([0x22_u8; 20])),
    },
    Eip712Value {
        name: "value",
        value: Eip712ValueKind::Uint64(10),
    },
];
let domain = Eip712DomainData {
    name: Some("Example"),
    version: Some("1"),
    chain_id: Some(ChainId::new(1)),
    verifying_contract: Some(Address::from([0xcc_u8; 20])),
    salt: None,
};
let mut scratch = [0_u8; 256];
let _digest = eip712_typed_data_signing_digest::<ExampleKeccak>(
    domain,
    &types,
    "Permit",
    &values,
    &mut scratch,
)?;
# #[derive(Default)]
# struct ExampleKeccak;
# impl eth::hash::Keccak256 for ExampleKeccak {
#     fn update(&mut self, input: &[u8]) { let _ = input; }
#     fn finalize(self) -> B256 { B256::from([0x33_u8; 32]) }
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

JSON-RPC typed-data parsing is available only through the opt-in
`eip712-json` feature. It uses explicit parser limits, rejects duplicate JSON
object keys, and still relies on a caller-provided Keccak backend.

```rust,ignore
use eth::verify::{Eip712JsonLimits, eip712_json_typed_data_signing_digest};

let json = r#"{
  "types": {"Permit": [{"name": "owner", "type": "address"}]},
  "primaryType": "Permit",
  "domain": {"chainId": 1},
  "message": {"owner": "0x1111111111111111111111111111111111111111"}
}"#;
let mut scratch = [0_u8; 512];
let _digest = eip712_json_typed_data_signing_digest::<ExampleKeccak>(
    json,
    Eip712JsonLimits::DEFAULT,
    &mut scratch,
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Sender Recovery

Sender recovery operates on an already constructed Ethereum signing digest.
Transaction callers should prefer the signing-hash helpers above over
hand-built transaction digests, then recover the sender with an admitted
Keccak-256 backend:

```rust
use eth::hash::Keccak256;
use eth::primitives::B256;
use eth::protocol::SignatureYParity;
use eth::verify::{
    EthereumSignature, RecoverableSecp256k1, recover_sender_from_digest_with_backend,
};

struct PlatformKeccak {
    output: B256,
}

impl Keccak256 for PlatformKeccak {
    fn update(&mut self, input: &[u8]) {
        let _ = input;
    }

    fn finalize(self) -> B256 {
        self.output
    }
}

struct PlatformSecp256k1;

impl RecoverableSecp256k1 for PlatformSecp256k1 {
    fn recover_uncompressed_public_key(
        &mut self,
        signing_digest: B256,
        signature: EthereumSignature,
    ) -> Result<[u8; 64], eth::error::VerifyError> {
        let _ = (signing_digest, signature);
        Ok([0x55_u8; 64])
    }
}

let digest = B256::from([0x44_u8; 32]);
let signature = EthereumSignature::from_parts(
    [0x11_u8; 32],
    [0x22_u8; 32],
    SignatureYParity::Even,
);

let _result = recover_sender_from_digest_with_backend(
    digest,
    signature,
    PlatformSecp256k1,
    PlatformKeccak {
        output: B256::from([0x33_u8; 32]),
    },
);
```

The recovery layer rejects malformed scalar values, high-s signatures, and
non-Ethereum recovery IDs. The example hasher above is illustrative only and
does not compute a real digest. Production hashers must implement Ethereum
Keccak-256, not FIPS SHA3-256, and should be checked with
`eth::hash::verify_empty_digest_with` before being wired into
`recover_sender_from_digest_with_backend`. A wrong secp256k1 or Keccak backend
produces a wrong sender address silently; there is no runtime cross-check. A
successful recovered address is still not a full transaction-validity proof.

## Constant-Time Composition

`B256::ct_eq` and `Wei::ct_eq` return `subtle::Choice` so compound checks can
use `&` and `|` without short-circuiting:

```rust
use eth::primitives::B256;

let block_hash = B256::from([1_u8; 32]);
let expected_block_hash = B256::from([1_u8; 32]);
let receipts_root = B256::from([2_u8; 32]);
let expected_receipts_root = B256::from([2_u8; 32]);

let valid = block_hash.ct_eq(&expected_block_hash)
    & receipts_root.ct_eq(&expected_receipts_root);

assert!(bool::from(valid));
```

Convert `Choice` to `bool` only at the final trust boundary.

## Keccak Boundary

`eth` defines a `no_std` Keccak-256 trait boundary and intentionally does not
ship a default hashing backend yet:

```rust
use eth::hash::{Keccak256, hash_one};
use eth::primitives::B256;

struct PlatformKeccak {
    output: B256,
}

impl Keccak256 for PlatformKeccak {
    fn update(&mut self, input: &[u8]) {
        let _ = input;
    }

    fn finalize(self) -> B256 {
        self.output
    }
}

let digest = hash_one(
    PlatformKeccak {
        output: B256::from([0x44_u8; 32]),
    },
    b"ethereum",
);

assert_eq!(<[u8; 32]>::from(digest), [0x44_u8; 32]);
```

Implementations must compute Ethereum Keccak-256, not FIPS SHA3-256. See
[`docs/keccak-boundary.md`](docs/keccak-boundary.md) for the dependency
decision and future backend admission checklist.

## Stable Errors

Error values expose stable codes, messages, and categories. They do not carry
input bytes, keys, signatures, or other secret-bearing payloads:

```rust
use eth::error::{DecodeError, DecodeErrorCategory, ResourceError};

let error = DecodeError::AllocationExceeded;

assert_eq!(error.code(), "ETH_CODEC_ALLOCATION_EXCEEDED");
assert_eq!(error.category(), DecodeErrorCategory::ResourceExhaustion);
assert_eq!(error.resource(), Some(ResourceError::AllocationBytes));
assert_eq!(error.to_string(), "decoder exceeded the active allocation limit");
```

## Decode Budgets

Every future untrusted decoder is required to use explicit limits. Use
`DecodeAccumulator` when more than one allocation can occur:

```rust
use eth::codec::{DecodeError, DecodeLimits};

let limits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 16,
    max_nesting_depth: 4,
    max_total_allocation: 64,
    max_proof_nodes: 8,
    max_total_items: 32,
};

assert_eq!(limits.check_input_len(512), Ok(()));

let mut budget = limits.accumulator();
assert_eq!(budget.check_allocation(32), Ok(()));
assert_eq!(budget.check_allocation(32), Ok(()));
assert_eq!(budget.check_allocation(1), Err(DecodeError::AllocationExceeded));
assert_eq!(budget.account_items(33), Err(DecodeError::ItemCountExceeded));
```

## RLP Codec

The RLP codec admits canonical byte-string scalars, lists, and Ethereum
integers with exact consumption. Decoders require explicit limits; encoders are
buffer-based and do not allocate:

```rust
use eth::codec::{
    DecodeLimits, RlpListForm, RlpScalarForm, decode_rlp_list, decode_rlp_scalar, decode_rlp_u64,
    encode_decoded_scalar, encode_rlp_list_payload, encode_rlp_scalar,
};

let limits = DecodeLimits {
    max_input_bytes: 32,
    max_list_items: 4,
    max_nesting_depth: 4,
    max_total_allocation: 32,
    max_proof_nodes: 4,
    max_total_items: 4,
};
let scalar = decode_rlp_scalar(&[0x83, b'd', b'o', b'g'], limits)?;

assert_eq!(scalar.payload(), b"dog");
assert_eq!(scalar.encoded_len(), 4);
assert_eq!(scalar.header_len(), 1);
assert_eq!(scalar.form(), RlpScalarForm::ShortString);

let mut encoded = [0_u8; 8];
let written = encode_decoded_scalar(scalar, &mut encoded)?;
assert_eq!(written, 4);
assert_eq!(encoded.get(..written), Some([0x83, b'd', b'o', b'g'].as_slice()));

assert_eq!(decode_rlp_u64(&[0x82, 0x04, 0x00], limits)?, 1024);
assert!(decode_rlp_u64(&[0x82, 0x00, 0x01], limits).is_err());

let list = decode_rlp_list(&[0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'], limits)?;

assert_eq!(list.item_count(), 2);
assert_eq!(list.form(), RlpListForm::ShortList);
let mut items = list.items();
let first = items.next().transpose()?.and_then(|item| item.as_scalar());
let second = items.next().transpose()?.and_then(|item| item.as_scalar());
assert!(matches!(first, Some(item) if item.payload() == b"cat"));
assert!(matches!(second, Some(item) if item.payload() == b"dog"));

let mut scalar_output = [0_u8; 8];
assert_eq!(encode_rlp_scalar(b"cat", &mut scalar_output)?, 4);
assert_eq!(scalar_output.get(..4), Some([0x83, b'c', b'a', b't'].as_slice()));

let list_payload = [0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
let mut list_output = [0_u8; 16];
assert_eq!(encode_rlp_list_payload(&list_payload, limits, &mut list_output)?, 9);
assert_eq!(list_output.get(..9), Some([0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'].as_slice()));
# Ok::<(), eth::error::DecodeError>(())
```

The RLP parser surface has cargo-fuzz targets and committed seed fixtures. See
[`docs/fuzzing.md`](docs/fuzzing.md) for seed materialization, target scope, and
crash reproduction.

## Withdrawals

EIP-4895 withdrawal lists decode into an explicitly unvalidated borrowed model.
The decoder checks canonical RLP shape, `uint64` indexes, 20-byte recipient
addresses, and nonzero Gwei amounts, but it does not prove header
`withdrawals_root` membership or state-balance application:

```rust
use eth::codec::DecodeLimits;
use eth::protocol::decode_withdrawals;

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 8,
    max_nesting_depth: 4,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 16,
};
let raw = [
    0xd9, 0xd8, 0x01, 0x02, 0x94, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
    0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42,
    0x43, 0x03,
];

let withdrawals = decode_withdrawals(&raw, limits)?;
let mut entries = withdrawals.entries();
let first = entries.next().transpose()?.ok_or("missing withdrawal")?;

assert_eq!(withdrawals.len(), 1);
assert_eq!(first.index.get(), 1);
assert_eq!(first.validator_index.get(), 2);
assert_eq!(first.amount.get(), 3);
assert!(entries.next().is_none());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## MPT Nodes

The verifier crate decodes Merkle Patricia Trie node shape without computing a
root. Branch nodes must contain sixteen child references plus one scalar value;
extension and leaf nodes must contain a compact hex-prefix path plus a child
reference or scalar value:

```rust
use eth::codec::DecodeLimits;
use eth::verify::{MptNode, MptNodeReference, decode_mpt_node};

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 32,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 64,
};
let raw_leaf = [0xc5, 0x20, 0x83, b'd', b'o', b'g'];

let node = decode_mpt_node(&raw_leaf, limits)?;

if let MptNode::Leaf(leaf) = node {
    assert!(leaf.path.is_leaf());
    assert_eq!(leaf.path.nibble_count()?, 0);
    assert_eq!(leaf.value, b"dog");
} else {
    assert!(false);
}

let branch = [0xd1, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
let branch = decode_mpt_node(&branch, limits)?;
if let MptNode::Branch(branch) = branch {
    assert!(branch
        .children()
        .all(|child| matches!(child, Ok(MptNodeReference::Empty))));
} else {
    assert!(false);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Transaction and receipt inclusion proofs can be checked against trusted trie
roots. The verifier derives the key as `rlp(transaction_index)`, hashes proof
nodes through the caller-provided Keccak boundary, and compares the included
value byte-for-byte:

```rust
use eth::codec::DecodeLimits;
use eth::hash::TinyKeccak256;
use eth::primitives::B256;
use eth::verify::{TransactionTrieRoot, verify_transaction_inclusion};

let limits = DecodeLimits {
    max_input_bytes: 512,
    max_list_items: 64,
    max_nesting_depth: 16,
    max_total_allocation: 1024,
    max_proof_nodes: 8,
    max_total_items: 128,
};

# let trusted_root = B256::from_bytes([0_u8; 32]);
# let encoded_transaction = [0x80_u8];
# let proof_nodes: [&[u8]; 0] = [];
let root = TransactionTrieRoot::from_b256(trusted_root);
let result = verify_transaction_inclusion(
    root,
    0,
    &encoded_transaction,
    &proof_nodes,
    limits,
    TinyKeccak256::default,
);

assert!(result.is_err());
```

Account and storage proof APIs derive keys as `keccak256(address)` and
`keccak256(slot_key)`, then compare the encoded account or storage value
byte-for-byte. They do not decode account fields, prove that a storage root
belongs to a specific account, or interpret the storage scalar. See
[`docs/mpt-nodes.md`](docs/mpt-nodes.md).

## Transaction Envelopes

The protocol crate can classify the outer transaction envelope without decoding
or validating transaction fields:

```rust
use eth::codec::DecodeLimits;
use eth::protocol::{TransactionEnvelope, decode_transaction_envelope};

let limits = DecodeLimits {
    max_input_bytes: 32,
    max_list_items: 4,
    max_nesting_depth: 4,
    max_total_allocation: 32,
    max_proof_nodes: 4,
    max_total_items: 4,
};

let envelope = decode_transaction_envelope(&[0x02, 0xc0], limits)?;

assert!(matches!(envelope, TransactionEnvelope::Typed(_)));
if let TransactionEnvelope::Typed(typed) = envelope {
    assert_eq!(u8::from(typed.transaction_type), 2);
    assert_eq!(typed.payload, &[0xc0]);
}
# Ok::<(), eth::error::TransactionEnvelopeError>(())
```

Typed payloads can be classified first, then decoded with the matching
transaction decoder. Legacy transactions can also be decoded into an explicitly
unvalidated field model:

```rust
use eth::codec::DecodeLimits;
use eth::protocol::{LegacyTransactionTo, decode_legacy_transaction};

let limits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 4,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};
let raw = [0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02];

let tx = decode_legacy_transaction(&raw, limits)?;

assert_eq!(tx.nonce.get(), 1);
assert_eq!(tx.gas_limit.get(), 21_000);
assert_eq!(tx.to, LegacyTransactionTo::Create);
assert_eq!(tx.input, &[] as &[u8]);
assert_eq!(tx.eip155_chain_id(), None);
# Ok::<(), eth::error::LegacyTransactionDecodeError>(())
```

The decoded value is not chain-valid, signature-valid, sender-recovered, or
fork-valid. It is only a bounded, canonical field parse. Use
`eip155_chain_id` instead of subtracting directly from the raw `v` signature
word; reserved `ChainId(0)` maps to `None`.

## Optional Sanitization

The main facade stays small by default. Applications that handle local secret
material can opt into the sanitization bridge:

```rust,ignore
use eth::sanitization::{SecretBytes32, SecureSanitize};

let mut key = SecretBytes32::from_array([0x42_u8; 32]);
key.secure_sanitize();
assert!(key.constant_time_eq(&[0_u8; 32]));
```

For derive macros, depend on the support crate directly:

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.7", features = ["derive"] }
```

Public RLP encode/decode derives live in `eth-valkyoth-derive`:

```toml
[dependencies]
eth-valkyoth-derive = "0.17"
eth-valkyoth-codec = "0.17"
```

The derive surface is intentionally conservative. It supports reviewed structs
only, rejects generics/enums/unions, requires `DecodeLimits` for decode, and
keeps skipped fields explicit with `#[eth_rlp(skip, default, reason = "...")]`.

## Workspace Shape

Most users should depend on the facade crate, `eth`. The support crates are
published separately so implementation boundaries stay small, `no_std`
friendly, and independently testable.

| Crate | Default | Purpose |
| --- | --- | --- |
| `eth` | yes | Facade crate over stable protocol-core crates. |
| `eth-valkyoth-primitives` | yes | Chain, fork, block, gas, nonce, address, hash, wei, and bounded value types. |
| `eth-valkyoth-codec` | yes | Bounded exact-consumption wire codec policy. |
| `eth-valkyoth-hash` | yes | Keccak-256 trait boundary for caller-provided hash implementations. |
| `eth-valkyoth-protocol` | yes | Fork-aware validation states and protocol context. |
| `eth-valkyoth-verify` | yes | Verification boundaries for signatures, proofs, replay domains, and EIP-712 typed-data hashing. |
| `eth-valkyoth-sanitization` | no | Optional bridge to the `sanitization` crate for secret-bearing Ethereum data. |
| `eth-valkyoth-derive` | no | Optional sanitization and RLP derive macros. |
| `eth-valkyoth-evm` | no | Explicit no_std EVM execution boundary; no backend admitted yet. |
| `eth-valkyoth-evm-core` | no | Dependency-free native EVM core domains plus gas-metered basic bounded opcode execution, explicit host-state reads, fail-closed call/create planning, native precompile execution through BLAKE2F, and canonical EIP-2537 BLS wire/frame parsing while arithmetic remains fail closed. |
| `eth-valkyoth-rpc` | no | Future explicit RPC trust-policy boundary. |
| `eth-valkyoth-signer` | no | Future signer isolation boundary. |
| `eth-valkyoth-reth` | no | Future Reth integration boundary. |
| `eth-valkyoth-testkit` | no | Test fixtures, conformance helpers, and adversarial inputs. |

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. New deployments should use
the pinned stable Rust `1.97.0` until the toolchain policy is updated.

Compatibility evidence for `0.52.1`:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0`-`1.96.1` | `cargo check --workspace --all-features` on every supported toolchain |
| `1.97.0` | Full release gate |

## Checks

```bash
scripts/checks.sh
scripts/release_0_52_1_gate.sh
```

For dependency-policy checks, install `cargo-deny` and `cargo-audit`, then run:

```bash
cargo deny check
cargo audit
```

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Release Plan](docs/RELEASE_PLAN.md)
- [Advanced Precompile Backends](docs/advanced-precompile-backends.md)
- [Block Headers](docs/block-headers.md)
- [Receipts](docs/receipts.md)
- [Withdrawals](docs/withdrawals.md)
- [Keccak Boundary](docs/keccak-boundary.md)
- [Transaction Signing Hashes](docs/transaction-signing-hashes.md)
- [Transaction Signature Validation](docs/transaction-signature-validation.md)
- [k256 Dependency Admission](docs/dependency-admission-k256.md)
- [Fuzzing](docs/fuzzing.md)
- [Scope](docs/SCOPE.md)
- [Threat Model](docs/threat-model.md)
- [Spec Matrix](docs/SPEC_MATRIX.md)
- [Spec Source Policy](docs/spec-source-policy.md)
- [GitHub Security Settings](docs/github-security-settings.md)
- [Secret Handling Policy](docs/secret-handling-policy.md)
- [Modularity Policy](docs/modularity-policy.md)
- [Supply-Chain Security](docs/supply-chain-security.md)
- [Unsafe Policy](docs/unsafe-policy.md)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
