<p align="center">
  <b>future signer isolation boundary for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-signer">Docs.rs</a>
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

# eth-valkyoth-signer

Optional signer-identity boundary for
[`eth`](https://crates.io/crates/eth). Default facade builds do not include it.

```sh
cargo add eth --features signer
```

## Example

Expose a public address without adding private-key storage:

```rust
use eth::primitives::Address;
use eth::signer::SignerIdentity;

struct PublicIdentity(Address);
impl SignerIdentity for PublicIdentity {
    fn address(&self) -> Address { self.0 }
}
let identity = PublicIdentity(Address::from_bytes([0_u8; 20]));
assert_eq!(identity.address(), Address::from_bytes([0_u8; 20]));
```

`SignerIdentity` reports an address; it does not prove control of it. This
package does not yet sign transactions, store keys, operate a wallet, contact a
hardware signer, or sanitize secrets automatically. Signature validation and
recovery are separate verification APIs. Private-key ownership and signing are
assigned to later [releases](https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md).

## License

MIT OR Apache-2.0, at your option.
