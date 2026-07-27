# Execution Resource Governor

Status: `v0.53.0` implementation complete; awaiting pentest.

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
pre-reserves the complete configured address and storage capacity. Its AVL
indexes perform worst-case `O(log n)` comparisons and rotations and do not
allocate while warming accesses. Reset is `O(n)` and retains only the bounded
constructor allocation. Root-attempt rollback is `O(n log n)` and performs no
allocation.

Both profiles enforce gas-derived hard ceilings. `warm_storage` is atomic: a
capacity failure cannot add only the address or only the slot. Child-call
reverts preserve EIP-2929 transaction warmth; failed or reverted root attempts
restore the exact pre-attempt state.

## Governor Contract

`ExecutionResourceLimits::try_new` rejects zero capacities and values above the
reviewed ceilings for:

- warmed addresses and storage slots;
- journal entries and checkpoints;
- iterative frames and visible EVM memory;
- retained reusable-arena bytes and cache entries;
- abstract execution work.

`ExecutionGovernor::reset_transaction` is mandatory before accounting.
`charge` records cumulative work such as distinct admitted entries.
`observe_capacity` records a simultaneous or retained high-water requirement
for reusable frames, checkpoint depth, memory, arenas, and caches. Both are
checked before mutation and atomic on exhaustion. Accounting is monotonic for
the transaction: failed or cancelled work is not refunded.

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
  logarithmic tree-height bound;
- atomic capacity, exact rollback, commit, reset, cancellation, retained
  allocation, and generation tests;
- `fuzz/fuzz_targets/access_tracker.rs`;
- `cargo run -p eth-valkyoth-evm-core --release --features std --example
  access_tracker_benchmark`.
