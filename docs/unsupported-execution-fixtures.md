# Unsupported Execution Fixtures

Status: v0.35.0 unsupported fixture list.

`eth` does not claim broad Ethereum execution-test compatibility yet. `v0.41.0`
adds a local deterministic vector suite for the first basic native opcode
subset, but official state-test groups remain explicitly unsupported until the
matching protocol layers exist:

| Fixture group | Reason |
| --- | --- |
| `TransactionTests` | Transaction semantic validation is scheduled for `v0.72.0`, with complete fixture admission at `v0.87.0`. |
| `BlockchainTests` | Header/block validity, state transition, output/root handling, and complete fixture admission are scheduled across `v0.73.0..=v0.75.0` and `v0.87.0`. |
| `GenesisTests` | Genesis state construction and chain configuration import are scheduled for `v0.71.0`. |
| `TrieTests` | MPT proof verification exists, but generic trie construction and root computation are scheduled for `v0.76.0`. |
| `DifficultyTests` | Difficulty and post-Merge terminal validation rules are scheduled for `v0.73.0`. |
| `EOFTests` | EOF format validation, execution, deployment, and complete fixture admission are scheduled for `v0.82.0..=v0.87.0`. |

Adding support for a fixture group requires updating
[`conformance/execution-fixtures.toml`](../conformance/execution-fixtures.toml),
this document, the release notes, and the verification command.
