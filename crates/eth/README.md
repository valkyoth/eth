<p align="center">
  <b>no_std-first Ethereum protocol building blocks for Rust.</b><br>
  Explicit domains, bounded decode policy, constant-time primitives, and security-gated release evidence.
</p>

<div align="center">
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

`eth` is the public facade crate for a `no_std`-first Ethereum
execution-layer protocol workspace.

The crate is intentionally conservative at `0.9.0`: it provides explicit
Ethereum primitive domains, bounded decode-budget policy, stable error
categories, small first-party crate boundaries, optional sanitization support,
and release evidence before RPC, signer, EVM, Reth, or P2P integrations become
real dependencies.

## Current Status

The current release candidate is `0.9.0`.

Implemented now:

- `no_std` facade with small first-party support crates.
- Ethereum domain newtypes for chain, block, gas, nonce, timestamp, address,
  hash, wei, and transaction type values.
- Constant-time equality composition for fixed-width hash and wei values.
- Bounded decode limits plus stateful cumulative allocation, item, and proof-node
  accounting.
- Canonical RLP scalar, list, and integer decoding plus no-allocation canonical
  encoding helpers.
- Stable error codes, messages, categories, and formatting for codec,
  protocol, fork, feature, resource, and verification failures.
- Optional sanitization bridge and derive macros outside the default feature
  set.
- Release gates for formatting, clippy, tests, packaging, MSRV compatibility,
  dependency policy, audit, SBOM, and pentest evidence.

Not implemented yet:

- No RPC transport.
- No signer or local key storage.
- No EVM execution adapter.
- No Reth or P2P integration.
- No transaction or block parser yet.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Latest verified stable | Rust `1.96.0` |
| Default target | `no_std` |
| Default features | protocol-core only |
| Default networking/signing | none |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Release evidence | local gates, cargo-deny, cargo-audit, SBOM, pentest report |
| Crate versions | tracked in the [version matrix](https://github.com/valkyoth/eth/blob/main/docs/CRATE_VERSION_MATRIX.md) |

## Install

```toml
[dependencies]
eth = "0.9"
```

Disable defaults explicitly for embedded or freestanding builds:

```toml
[dependencies]
eth = { version = "0.9", default-features = false }
```

Optional sanitization support:

```toml
[dependencies]
eth = { version = "0.9", features = ["sanitization"] }
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

Legacy transactions are not typed EIP-2718 envelopes. Use
`TransactionType::LEGACY` for APIs that need a legacy domain value, and
`try_new_typed` for type bytes that will be encoded as typed envelopes.

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

## Support Crates

Most users should depend on `eth`. The `eth-valkyoth-*` crates are published so
the workspace can keep small, auditable boundaries:

| Crate | Default | Purpose |
| --- | --- | --- |
| `eth-valkyoth-primitives` | yes | Chain, block, gas, nonce, address, hash, wei, and transaction type domains. |
| `eth-valkyoth-codec` | yes | Bounded exact-consumption wire codec policy. |
| `eth-valkyoth-protocol` | yes | Fork-aware validation states and protocol context. |
| `eth-valkyoth-verify` | yes | Verification boundaries for signatures, proofs, and replay domains. |
| `eth-valkyoth-sanitization` | no | Optional bridge to the `sanitization` crate. |
| `eth-valkyoth-derive` | no | Optional sanitization derive macros. |
| `eth-valkyoth-evm` | no | Future EVM adapter boundary. |
| `eth-valkyoth-rpc` | no | Future RPC trust-policy boundary. |
| `eth-valkyoth-signer` | no | Future signer isolation boundary. |
| `eth-valkyoth-reth` | no | Future Reth integration boundary. |
| `eth-valkyoth-testkit` | no | Future fixtures and conformance helpers. |

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. New deployments should use
the latest stable Rust verified by the release gates.

Compatibility evidence for `0.9.0`:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | `cargo check --workspace --all-features` |
| `1.91.0` | `cargo check --workspace --all-features` |
| `1.92.0` | `cargo check --workspace --all-features` |
| `1.93.0` | `cargo check --workspace --all-features` |
| `1.94.0` | `cargo check --workspace --all-features` |
| `1.95.0` | `cargo check --workspace --all-features` |
| `1.96.0` | full release gate |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
