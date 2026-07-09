# Native EVM Fork Matrix

Status: `v0.50.6`.

This document describes the first-party `eth-valkyoth-evm-core` fork model.
It is a support matrix for the native engine bootstrap, not a full Ethereum
execution-validity claim.

## Fork Identifiers

| Native fork | Identifier | Protocol hardfork alignment | Current native engine status |
| --- | ---: | --- | --- |
| `EvmFork::FRONTIER` | `0` | `Hardfork::Frontier` | Supported for the current basic opcode and bounded state-read subset with Frontier state-read gas. |
| `EvmFork::HOMESTEAD` | `1` | `Hardfork::Homestead` | Supported for the current basic opcode and bounded state-read subset with Frontier-equivalent state-read gas. |
| `EvmFork::TANGERINE_WHISTLE` | `2` | More granular than current protocol `Hardfork` enum | Supported for the current state-read subset with EIP-150 IO repricing. |
| `EvmFork::SPURIOUS_DRAGON` | `3` | More granular than current protocol `Hardfork` enum | Supported for the current state-read subset with EIP-150 IO repricing. |
| `EvmFork::BYZANTIUM` | `4` | `Hardfork::Byzantium` | Recognized; `REVERT` is introduced here. |
| `EvmFork::CONSTANTINOPLE` | `5` | More granular than current protocol `Hardfork` enum | Supported for the current state-read subset; `EXTCODEHASH` is introduced at 400 gas. |
| `EvmFork::PETERSBURG` | `6` | More granular than current protocol `Hardfork` enum | Supported as a Constantinople correction boundary for the current state-read subset. |
| `EvmFork::ISTANBUL` | `7` | More granular than current protocol `Hardfork` enum | Supported for the current state-read subset; EIP-1884 pricing and `SELFBALANCE` are admitted. |
| `EvmFork::BERLIN` | `8` | More granular than current protocol `Hardfork` enum | Supported for the current state-read subset; EIP-2929 warm/cold state accounting begins here. |
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
| Current arithmetic, bitwise, comparison, stack, memory, jump, `RETURN`, and base state opcodes | Frontier | Basic execution and bounded state reads are supported with fork-specific gas for the current subset. |
| `CREATE`, `CALL`, `CALLCODE` | Frontier | Recognized and stack/memory/policy validated, then fail closed with `CallCreateExecutionUnsupported`. |
| `DELEGATECALL` | Homestead | Recognized and stack/memory/policy validated, then fail closed with `CallCreateExecutionUnsupported`. |
| `REVERT` | Byzantium | Supported in the current control-flow shell. |
| `STATICCALL` | Byzantium | Recognized with static-frame policy, then fail closed with `CallCreateExecutionUnsupported`. |
| `EXTCODEHASH` | Constantinople | Supported with Constantinople/Petersburg pricing and later Istanbul/Berlin repricing. |
| `CREATE2` | Constantinople | Recognized and stack/memory/policy validated, then fail closed with `CallCreateExecutionUnsupported`. |
| `SELFBALANCE` | Istanbul | Supported at EIP-1884 `GasFastStep` pricing. |

`EvmFork::opcode_introduced_in(opcode)` exposes the introduction boundary.
`EvmFork::opcode_is_introduced(opcode)` answers whether a fork is at or after
that boundary.

`OpcodeTable::instruction(opcode)` returns metadata only when the modeled
opcode exists at the selected fork. The interpreter and gas schedule still fail
closed for unimplemented dispatcher arms.

## Historical State Gas

`v0.43.2` claims historical gas only for the currently executable state-read
subset:

| Fork range | `BALANCE` | `EXTCODESIZE` / `EXTCODECOPY` | `EXTCODEHASH` | `SLOAD` | Access tracking |
| --- | ---: | ---: | ---: | ---: | --- |
| Frontier through Homestead | 20 | 20 | not introduced | 50 | none |
| Tangerine Whistle through Byzantium | 400 | 700 | not introduced | 200 | none |
| Constantinople through Petersburg | 400 | 700 | 400 | 200 | none |
| Istanbul | 700 | 700 | 700 | 800 | none |
| Berlin and later supported forks | cold 2600 / warm 100 | cold 2600 / warm 100 | cold 2600 / warm 100 | cold 2100 / warm 100 | Berlin warm/cold sets |

This split is deliberate:

- `v0.43.1` makes the fork and opcode matrix explicit.
- `v0.43.2` fills in historical state gas where the current state-read subset
  is claimed.
- Later execution releases can build calls/create, precompiles, logs, refunds,
  and full fixture claims on top of a named fork matrix instead of a compressed
  "latest-like" fork model.

`SSTORE` remains a write shell. The engine charges the fork-specific storage
access precharge before returning `StateWriteUnsupported`; full net metering,
refunds, committed writes, and journal interaction are scheduled for later
state-write releases.

