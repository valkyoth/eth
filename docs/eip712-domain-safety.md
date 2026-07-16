# EIP-712 Domain Safety

Status: v0.27.0 optional JSON typed-data parser boundary has fuzz build
coverage, committed JSON seeds, and a raw JSON structural-depth regression.

EIP-712 structured-data signing extends Ethereum signing with
`keccak256("\x19\x01" || domainSeparator || hashStruct(message))`. The
`domainSeparator` is itself a hash of the `EIP712Domain` struct.

In `v0.21.0`, this workspace did not implement a typed-data encoder. It only
provided a safety boundary for the fields that are most likely to cause replay
or domain confusion when a caller already has a domain separator and message
hash:

- `chainId` must be present and match the expected execution chain;
- `verifyingContract` must be present and match the expected verifier address;
- wrong-domain failures happen before sender recovery is attempted;
- the signing digest helper applies the EIP-191 `0x1901` prefix before hashing.

In `v0.26.0`, `eth-valkyoth-verify` adds a no-allocation typed-data encoder
over caller-provided borrowed descriptors:

- `encode_eip712_type` writes canonical `encodeType(primaryType)` bytes into a
  caller scratch buffer;
- `encode_eip712_data` encodes admitted field values as 32-byte EIP-712 words;
- `eip712_hash_struct` computes `hashStruct`;
- `eip712_domain_separator` computes the `EIP712Domain` separator for admitted
  optional fields;
- `eip712_typed_data_signing_digest` computes the final `0x1901` digest from
  domain data, type descriptors, and message values.

In `v0.26.1`, `eth-valkyoth-verify` adds an optional `json` feature for
JSON-RPC typed-data payloads. The feature is disabled by default, requires
`std`, and keeps concrete Keccak backends caller-provided.

The `v0.52.0` security hardening applies a shared 64-type ceiling to the
borrowed and JSON paths. Reachability is collected with a fixed-capacity
visited set, so shared dependency DAGs are not recursively rediscovered for
each candidate type. Canonical dependencies are still emitted in lexical
order after the bounded traversal. Signing value types no longer implement
`Copy` or `Clone`, and their manual `Debug` output always redacts payloads.

The `v0.52.1` pentest hardening makes the signing schema fail closed before
hashing:

- struct and field names must use EIP-712 identifier syntax;
- unsupported aliases and atomic-looking names such as `uint`, `int`, invalid
  integer/byte widths, and fixed-point spellings cannot be custom structs;
- borrowed schemas reject duplicate struct and field names;
- borrowed values reject duplicate names at every nested struct level;
- borrowed structs admit at most 64 fields and 64 named values before the
  bounded duplicate checks run;
- borrowed arrays admit at most 256 elements at every dimension before
  element hashing starts;
- borrowed and JSON operations admit at most 4,096 recursive value visits,
  including repeated visits through shared borrowed slices;
- borrowed and JSON recursive hashing validate the schema once at the public
  boundary and reuse a fixed 64-entry type-hash cache;
- the optional JSON path applies the same identifier validation before graph
  traversal;
- failed `encode_eip712_data` calls clear the selected output region if any
  later field fails after earlier words were written.

The JSON boundary:

- rejects duplicate JSON object keys before any type map is accepted, using an
  independent object-width guard in addition to the input-byte limit;
- relies on `serde_json`'s default recursion guard while constructing the first
  JSON DOM; this is a security control and the crate must not enable
  `serde_json/unbounded_depth`;
- enforces explicit limits for input bytes, type count, field count, array
  length, dynamic bytes, string length, and recursion depth;
- rejects `ChainId(0)` in parsed EIP-712 domains;
- validates the shape of `types.EIP712Domain` even though the actual domain
  separator is built from the parsed `domain` object;
- parses signed decimal `intN` strings through the full 256-bit integer path
  before the typed encoder enforces the requested signed width;
- maps parsed type strings to the same typed-data encoder model used by the
  borrowed API;
- includes Ether Mail and adversarial parser-limit, malformed-hex,
  duplicate-field, fixed-array mismatch, domain-validation, and raw JSON
  structural-depth fixtures.
