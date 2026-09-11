<p align="center">
  <b>no_std Ethereum verification boundaries for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-verify">Docs.rs</a>
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

# eth-valkyoth-verify

Bounded Ethereum signature, typed-data and Merkle Patricia Trie verification
for [`eth`](https://crates.io/crates/eth). The default crate is `no_std`;
JSON parsing and concrete recovery backends are opt-in.

```sh
cargo add eth
```

## Example

Require the expected replay-protection domain before signature recovery:

```rust
use eth::primitives::{Address, ChainId};
use eth::verify::{Eip712Domain, require_eip712_domain};

let chain = ChainId::new(1);
let contract = Address::from_bytes([0_u8; 20]);
let domain = Eip712Domain::complete(chain, contract);
require_eip712_domain(chain, contract, domain)?;
# Ok::<(), eth::error::VerifyError>(())
```

This checks domain fields only, not a signature or the correctness of a supplied
domain separator. Use the typed encoder to construct the separator and message
hash before recovery.

## Current Scope

- Canonical transaction signatures and explicit secp256k1/Keccak backends.
- Bounded borrowed EIP-712 encoding/hashing; optional JSON typed-data parsing.
- MPT syntax, authenticated inclusion/absence, transaction/receipt membership,
  canonical account decoding, and account-bound storage proofs.
- Hash-addressed node resolution, snapshot-bound multiproof batches, cumulative
  output budgets and optional owned deduplicating arenas.

A trie root must come from an independent trust path, normally a separately
verified block header. Accepting root and proof from one untrusted RPC response
does not authenticate state. `VerifiedAccount` binds storage authority to the
authenticated account; absence maps to zero and explicitly stored zero is
rejected. Raw storage inclusion alone does not bind the root to an account.

Every untrusted traversal and actual hash must debit the same operation-wide
`DecodeSession`. Legacy iterators require trusted or independently bounded
inputs. Preflight rejects noncanonical nodes before invoking hash backends.

## Security Contract

Decoded transaction signature validation is still not full execution
validation. It does not itself prove fork validity, enforce fee rules, validate
account state, enforce EIP-7702 authorization chain/nonce/account-state policy,
or validate blob/KZG commitments. Use the protocol validity gate for the
non-cryptographic set-code transaction checks.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. The typed-data encoder now computes domain separators
and message hashes from borrowed descriptors. JSON-RPC typed-data parsing is
available only through the opt-in `json` feature and does not affect default
`no_std` builds. Both paths enforce the 64-type `EIP712_MAX_TYPES` ceiling.
Borrowed schemas additionally cap each struct at
`EIP712_MAX_FIELDS_PER_TYPE` (64) fields and
`EIP712_MAX_VALUES_PER_STRUCT` (64) named values. Every borrowed array
dimension is capped at `EIP712_MAX_ARRAY_ITEMS` (256), and every borrowed or
JSON operation is capped at `EIP712_MAX_VALUE_NODES` (4096) recursive value
visits, including repeated traversal through shared borrowed slices.
Borrowed operations also cap cumulative dynamic `bytes`, string, domain-name,
and domain-version hashing at `EIP712_MAX_DYNAMIC_VALUE_BYTES` (1 MiB) by
default. `Eip712Limits` and the `*_with_limits` entry points allow a stricter
deployment policy. Unsupported atomic-looking spellings and undefined array
base types are rejected during schema validation even when the supplied array
is empty. Schema validation runs once per public operation, dependency
discovery visits reachable types once before canonical ordering, and recursive
hashing reuses a fixed 64-entry type-hash cache.
Borrowed signing values are neither `Copy` nor `Clone`, and their manual
`Debug` implementations redact all payload contents.


The optional JSON path requires `std`, rejects duplicate keys and enforces
parser limits. Do not enable `serde_json/unbounded_depth`.

See [EIP-712](https://github.com/valkyoth/eth/blob/main/docs/eip712-domain-safety.md)
and the [specification matrix](https://github.com/valkyoth/eth/blob/main/docs/SPEC_MATRIX.md).

## License

MIT OR Apache-2.0, at your option.
