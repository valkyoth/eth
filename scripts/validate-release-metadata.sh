#!/usr/bin/env sh
set -eu

test ! -f PENTEST.md
test -f LICENSE-MIT
test -f LICENSE-APACHE
test -f SECURITY.md
test -f CHANGELOG.md
test -x scripts/validate-release-readiness.sh
test -x scripts/test-release-readiness.sh
test -x scripts/check_latest_tools.sh
test -x scripts/check_ethereum_upstream.py
test -x scripts/test-ethereum-upstream.py
test -x scripts/release_crates.py
test -x scripts/sync_spec_sources.py
test -x scripts/test-sync-spec-sources.py
test -x scripts/run_execution_fixtures.py
test -x scripts/run_differential_tests.py
test -x scripts/check_runtime_dependency_policy.py
test -x scripts/test-runtime-dependency-policy.py
test -x scripts/check_optional_boundary_policy.py
test -x scripts/test-optional-boundary-policy.py
test -f release-crates.toml
test -f docs/CRATE_VERSION_MATRIX.md
test -f conformance/execution-fixtures.toml
test -f release-notes/RELEASE_NOTES_0.1.0.md
test -f release-notes/RELEASE_NOTES_0.2.0.md
test -f release-notes/RELEASE_NOTES_0.3.0.md
test -f release-notes/RELEASE_NOTES_0.4.0.md
test -f release-notes/RELEASE_NOTES_0.5.0.md
test -f release-notes/RELEASE_NOTES_0.6.0.md
test -f release-notes/RELEASE_NOTES_0.7.0.md
test -f release-notes/RELEASE_NOTES_0.8.0.md
test -f release-notes/RELEASE_NOTES_0.9.0.md
test -f release-notes/RELEASE_NOTES_0.10.0.md
test -f release-notes/RELEASE_NOTES_0.11.0.md
test -f release-notes/RELEASE_NOTES_0.12.0.md
test -f release-notes/RELEASE_NOTES_0.13.0.md
test -f release-notes/RELEASE_NOTES_0.14.0.md
test -f release-notes/RELEASE_NOTES_0.15.0.md
test -f release-notes/RELEASE_NOTES_0.16.0.md
test -f release-notes/RELEASE_NOTES_0.16.1.md
test -f release-notes/RELEASE_NOTES_0.17.0.md
test -f release-notes/RELEASE_NOTES_0.18.0.md
test -f release-notes/RELEASE_NOTES_0.19.0.md
test -f release-notes/RELEASE_NOTES_0.20.0.md
test -f release-notes/RELEASE_NOTES_0.21.0.md
test -f release-notes/RELEASE_NOTES_0.22.0.md
test -f release-notes/RELEASE_NOTES_0.23.0.md
test -f release-notes/RELEASE_NOTES_0.24.0.md
test -f release-notes/RELEASE_NOTES_0.24.1.md
test -f release-notes/RELEASE_NOTES_0.24.2.md
test -f release-notes/RELEASE_NOTES_0.25.0.md
test -f release-notes/RELEASE_NOTES_0.26.0.md
test -f release-notes/RELEASE_NOTES_0.26.1.md
test -f release-notes/RELEASE_NOTES_0.27.0.md
test -f release-notes/RELEASE_NOTES_0.28.0.md
test -f release-notes/RELEASE_NOTES_0.29.0.md
test -f release-notes/RELEASE_NOTES_0.30.0.md
test -f release-notes/RELEASE_NOTES_0.31.0.md
test -f release-notes/RELEASE_NOTES_0.32.0.md
test -f release-notes/RELEASE_NOTES_0.33.0.md
test -f release-notes/RELEASE_NOTES_0.34.0.md
test -f release-notes/RELEASE_NOTES_0.35.0.md
test -f release-notes/RELEASE_NOTES_0.36.0.md
test -f release-notes/RELEASE_NOTES_0.37.0.md
test -f release-notes/RELEASE_NOTES_0.37.1.md
test -f release-notes/RELEASE_NOTES_0.37.2.md
test -f release-notes/RELEASE_NOTES_0.37.3.md
test -f release-notes/RELEASE_NOTES_0.37.4.md
test -f release-notes/RELEASE_NOTES_0.37.5.md
test -f release-notes/RELEASE_NOTES_0.38.0.md
test -f release-notes/RELEASE_NOTES_0.39.0.md
test -f release-notes/RELEASE_NOTES_0.40.0.md
test -f release-notes/RELEASE_NOTES_0.41.0.md
test -f security/pentest/v0.40.0.md
test -f security/pentest/v0.41.0.md
test -x scripts/release_0_9_gate.sh
test -x scripts/release_0_10_gate.sh
test -x scripts/release_0_11_gate.sh
test -x scripts/release_0_12_gate.sh
test -x scripts/release_0_13_gate.sh
test -x scripts/release_0_14_gate.sh
test -x scripts/release_0_15_gate.sh
test -x scripts/release_0_16_gate.sh
test -x scripts/release_0_17_gate.sh
test -x scripts/release_0_18_gate.sh
test -x scripts/release_0_19_gate.sh
test -x scripts/release_0_20_gate.sh
test -x scripts/release_0_21_gate.sh
test -x scripts/release_0_22_gate.sh
test -x scripts/release_0_23_gate.sh
test -x scripts/release_0_24_gate.sh
test -x scripts/release_0_25_gate.sh
test -x scripts/release_0_26_gate.sh
test -x scripts/release_0_26_1_gate.sh
test -x scripts/release_0_27_gate.sh
test -x scripts/release_0_28_gate.sh
test -x scripts/release_0_29_gate.sh
test -x scripts/release_0_30_gate.sh
test -x scripts/release_0_31_gate.sh
test -x scripts/release_0_32_gate.sh
test -x scripts/release_0_33_gate.sh
test -x scripts/release_0_34_gate.sh
test -x scripts/release_0_35_gate.sh
test -x scripts/release_0_36_gate.sh
test -x scripts/release_0_37_gate.sh
test -x scripts/release_0_37_1_gate.sh
test -x scripts/release_0_37_2_gate.sh
test -x scripts/release_0_37_3_gate.sh
test -x scripts/release_0_37_4_gate.sh
test -x scripts/release_0_37_5_gate.sh
test -x scripts/release_0_38_gate.sh
test -x scripts/release_0_39_gate.sh
test -x scripts/release_0_40_gate.sh
test -x scripts/release_0_41_gate.sh
test -x scripts/test-workspace-dependency-policy.py
test -f docs/spec-source-policy.md
test -f docs/reference-store.md
test -f docs/execution-fixture-harness.md
test -f docs/execution-fixture-report.md
test -f docs/unsupported-execution-fixtures.md
test -f docs/differential-test-harness.md
test -f docs/differential-test-report.md
test -f docs/revm-dependency-review.md
test -f docs/ethereum-upstream-check.md
test -f docs/core-independence-audit.md
test -f docs/constant-time-reference-policy.md
test -f docs/optional-parser-sanitization-policy.md
test -f docs/evm-execution-environment.md
grep -q 'execution_specs_repo' spec-lock.toml
grep -q 'local_reference_store_env' spec-lock.toml
grep -q 'local_reference_store_default' spec-lock.toml
if grep -q 'spec_required = true' spec-lock.toml; then
    grep -Eq 'checked_at = "[0-9]{4}-[0-9]{2}-[0-9]{2}"' spec-lock.toml
    grep -Eq 'execution_specs_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'execution_tests_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'eips_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'execution_apis_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'consensus_specs_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq '^execution_specs_repo = "https://github\.com/ethereum/[A-Za-z0-9_.-]+"$' spec-lock.toml
    grep -Eq '^execution_tests_repo = "https://github\.com/ethereum/[A-Za-z0-9_.-]+"$' spec-lock.toml
    grep -Eq '^eips_repo = "https://github\.com/ethereum/[A-Za-z0-9_.-]+"$' spec-lock.toml
    grep -Eq '^execution_apis_repo = "https://github\.com/ethereum/[A-Za-z0-9_.-]+"$' spec-lock.toml
    grep -Eq '^consensus_specs_repo = "https://github\.com/ethereum/[A-Za-z0-9_.-]+"$' spec-lock.toml
fi
grep -q 'license = "MIT OR Apache-2.0"' Cargo.toml
grep -q 'repository = "https://github.com/valkyoth/eth"' Cargo.toml
grep -q 'channel = "1.96.1"' rust-toolchain.toml
grep -q 'rust-version = "1.90"' Cargo.toml
grep -q 'valkyoth-eth-upstream-check/0.41.0' scripts/check_ethereum_upstream.py
