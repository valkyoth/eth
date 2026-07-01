# eth-valkyoth-derive

Optional derive macros for `eth-valkyoth-sanitization`.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.16"
```

Crates.io: <https://crates.io/crates/eth>

This package is only for users who explicitly opt into secret sanitization
ergonomics. It generates calls to `eth_valkyoth_sanitization::SecureSanitize`
and does not add runtime clearing by itself.

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.7", features = ["derive"] }
```

RLP encode/decode derives are under evaluation and are not exported by this
crate yet. The v0.16.1 prototype is private to tests and rejects generics,
enums, unions, and ambiguous `eth_rlp` field attributes until the codec trait
surface is stable.

Supported field attribute:

```rust
#[eth_sanitization(skip, reason = "non-secret label")]
```

Supported container attribute:

```rust
#[eth_sanitization(crate = "::my_sanitization_path")]
```

Enum derives are rejected because inactive variant backing bytes may retain
secret material after variant changes. Use a struct wrapper for secret-bearing
state.
