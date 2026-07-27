# Execution Resource Governor

Status: `v0.53.0` release candidate; pentest findings are remediated and the
clean retest passed.

## Purpose

Ethereum gas bounds consensus-visible execution, but it does not automatically
bound every host-side data structure or unit of integration work. This release
therefore separates two responsibilities:

- `EvmAccessTracker` supplies consensus warmth semantics;
- `ExecutionGovernor` supplies deployment-selected transaction resource
  authority.

Neither is a global singleton. Callers construct and inject the capability
selected for one execution context.

## Access Profiles

`EvmEmbeddedAccessTracker<A, S>` is allocation-free and scans at most `A`
addresses or `S` storage slots per operation. It exists for explicitly small
embedded profiles.

`EvmNodeAccessTracker` is available behind `alloc`. Construction fallibly
pre-reserves the complete configured address, storage, radix-node, and undo
capacity. Its compressed binary radix indexes perform lookup and insertion in
`O(w)`, where `w` is fixed at 160 bits for addresses and 416 bits for
address/storage keys. No allocation occurs while warming accesses. Reset is
`O(n)` and retains only the bounded constructor allocation. Reset, rollback,
and drop erase key bytes through the audited optional sanitization bridge.
Rollback is `O(k)` for the `k` unique insertions made after the checkpoint and
does not inspect or rebuild retained outer-scope entries.

Both profiles enforce gas-derived hard ceilings. `warm_storage` is atomic: a
capacity failure cannot add only the address or only the slot. LIFO scope
checkpoints preserve pre-entry warmth but remove every address and slot first
warmed inside a reverted scope, as required by EIP-2929. The global address
ceiling includes paid EIP-2930 entries plus Prague's maximum initialized warm
set; deployments may select a lower fork-specific capacity.

## Governor Contract

`ExecutionResourceLimits::try_new` rejects zero capacities and values above the
reviewed ceilings for:

- warmed addresses and storage slots;
- journal entries and checkpoints;
- iterative frames and visible EVM memory;
- retained reusable-arena bytes and cache entries;
- abstract execution work.

`ExecutionGovernor::reset_transaction` is mandatory before accounting.
`charge` accepts only `CumulativeExecutionResource` and records distinct
addresses, slots, and journal entries. `observe_capacity` accepts only
`HighWaterExecutionResource` and records simultaneous or retained frame,
checkpoint, memory, arena, and cache requirements. The type split prevents an
integration from applying reusable high-water accounting to cumulative work.
Both paths are checked before mutation and atomic on exhaustion.

One work unit represents one scheduled interpreter or backend step.
`MAX_EXECUTION_WORK_UNITS` matches the reviewed one-million-step interpreter
ceiling. Integrations must charge multi-step operations by a reviewed upper
bound and may configure a lower deployment limit.

`ExecutionWorkToken` is non-copyable and has no public constructor. A parent
may delegate no more units than it owns. Delegation subtracts from the parent,
so nesting cannot create authority. Tokens carry the issuing transaction
generation for later node-governor integration.

The governor is a capability API, not an implicit scheduler. Hosts must route
every governed operation through it. `v0.65.0` assigns the complete node
resource-governor binding and operational policy freeze.

## Verification

Release evidence includes:

- differential operation streams across both access profiles;
- sorted and reverse-order distinct address/storage insertion with a checked
  fixed key-width lookup-depth bound;
- repeated empty and one-insertion child reverts over a populated outer scope,
  plus structural undo tests proving retained nodes are not rebuilt;
- atomic capacity, exact rollback, commit, reset, cancellation, retained
  allocation, and generation tests;
- `fuzz/fuzz_targets/access_tracker.rs`;
- `cargo run -p eth-valkyoth-evm-core --release --features std --example
  access_tracker_benchmark`.
