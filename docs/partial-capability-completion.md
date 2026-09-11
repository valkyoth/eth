# Partial Capability Completion Map

Reviewed for the v0.60.0 candidate against the current release plan.
The five yellow README rows describe the **candidate v0.60.0** scope, not
the latest internal tag. Each has concrete implementation and acceptance
milestones below; none is an indefinite deferral.

| README capability | Implementation owners | Completion and verification gates |
| --- | --- | --- |
| Legacy, EIP-2930, EIP-1559, EIP-4844 transactions | [v0.132.0 rule families and v0.133.0 semantic validity](RELEASE_PLAN.md#v01320---transaction-validity-rule-families), state application v0.135.0/v0.137.0, [v0.150.0 blob integration](RELEASE_PLAN.md#v01500---blob-transaction-and-block-integration) | Official per-family transaction vectors and client differential checks at v0.133.0; integrated execution fixtures and differential/performance gates v0.166.0/v0.167.0. |
| EIP-7702 set-code transactions | v0.133.0 semantic validity, [v0.136.0 EIP-7702 state application](RELEASE_PLAN.md#v01360---eip-7702-state-application), v0.137.0 journaling | Authority ordering, delegation, nonce/balance/fee effects, rollback and reauthorization vectors; execution integration at v0.166.0/v0.167.0. |
| Headers, receipts, withdrawals | [v0.134.0 header/block validity](RELEASE_PLAN.md#v01340---header-and-block-validity), v0.137.0 transition, v0.138.0/v0.139.0 system operations, v0.140.0 receipts/logs/bloom/withdrawals, v0.141.0/v0.142.0 trie roots | Parent/fork substitution and full block/root/receipt fixtures at v0.166.0/v0.167.0; historical PoW difficulty, ommers, rewards and genesis-to-Merge admission at v0.411.0..=v0.413.0. |
| Native EVM execution | v0.124.0 core integration, [v0.135.0..=v0.137.0 transaction journal/state transition](RELEASE_PLAN.md#v01350---ordered-transaction-state-journal), v0.138.0..=v0.150.0 system/state/blob integration, v0.151.0..=v0.165.0 explicit fork admission | v0.129.0/v0.130.0 initial fixture/audit gates, v0.166.0/v0.167.0 complete execution tests, historical v0.411.0..=v0.413.0, and full-client v0.428.0..=v0.433.0 interoperability/performance/audit/remediation. |
| BLS12-381 and KZG | [v0.56.0..=v0.70.0 BLS field/group/subgroup/MSM/map/pairing and precompiles](RELEASE_PLAN.md#v0560---bls12-381-base-field), [v0.143.0..=v0.150.0 KZG setup/polynomials/proofs/point evaluation/blob integration](RELEASE_PLAN.md#v01430---kzg-trusted-setup-boundary) | Official vectors, independent oracles and charged execution at v0.70.0 and v0.149.0/v0.150.0. Consensus secret signing/aggregation/batch isolation are separately v0.309.0..=v0.311.0; PeerDAS v0.312.0..=v0.315.0; full-stack crypto readmission v0.407.0..=v0.410.0. |

## Promotion Rule

The detailed [release plan](RELEASE_PLAN.md) owns each milestone's goal,
deliverables, verification and exact-commit pentest stop. This index does not
replace or relax those contracts. In particular, parsing or arithmetic alone
cannot turn a consensus-validation or precompile-execution claim green.

At a public checkpoint, review each affected row against retained tests,
remediated pentest findings, CI and the specification matrix. Promote only the
verified scope present in the published archives. Split rows when one bounded
sub-capability is complete but another is not; do not mark all BLS/KZG green
because G1ADD alone works. New fork rules require their own admission evidence.
Full production claims still require the final v0.442.0..=v0.449.0 gates.

Recheck these links whenever the roadmap is renumbered or a README scope changes.
