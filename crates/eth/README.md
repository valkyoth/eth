<p align="center">
  <b>no_std-first Ethereum protocol building blocks for Rust.</b><br>
  Explicit domains, bounded decode policy, constant-time primitives, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://docs.rs/eth">Docs.rs</a>
  |
  <a href="https://github.com/eldryoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/eldryoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/eldryoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/eldryoth/eth">
    <img src="https://raw.githubusercontent.com/eldryoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth

`eth` is the public facade crate for a `no_std`-first Ethereum
execution-layer protocol workspace.

The crate is intentionally conservative at `0.3.0`: it provides explicit
Ethereum primitive domains, bounded decode-budget policy, small first-party
crate boundaries, optional sanitization support, and release evidence before
RPC, signer, EVM, Reth, or P2P integrations become real dependencies.

## Current Status

The current release candidate is `0.3.0`.

Implemented now:

- `no_std` facade with small first-party support crates.
- Ethereum domain newtypes for chain, block, gas, nonce, timestamp, address,
  hash, wei, and transaction type values.
- Constant-time equality composition for fixed-width hash and wei values.
- Bounded decode limits plus stateful cumulative allocation accounting.
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
| License | `EUPL-1.2` |
| MSRV | Rust `1.90.0` |
| Latest verified stable | Rust `1.96.0` |
| Default target | `no_std` |
| Default features | protocol-core only |
| Default networking/signing | none |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Release evidence | local gates, cargo-deny, cargo-audit, SBOM, pentest report |

## Install

```toml
[dependencies]
eth = "0.3"
```

Disable defaults explicitly for embedded or freestanding builds:

```toml
[dependencies]
eth = { version = "0.3", default-features = false }
```

Optional sanitization support:

```toml
[dependencies]
eth = { version = "0.3", features = ["sanitization"] }
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
};

assert_eq!(limits.check_input_len(512), Ok(()));

let mut budget = limits.accumulator();
assert_eq!(budget.check_allocation(32), Ok(()));
assert_eq!(budget.check_allocation(32), Ok(()));
assert_eq!(budget.check_allocation(1), Err(DecodeError::AllocationExceeded));
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
eth-valkyoth-sanitization = { version = "0.3", features = ["derive"] }
```

## Support Crates

Most users should depend on `eth`. The `eth-valkyoth-*` crates are published so
the workspace can keep small, auditable boundaries:

| Crate | Default | Purpose |
| --- | --- | --- |
| `eth-valkyoth-primitives` | yes | Chain, block, gas, nonce, address, hash, wei, and transaction type domains. |
| `eth-valkyoth-codec` | yes | Bounded exact-consumption wire decoding policy. |
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

Compatibility evidence for `0.3.0`:

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

Licensed under the European Union Public Licence 1.2.
