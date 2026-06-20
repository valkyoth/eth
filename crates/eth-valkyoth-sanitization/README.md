# eth-valkyoth-sanitization

Optional sanitization bridge for `eth`.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.3"
```

Crates.io: <https://crates.io/crates/eth>

This package exists for users who explicitly want best-effort memory
sanitization for secret-bearing Ethereum data. It depends on
[`sanitization`](https://crates.io/crates/sanitization), so it is not part of
the default `eth` dependency graph.

```toml
[dependencies]
eth-valkyoth-sanitization = "0.3"
```

For derive macros:

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.3", features = ["derive"] }
```

The derive macros generate calls to `eth_valkyoth_sanitization::SecureSanitize`.
They do not replace review of secret ownership, copies, logging, paging, swap,
crash dumps, or compiler/runtime behavior.
