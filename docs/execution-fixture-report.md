# Execution Fixture Report

Status: v0.35.0 implementation report.

Source: `ethereum/tests`

Pinned revision: `c67e485ff8b5be9abc8ad15345ec21aa22e290d9`

## Claimed Pass Set

| Fixture group | Runner | Result |
| --- | --- | --- |
| `RLPTests` | `eth-valkyoth-codec::execution_rlp_fixtures` | Passing in local conformance run |

The claimed pass set is intentionally narrow. It covers the RLP codec surface
that this crate already implements and avoids implying support for transaction,
blockchain, genesis, trie-construction, difficulty, or EOF fixtures.

## Local Evidence

The conformance run used a filtered checkout of `ethereum/tests` at the pinned
revision and executed:

```sh
scripts/run_execution_fixtures.py --execution-tests /tmp/eth-tests-layout
```

Release CI validates the fixture manifest with:

```sh
scripts/run_execution_fixtures.py --check
```
