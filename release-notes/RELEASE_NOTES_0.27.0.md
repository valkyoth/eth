# eth 0.27.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.27.0` admits the first optional first-party software Keccak-256 backend.

The default crate graph remains implementation-neutral. Applications that want
the reviewed software backend can opt into:

```toml
[dependencies]
eth = { version = "0.27", features = ["keccak-tiny"] }
```

## Added

- Added `eth-valkyoth-hash` feature `tiny-keccak`.
- Added facade feature `keccak-tiny`.
- Added `TinyKeccak256`, available only when the backend feature is enabled.
- Added public `KECCAK256_ABC` alongside the existing `KECCAK256_EMPTY`.
- Added backend conformance tests for empty input, `abc`, and chunk-boundary
  independence.

## Security Notes

- `tiny-keccak` is not in the default feature graph.
- The admitted dependency is `tiny-keccak 2.0.2` with default features disabled
  and only its `keccak` feature enabled.
- `tiny-keccak` is licensed `CC0-1.0`; `deny.toml` now explicitly admits that
  license.
- `TinyKeccak256` does not claim a documented sponge-state zeroization
  contract. Deployments that require hasher state clearing should continue to
  provide a custom hasher with an explicit sanitization contract.
- The backend is Ethereum Keccak-256, not FIPS SHA3-256, and is checked with
  KATs that distinguish those functions.

## Dependency Review

- `tiny-keccak 2.0.2` was checked as the current crates.io version on
  2026-07-02.
- `cargo info tiny-keccak` reports default features empty and explicit
  `keccak` support.
- `crunchy 0.2.4` is pulled transitively by `tiny-keccak`.
- `cargo deny check` and `cargo audit` must pass before tagging.

## Versioning

- `eth-valkyoth-hash` publishes as `0.11.0`.
- The facade crate publishes as `eth` `0.27.0`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest must pass before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-hash --all-features
cargo clippy -p eth-valkyoth-hash -p eth --all-targets --all-features -- -D warnings
scripts/release_0_27_gate.sh
```
