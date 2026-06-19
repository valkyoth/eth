# eth Secret Handling Policy

No secret-bearing type exists in `v0.1.0`. This policy applies before local
signing, keystore, ECDH, BLS, or hardware-secret integration is implemented.

Secret-bearing types include private keys, seed phrases, ECDH outputs, BLS
scalars, decrypted keystore bytes, signer credentials, and signing preimages
that are not already public transaction data.

Rules:

- Do not derive `Copy`.
- Do not derive revealing `Debug`.
- Do not expose ordinary `Clone` unless a security review admits it.
- Prefer external signer boundaries over local key material.
- Admit a no_std zeroization dependency before implementing local secret
  storage.
- Redact secrets from errors, logs, panic text, metrics, and test output.
- Document residual limits: zeroization cannot erase historical copies, stack
  spills, registers, swap, core dumps, or privileged memory reads.

Public values such as Ethereum addresses and block hashes are not secret-bearing
types, but hash comparisons used inside proof or authentication boundaries must
use explicit constant-time helpers where timing is security-relevant.
