<p align="center">
  <b>no_std-first Ethereum protocol building blocks for Rust.</b><br>
  Explicit domains, bounded decode policy, constant-time primitives, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://docs.rs/eth">Docs.rs</a>
  |
  <a href="docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="docs/threat-model.md">Threat Model</a>
  |
  <a href="SECURITY.md">Security</a>
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

Status: `v0.12.0` legacy transaction decode implementation is complete; pentest
is pending. `v0.11.0` is the latest tagged release.

Implemented now:

- Rust workspace pinned to stable `1.96.1`.
- MSRV policy for Rust `1.90.0` through `1.96.1`.
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
- Caller-provided Keccak-256 trait boundary without a default hash
  implementation dependency.
- RLP fuzz harness with committed hex seed corpus and crash reproduction docs.
- Stable error codes, messages, categories, and formatting for codec,
  protocol, fork, feature, resource, and verification failures.
- Optional sanitization and derive support crates outside the default feature
  set.
- MIT OR Apache-2.0 license.
- Security, modularity, supply-chain, implementation, and release planning docs.
- Local check, release-gate, dependency-policy, SBOM, and pentest evidence.
- Independent support-crate release planning for crates.io push limits.

Not implemented yet:

- No RPC transport.
- No signer or local key storage.
- No EVM execution adapter.
- No Reth or P2P integration.
- No typed transaction field parsers yet.
- No transaction signature validation or sender recovery yet.
- No block parser yet.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned toolchain | Rust `1.96.1` |
| Default target | `no_std` |
| Default runtime dependencies | protocol-core support crates only |
| Optional hardening dependencies | `sanitization` and proc-macro tooling behind opt-in crates/features |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | protocol-core only |
| Network/signing defaults | none |
| Release evidence | local gates, cargo-deny, cargo-audit, SBOM, pentest report |
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready Ethereum execution-layer toolkit |

## Install

```toml
[dependencies]
eth = "0.11"
```

For optional sanitization support:

```toml
[dependencies]
eth = { version = "0.11", features = ["sanitization"] }
```

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `std` | no | Enables `std` support in admitted core crates. |
| `evm` | no | Future explicit EVM adapter boundary. |
| `rpc` | no | Future explicit RPC trust-policy boundary. |
| `sanitization` | no | Re-exports optional secret sanitization bridge APIs. |
| `signer` | no | Future signer isolation boundary. |
| `reth` | no | Future Reth integration boundary. |
| `testkit` | no | Test fixtures, conformance helpers, and adversarial inputs. |

Default builds do not enable networking, signing, local key storage, Reth, P2P,
or EVM execution.

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

Typed payloads are still opaque bytes. Legacy transactions can also be decoded
into an explicitly unvalidated field model:

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
assert_eq!(tx.input, &[]);
assert_eq!(tx.eip155_chain_id(), None);
# Ok::<(), eth::error::LegacyTransactionDecodeError>(())
```

The decoded value is not chain-valid, signature-valid, sender-recovered, or
fork-valid. It is only a bounded, canonical field parse. Use
`eip155_chain_id` instead of subtracting directly from the raw `v` signature
word.

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
| `eth-valkyoth-verify` | yes | Verification boundaries for signatures, proofs, and replay domains. |
| `eth-valkyoth-sanitization` | no | Optional bridge to the `sanitization` crate for secret-bearing Ethereum data. |
| `eth-valkyoth-derive` | no | Optional sanitization derive macros. |
| `eth-valkyoth-evm` | no | Future REVM adapter boundary. |
| `eth-valkyoth-rpc` | no | Future explicit RPC trust-policy boundary. |
| `eth-valkyoth-signer` | no | Future signer isolation boundary. |
| `eth-valkyoth-reth` | no | Future Reth integration boundary. |
| `eth-valkyoth-testkit` | no | Test fixtures, conformance helpers, and adversarial inputs. |

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. New deployments should use
the pinned stable Rust `1.96.1` until the toolchain policy is updated.

Compatibility evidence for `0.12.0`:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | `cargo check --workspace --all-features` |
| `1.91.0` | `cargo check --workspace --all-features` |
| `1.92.0` | `cargo check --workspace --all-features` |
| `1.93.0` | `cargo check --workspace --all-features` |
| `1.94.0` | `cargo check --workspace --all-features` |
| `1.95.0` | `cargo check --workspace --all-features` |
| `1.96.0` | `cargo check --workspace --all-features` |
| `1.96.1` | full release gate |

## Checks

```bash
scripts/checks.sh
scripts/release_0_12_gate.sh
scripts/validate-release-readiness.sh v0.12.0
```

For dependency-policy checks, install `cargo-deny` and `cargo-audit`, then run:

```bash
cargo deny check
cargo audit
```

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Release Plan](docs/RELEASE_PLAN.md)
- [Keccak Boundary](docs/keccak-boundary.md)
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
