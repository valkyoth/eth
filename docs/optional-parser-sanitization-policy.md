# Optional Parser And Sanitization Boundary Policy

Status: `v0.37.5` policy implemented, pentest remediated, and retest clean.

This document closes the optional parser and sanitization dependency slice from
the core independence audit. It records exactly when `serde`, `serde_json`,
`eth-valkyoth-sanitization`, and the external `sanitization` crate enter the
dependency graph.

## Default Facade Policy

The default `eth` facade graph must not include:

- `serde`;
- `serde_json`;
- `eth-valkyoth-sanitization`;
- `sanitization`.

Those crates are useful, but they are not part of the protocol-core default.
Default users get bounded first-party protocol parsing, typed-data encoding,
verification boundaries, and primitive domains without a JSON parser or secret
memory-clearing dependency.

## EIP-712 JSON Parser Boundary

The optional JSON parser path is:

```text
eth/eip712-json -> eth-valkyoth-verify/json -> std + serde + serde_json
```

This feature is a `std` parser boundary for JSON-RPC typed-data payloads. It
must remain opt-in because it admits parser dependencies and accepts untrusted
JSON text. The boundary keeps these constraints:

- duplicate JSON object keys are rejected;
- raw JSON DOM construction has fixed caps on object members, array items, and
  string byte length before values are retained;
- explicit typed-data parser limits cap input bytes, EIP-712 array items,
  EIP-712 string values, and typed-data structures;
- raw JSON structural depth is checked before the typed-data walker;
- `serde_json/unbounded_depth` is not admitted;
- parsing does not add a concrete Keccak or secp256k1 implementation.

Future JSON-facing APIs must either use this boundary or add a new documented
feature with its own dependency, fuzz, and pentest evidence.

## Sanitization Boundary

The optional sanitization path is:

```text
eth/sanitization -> eth-valkyoth-sanitization -> sanitization
```

This bridge is for applications that explicitly want best-effort secret
clearing under the `eth-valkyoth-*` namespace. It is not a guarantee that all
historical copies are erased. Callers still need to control logs, crash dumps,
swap, serialization, clones, and signer boundaries.

Hardening features remain explicit on `eth-valkyoth-sanitization`:

- `memory-lock`;
- `guard-pages`;
- `multi-pass-clear`;
- `cache-flush`;
- `register-scrub`;
- `zeroize-interop`.

The `hardened-only` feature remains fail-closed unless the required hardening
features are also enabled.

## Executable Gate

`scripts/check_optional_boundary_policy.py` enforces this policy by checking:

- `eth/default` remains empty;
- `eth/eip712-json` only forwards `eth-valkyoth-verify/json`;
- `eth/sanitization` only admits `eth-valkyoth-sanitization`;
- `eth-valkyoth-verify/json` explicitly requires `std`, `serde`, and
  `serde_json`;
- `serde` and `serde_json` remain optional in `eth-valkyoth-verify`;
- no manifest enables `serde_json/unbounded_depth`;
- the default graph excludes JSON parser and sanitization crates;
- the JSON graph includes `serde` and `serde_json` but excludes sanitization;
- the sanitization graph includes `eth-valkyoth-sanitization` and
  `sanitization` but excludes JSON parser crates.

The `v0.37.5` release gate captures separate default, `eip712-json`,
`sanitization`, and all-feature cargo-tree evidence under
`target/release_0_37_5_*_tree.txt`.
