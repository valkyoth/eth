# EIP-712 Domain Safety

Status: v0.21.0 implementation ready for pentest.

EIP-712 structured-data signing extends Ethereum signing with
`keccak256("\x19\x01" || domainSeparator || hashStruct(message))`. The
`domainSeparator` is itself a hash of the `EIP712Domain` struct.

This workspace does not implement a full typed-data encoder in `v0.21.0`.
Instead, `eth-valkyoth-verify` provides a safety boundary for the fields that
are most likely to cause replay or domain confusion when a caller already has a
domain separator and message hash:

- `chainId` must be present and match the expected execution chain;
- `verifyingContract` must be present and match the expected verifier address;
- wrong-domain failures happen before sender recovery is attempted;
- the signing digest helper applies the EIP-191 `0x1901` prefix before hashing.

Callers remain responsible for computing `domainSeparator` and
`hashStruct(message)` with a conformant EIP-712 encoder. The helper APIs do not
parse JSON-RPC typed-data payloads, validate field order, validate type graphs,
or prove that the supplied domain separator was derived from the checked
`Eip712Domain` view.

Future EIP-712 work should add a typed-data encoder only behind the same
bounded, no-allocation, fuzzed approach used for RLP and transaction decoding.
