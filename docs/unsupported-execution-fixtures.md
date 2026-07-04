# Unsupported Execution Fixtures

Status: v0.35.0 unsupported fixture list.

`eth` does not claim broad Ethereum execution-test compatibility yet. The
following upstream fixture groups remain explicitly unsupported until the
matching protocol layers exist:

| Fixture group | Reason |
| --- | --- |
| `TransactionTests` | Transaction semantic validation remains staged after syntactic typed transaction decode and signature validation. |
| `BlockchainTests` | Full block execution, EVM state transition, and consensus header validation are not implemented yet. |
| `GenesisTests` | Genesis state construction and chain configuration import are not implemented yet. |
| `TrieTests` | MPT proof verification exists, but generic trie construction fixtures are not admitted yet. |
| `DifficultyTests` | Difficulty and post-Merge terminal validation rules are not implemented yet. |
| `EOFTests` | EOF/EVM bytecode validation is scheduled with optional execution support. |

Adding support for a fixture group requires updating
[`conformance/execution-fixtures.toml`](../conformance/execution-fixtures.toml),
this document, the release notes, and the verification command.
