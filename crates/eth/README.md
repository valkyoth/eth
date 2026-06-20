# eth

`eth` is the public facade crate for this workspace.

Prefer depending on this crate instead of the lower-level `eth-valkyoth-*`
support crates:

```toml
[dependencies]
eth = "0.3"
```

Crates.io: <https://crates.io/crates/eth>

The support crates exist to keep implementation boundaries small, `no_std`
friendly, and independently testable. They are published because Cargo resolves
workspace packages as separate crates, not because they are the primary public
API.
