<p align="center">
  <b>Ethereum conformance testkit boundary for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-testkit">Docs.rs</a>
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

# eth-valkyoth-testkit

Corpus-revision metadata for
[`eth`](https://crates.io/crates/eth), available through an optional feature.

```sh
cargo add eth --features testkit
```

## Example

```rust
use eth::testkit::CorpusRevision;

let corpus = CorpusRevision::new("application-fixtures", "reviewed-revision");
assert_eq!(corpus.name, "application-fixtures");
assert_eq!(corpus.revision, "reviewed-revision");
```

This package records names and revisions; it does not download, authenticate
or execute a corpus. The repository's fixture importers, independent oracles,
fuzz targets and client differential harnesses are development tools, not
features of this marker API.

See [fixture testing](https://github.com/valkyoth/eth/blob/main/docs/execution-fixture-harness.md)
and [differential testing](https://github.com/valkyoth/eth/blob/main/docs/differential-test-harness.md).

## License

MIT OR Apache-2.0, at your option.
