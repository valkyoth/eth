# eth 0.3.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.3.0` makes the core Ethereum primitive domains explicit before parser,
transaction, signer, RPC, or EVM integration work expands the public API.

## Included

- Add explicit chain, block, gas, nonce, timestamp, address, hash, wei, and
  transaction type domains.
- Add bounded constructors where protocol limits exist.
- Add optional `eth-valkyoth-sanitization` bridge crate for users who opt into
  best-effort secret memory clearing.
- Add optional `eth-valkyoth-derive` macros for explicit sanitization users.
- Address v0.3.0 pentest findings for constant-time equality composition,
  cumulative decode allocation accounting, enum sanitization acknowledgement,
  typed transaction disambiguation, release-script `--no-verify` handling,
  GitHub Actions pin checks, and spec-lock readiness gates.
- Add constructor and conversion tests for all primitive domains.

## Verification

```bash
scripts/checks.sh
scripts/release_0_3_gate.sh
cargo deny check
cargo audit
```
