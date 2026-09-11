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

Opt-in optimizer-resistant clearing for secret-bearing Ethereum data. This
bridge depends on [`sanitization`](https://crates.io/crates/sanitization)
and is absent from the default [`eth`](https://crates.io/crates/eth) graph.

Use the facade for basic wiping:

```sh
cargo add eth --features sanitization
```

## Example

The bytes below are demonstration data, not a private key:

```rust
use eth::sanitization::wipe;

let mut bytes = *b"public demonstration bytes";
wipe::array(&mut bytes);
assert!(bytes.iter().all(|byte| *byte == 0));
```

For optional struct derives, depend directly on the bridge:

```sh
cargo add eth-valkyoth-sanitization --features derive
```

```rust
#[derive(eth_valkyoth_sanitization::SecureSanitize, eth_valkyoth_sanitization::SecureSanitizeOnDrop)]
struct OwnedBytes {
    bytes: [u8; 8],
}
use eth_valkyoth_sanitization::SecureSanitize;
let mut value = OwnedBytes { bytes: *b"example!" };
value.secure_sanitize();
assert!(value.bytes.iter().all(|byte| *byte == 0));
```

The bridge uses the `sanitization 2.1` API. `wipe` is the canonical clearing
boundary. Field-wise derives implement `DropSafeSanitize`; generated drop
requires `DropSafeSanitize + Unpin`. Enums are rejected because inactive
variant backing storage may retain previous secrets.

## Hardening

For a target that supports these controls:

```sh
cargo add eth-valkyoth-sanitization --features hardened-only,memory-lock,multi-pass-clear,cache-flush,register-scrub
```

`hardened-only` rejects a build without the required hardening feature set.
`HARDENING_FEATURES_ENABLED` reports compile-time selection, **not successful
OS protection**. Applications handling keys or seeds must inspect the
`ProtectionReport` returned by protected containers and fail according to
their deployment policy. Review target support before selecting additional
guard-page, canary, fork-exclusion or interoperability features.

Clearing does not replace review of ownership, copies, logs, paging, swap,
crash dumps or compiler/runtime behavior. Legacy `sanitize_bytes`,
`best_effort::sanitize_bytes_best_effort` and `HARDENED_MODE` names remain
compatibility aliases; new code should use `wipe` and the explicit runtime
protection report.

## License

MIT OR Apache-2.0, at your option.