## Call/Create Safety Boundary

`v0.44.0` admits call/create metadata and planning without admitting nested
execution. The interpreter validates call/create stack shape, memory ranges,
fork introduction boundaries, static-frame value/create restrictions, and
call-depth policy. After validation, it returns
`CallCreateExecutionUnsupported` without popping operands, performing host
calls, or committing state.

The exported policy domains are:

- `EvmCallFramePolicy` for depth, static-frame, value-transfer, and create
  admission;
- `EvmCallPlan` and `EvmCreatePlan` for parsed call/create metadata;
- `EvmReturnDataRange` for bounded return-data copy validation;
- `EvmJournal` and `EvmJournalCheckpoint` for explicit LIFO commit/revert
  policy.

This is intentionally narrower than full call execution. Later releases must
wire nested execution, gas forwarding/stipends, account creation, value
transfer, returndata copying, and journaled state writes before the native EVM
claims call/create execution compatibility.

## Precompile Registry Boundary

`v0.45.0` adds fork-aware precompile descriptors and bounded precompile plans.
`v0.46.0` adds dependency-free SHA-256 and RIPEMD-160 execution. `v0.47.0`
adds ECRECOVER execution through caller-provided secp256k1 and Keccak backend
traits. `v0.48.0` adds bounded first-party ModExp parsing, EIP-198/EIP-2565
gas, execution, and fuzzing with an explicit operand cap. `v0.49.0` adds
first-party BN254 add/mul execution with canonical field and point validation.
`v0.50.0` adds the BN254 pairing frame boundary with empty-input execution and
G2 curve validation. `v0.50.1` adds G2 subgroup validation, `v0.50.2` adds the
Fp6/Fp12 tower foundation, `v0.50.3` adds the validated tuple stream plus
the atomic gas-meter charging, `v0.50.4` adds line-function helpers,
`v0.50.5` adds internal Miller-loop accumulation, `v0.50.6` adds sparse
line-factor multiplication and benchmark evidence, and `v0.50.7` adds bounded
final exponentiation while non-empty pairing algebra remains fail-closed.
The registry recognizes the
canonical low-address accounts for Frontier precompiles,
Byzantium modular exponentiation and BN254 precompiles, Istanbul BLAKE2F,
Cancun KZG point evaluation, and Prague BLS12-381 precompiles.

| Precompile domain | Address range | First admitted native fork | Execution status |
| --- | ---: | --- | --- |
| `ecrecover`, SHA-256, RIPEMD-160, identity | `0x01..=0x04` | Frontier | Identity, SHA-256, and RIPEMD-160 execute dependency-free. ECRECOVER executes only with caller-provided secp256k1 and Keccak backends. |
| Modular exponentiation | `0x05` | Byzantium | Executes through bounded first-party no-alloc bigint code with EIP-198 and EIP-2565 gas formulas and an explicit release operand cap. |
| BN254 add/mul/pairing | `0x06..=0x08` | Byzantium | Add and scalar multiplication execute dependency-free with canonical field and point validation. Pairing validates bounded frames, G2 curve membership, G2 subgroup membership, tuple streaming, line-function arithmetic, sparse Miller-loop accumulation, and bounded final exponentiation, and executes empty input; non-empty pairing algebra fails closed until `v0.50.8`/`v0.50.9` complete the optimal-ate post-loop terms and result admission documented in `docs/bn254-pairing-economics.md`. |
| BLAKE2F | `0x09` | Istanbul | Exact 213-byte input planning and round-count gas extraction; execution fails closed without a backend. |
| KZG point evaluation | `0x0a` | Cancun | Exact 192-byte input planning and fixed 50,000 gas; proof verification backend is deferred. |
| BLS12-381 precompiles | `0x0b..=0x11` | Prague | Address/fork admission only until audited BLS backends and vectors are added. |

The registry is still intentionally narrower than a full precompile executor.
Plans enforce the release input ceiling before gas calculation, expose fixed
output sizes where known, and return `PrecompileBackendUnavailable` for
remaining cryptographic execution until reviewed backend crates or first-party
implementations are admitted. ECRECOVER intentionally accepts high-s
signatures because EIP-2 changed transaction validity but left the recover
precompile unchanged. ModExp intentionally rejects operands above the release
cap until larger first-party bigint execution is separately reviewed.

Future interpreter dispatch must preserve the fail-closed boundary: a
`PrecompileBackendUnavailable` result from a planned precompile is a reverting
precompile call, never success or a no-op. Dispatch must also charge the
precompile gas before invoking validation or execution for expensive paths.
The dispatcher-facing plan methods for ModExp, BN254 add/mul, and BN254
pairing require a mutable gas meter and charge before crate-internal low-level
helpers are reached.
