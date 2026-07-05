# Unsupported Execution Fixtures

Status: v0.35.0 unsupported fixture list.

`eth` does not claim broad Ethereum execution-test compatibility yet. The
following upstream fixture groups remain explicitly unsupported until the
matching protocol layers exist:

| Fixture group | Reason |
| --- | --- |
| `TransactionTests` | Transaction semantic validation is scheduled for `v0.49.0`. |
| `BlockchainTests` | Full block validity and state transition coverage is scheduled across `v0.50.0`, `v0.51.0`, and `v0.55.0`. |
| `GenesisTests` | Genesis state construction and chain configuration import are scheduled for `v0.48.0`. |
| `TrieTests` | MPT proof verification exists, but generic trie construction and root computation are scheduled for `v0.53.0`. |
| `DifficultyTests` | Difficulty and post-Merge terminal validation rules are scheduled for `v0.50.0`. |
| `EOFTests` | EOF/EVM bytecode validation is scheduled with native EVM execution and full execution fixture admission through `v0.40.0` through `v0.55.0`. |

Adding support for a fixture group requires updating
[`conformance/execution-fixtures.toml`](../conformance/execution-fixtures.toml),
this document, the release notes, and the verification command.
