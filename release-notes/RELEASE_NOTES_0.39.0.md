# eth 0.39.0 Release Notes

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

`0.39.0` adds the bounded gas-estimation boundary. It still does not admit an
EVM execution backend.

## Added

- `eth-valkyoth-evm` publishes `GasEstimationPolicy`,
  `GasEstimationTermination`, `GasEstimationRequest`, `GasEstimationReport`,
  `GasEstimationStatus`, and `GasEstimationError`.
- Gas-estimation policies require a non-zero maximum attempt count, non-zero
  gas cap, non-zero termination guard, and hard release ceilings for all three
  resource classes.
- Estimation requests reject gas caps above the selected block gas limit.
- Estimation reports reject attempt counts above policy and estimates above
  the selected gas cap.
- The facade error module re-exports `GasEstimationError` when the `evm`
  feature is enabled.
- `scripts/release_0_39_gate.sh` captures default, `evm`, and all-feature
  dependency tree evidence.

## Changed

- `eth-valkyoth-evm` publishes as `0.10.0`.
- `eth` publishes as `0.39.0` and points its optional `evm` feature at
  `eth-valkyoth-evm 0.10.0`.

## Security Notes

- Gas estimation remains a boundary contract only; no backend is admitted by
  this release.
- Future estimators must carry a reviewed maximum attempt count, gas cap, and
  deterministic termination policy before they can run.
- `MAX_GAS_ESTIMATION_ATTEMPTS`, `MAX_GAS_ESTIMATION_GAS_CAP`,
  `MAX_GAS_ESTIMATION_BACKEND_STEPS`, and
  `MAX_GAS_ESTIMATION_TIMEOUT_MILLIS` prevent callers from treating
  practically infinite values as a bounded policy.
- The gas cap is bound to the block gas limit at request construction time.
- Reports bind gas-estimation outcomes to the existing execution report, which
  already carries fork, block, transaction hash, and snapshot identity.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm`
- `cargo check -p eth --features evm`
- `cargo clippy -p eth-valkyoth-evm -p eth --all-targets --all-features -- -D warnings`
- `cargo tree -p eth --no-default-features --features evm -e normal`
- `cargo update --dry-run --verbose`
- `scripts/release_0_39_gate.sh`

## Pentest Remediation

- Initial pentest found that `GasEstimationPolicy::try_new` rejected zero
  resource limits but did not reject practically unbounded maximum values.
- Remediation added hard release ceilings for attempts, gas cap, backend steps,
  and worker timeout values, plus deterministic too-high error variants.
- Remediation also added coverage for zero `WorkerTimeout` and
  `WorkerIsolation` policies.

## Pentest

- External pentest found one medium bounded-resource issue and one low
  coverage gap.
- Remediation added hard release ceilings for gas-estimation attempts, gas cap,
  backend steps, and worker timeout values, plus missing worker-termination
  coverage.
- Clean retest passed.
- Permanent report: `security/pentest/v0.39.0.md`.

## Versioning

- `eth-valkyoth-evm` publishes as `0.10.0`.
- `eth` publishes as `0.39.0`.
- Other support crates are unchanged and are not republished.
