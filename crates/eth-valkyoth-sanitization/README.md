<p align="center">
  <b>optional sanitization bridge for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-sanitization">Docs.rs</a>
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

# eth-valkyoth-sanitization

Optional sanitization bridge for `eth`.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.52.5"
```

Crates.io: <https://crates.io/crates/eth>

This package exists for users who explicitly want optimizer-resistant memory
clearing for secret-bearing Ethereum data. It depends on
[`sanitization`](https://crates.io/crates/sanitization), so it is not part of
the default `eth` dependency graph.

```toml
[dependencies]
eth-valkyoth-sanitization = "0.8"
```

For derive macros:

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.8", features = ["derive"] }
```

The `0.8` bridge uses `sanitization 2.0.3`. The canonical `wipe` module replaces
the removed best-effort wipe surface. The sanitization derive macros generate
calls to `eth_valkyoth_sanitization::SecureSanitize`, implement
`DropSafeSanitize` for their field-wise sanitizers, and require
`DropSafeSanitize + Unpin` for generated drop code.

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
    version = "0.8",
    features = [
        "hardened-only",
        "memory-lock",
        "multi-pass-clear",
        "cache-flush",
        "register-scrub",
    ]
}
```

The `hardened-only` feature fails compilation unless the legacy hardening
feature set is present. `HARDENING_FEATURES_ENABLED` reports only compile-time
selection; it does not prove that an OS control succeeded.

Applications that handle private keys or seeds must inspect the
`ProtectionReport` returned by protected containers and fail according to
their deployment policy. A compile-time assertion can still prevent an
accidental weak feature build:

```rust
const _: () = assert!(
    eth_valkyoth_sanitization::HARDENING_FEATURES_ENABLED,
    "enable memory-lock, multi-pass-clear, cache-flush, and register-scrub"
);
```

Ordinary owned buffers use the canonical wipe boundary:

```rust
use eth_valkyoth_sanitization::wipe;

let mut key = [0x42_u8; 32];
wipe::array(&mut key);
assert_eq!(key, [0_u8; 32]);
```

The deprecated `sanitize_bytes`,
`best_effort::sanitize_bytes_best_effort`, and `HARDENED_MODE` names remain
available so the `eth 0.52.5` patch release preserves the public facade exposed
by `eth 0.52.4`. New code should use `wipe` and
`HARDENING_FEATURES_ENABLED`; protected containers must still inspect their
runtime `ProtectionReport`.
