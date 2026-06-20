# eth-valkyoth-derive

Optional derive macros for `eth-valkyoth-sanitization`.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.3"
```

Crates.io: <https://crates.io/crates/eth>

This package is only for users who explicitly opt into secret sanitization
ergonomics. It generates calls to `eth_valkyoth_sanitization::SecureSanitize`
and does not add runtime clearing by itself.

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.3", features = ["derive"] }
```

Supported field attribute:

```rust
#[eth_sanitization(skip)]
```

Supported container attribute:

```rust
#[eth_sanitization(crate = "::my_sanitization_path")]
```

Enums must also acknowledge that inactive variant backing bytes are not cleared
by matching the active variant:

```rust
#[eth_sanitization(enum_inactive_variant_bytes = "acknowledged")]
```
