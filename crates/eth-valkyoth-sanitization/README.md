# eth-valkyoth-sanitization

Optional sanitization bridge for `eth`.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.8"
```

Crates.io: <https://crates.io/crates/eth>

This package exists for users who explicitly want best-effort memory
sanitization for secret-bearing Ethereum data. It depends on
[`sanitization`](https://crates.io/crates/sanitization), so it is not part of
the default `eth` dependency graph.

```toml
[dependencies]
eth-valkyoth-sanitization = "0.6"
```

For derive macros:

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.7", features = ["derive"] }
```

The derive macros generate calls to `eth_valkyoth_sanitization::SecureSanitize`.
They do not replace review of secret ownership, copies, logging, paging, swap,
crash dumps, or compiler/runtime behavior.

Enum derives are rejected because Rust does not guarantee inactive variant
backing bytes are cleared when the active variant changes. Use a struct wrapper
for secret material until a verified full-width clear primitive exists.

For private-key or seed deployments, enable and verify the hardening features
that match the target:

```toml
[dependencies]
eth-valkyoth-sanitization = {
    version = "0.7",
    features = [
        "hardened-only",
        "memory-lock",
        "multi-pass-clear",
        "cache-flush",
        "register-scrub",
    ]
}
```

The `hardened-only` feature fails compilation unless the full hardened feature
set is present. The crate also exposes `HARDENED_MODE` so applications can
assert the selected feature set in their own startup checks.

Applications that handle private keys or seeds should add a hard fail for the
expected feature set:

```rust
const _: () = assert!(
    eth_valkyoth_sanitization::HARDENED_MODE,
    "enable memory-lock, multi-pass-clear, cache-flush, and register-scrub"
);
```

Best-effort clearing helpers live under `best_effort` so the weaker guarantee
is visible at the call site.
