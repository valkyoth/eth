# Native EVM Fork Matrix

Status: `v0.43.1`.

This document describes the first-party `eth-valkyoth-evm-core` fork model.
It is a support matrix for the native engine bootstrap, not a full Ethereum
execution-validity claim.

## Fork Identifiers

| Native fork | Identifier | Protocol hardfork alignment | Current native engine status |
| --- | ---: | --- | --- |
| `EvmFork::FRONTIER` | `0` | `Hardfork::Frontier` | Recognized; basic stack/control-flow opcodes only. State execution fails closed until historical gas is implemented. |
| `EvmFork::HOMESTEAD` | `1` | `Hardfork::Homestead` | Recognized; basic stack/control-flow opcodes only. |
| `EvmFork::TANGERINE_WHISTLE` | `2` | More granular than current protocol `Hardfork` enum | Recognized; historical gas schedule planned. |
| `EvmFork::SPURIOUS_DRAGON` | `3` | More granular than current protocol `Hardfork` enum | Recognized; historical gas schedule planned. |
| `EvmFork::BYZANTIUM` | `4` | `Hardfork::Byzantium` | Recognized; `REVERT` is introduced here. |
| `EvmFork::CONSTANTINOPLE` | `5` | More granular than current protocol `Hardfork` enum | Recognized; `EXTCODEHASH` is introduced here but state execution still fails closed until historical gas is implemented. |
| `EvmFork::PETERSBURG` | `6` | More granular than current protocol `Hardfork` enum | Recognized as a Constantinople correction boundary. |
| `EvmFork::ISTANBUL` | `7` | More granular than current protocol `Hardfork` enum | Recognized; `SELFBALANCE` is introduced here but state execution still fails closed until historical gas is implemented. |
| `EvmFork::BERLIN` | `8` | More granular than current protocol `Hardfork` enum | Recognized; warm/cold state accounting begins here conceptually, but this release keeps state execution claimed only for London and later. |
| `EvmFork::LONDON` | `9` | `Hardfork::London` | Supported for the current warm/cold state-access model. |
| `EvmFork::SHANGHAI` | `10` | `Hardfork::Shanghai` | Supported for the current native engine subset. |
| `EvmFork::CANCUN` | `11` | `Hardfork::Cancun` | Supported for the current native engine subset. |
| `EvmFork::PRAGUE` | `12` | `Hardfork::Prague` | Supported for the current native engine subset. |
| `EvmFork::AMSTERDAM` | `13` | `Hardfork::Amsterdam` | Known to the roadmap but rejected by `OpcodeTable::try_new` until a concrete fork scope is admitted. |

`EvmFork::is_known()` returns true for roadmap-known forks through Amsterdam.
`EvmFork::is_supported()` returns true only through Prague in this release.

## Opcode Boundaries

The native engine now records the introduction fork for every modeled opcode:

| Opcode domain | Introduction boundary | Execution status |
| --- | --- | --- |
| Current arithmetic, bitwise, comparison, stack, memory, jump, `RETURN`, and base state opcodes | Frontier | Basic non-state execution is supported. Pre-London state execution fails closed until `v0.43.2`. |
| `REVERT` | Byzantium | Supported in the current control-flow shell. |
| `EXTCODEHASH` | Constantinople | Recognized as introduced, executable only for the currently claimed London-and-later state model. |
| `SELFBALANCE` | Istanbul | Recognized as introduced, executable only for the currently claimed London-and-later state model. |

`EvmFork::opcode_introduced_in(opcode)` exposes the introduction boundary.
`EvmFork::opcode_is_introduced(opcode)` answers whether a fork is at or after
that boundary.

`OpcodeTable::instruction(opcode)` remains stricter than historical existence:
it returns metadata only when the native engine admits that opcode for the
selected fork. This prevents old forks from accidentally using later gas
schedules.

## Deferred Historical Gas Work

`v0.43.2` is the scheduled pass for pre-Berlin state gas schedules. Until that
release implements and tests historical state pricing, state opcodes under
Frontier through Berlin fail closed rather than falling through to the
London/Berlin warm/cold constants.

This split is deliberate:

- `v0.43.1` makes the fork and opcode matrix explicit.
- `v0.43.2` fills in historical state gas where the current state-opcode subset
  is claimed.
- Later execution releases can build calls/create, precompiles, logs, refunds,
  and full fixture claims on top of a named fork matrix instead of a compressed
  "latest-like" fork model.
