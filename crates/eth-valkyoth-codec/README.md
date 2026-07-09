<p align="center">
  <b>bounded no_std Ethereum wire codec policy for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-codec">Docs.rs</a>
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

# eth-valkyoth-codec

Support crate for `eth`: bounded `no_std` Ethereum wire codec policy.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.37"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The RLP parser surface is covered by the workspace fuzz harness. See the
project fuzzing guide for target names, committed seed corpus handling, and
crash reproduction:

<https://github.com/valkyoth/eth/blob/main/docs/fuzzing.md>

The `0.17.0` release adds public `RlpEncode` and `RlpDecode` traits plus
`RlpDeriveError` for derive-generated struct encoders and decoders. Decoders
require explicit `DecodeLimits`; callers must discard encode output buffers
after any returned error.
