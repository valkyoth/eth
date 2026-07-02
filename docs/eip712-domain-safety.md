# EIP-712 Domain Safety

Status: v0.26.0 typed-data encoder implemented, pentested, and ready for the
final GitHub/tag gate.

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

The encoder intentionally does not parse JSON-RPC typed-data payloads. Callers
must parse JSON at their boundary, review the resulting type graph and values,
then pass bounded borrowed descriptors into this crate.
