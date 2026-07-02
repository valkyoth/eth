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
test -x scripts/release_crates.py
test -f release-crates.toml
test -f docs/CRATE_VERSION_MATRIX.md
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
test -f docs/spec-source-policy.md
grep -q 'execution_specs_repo' spec-lock.toml
grep -q 'local_reference_store_env' spec-lock.toml
grep -q 'local_reference_store_default' spec-lock.toml
if grep -q 'spec_required = true' spec-lock.toml; then
    grep -Eq 'execution_specs_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'execution_tests_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'eips_rev = "[0-9a-f]{40}"' spec-lock.toml
fi
grep -q 'license = "MIT OR Apache-2.0"' Cargo.toml
grep -q 'repository = "https://github.com/valkyoth/eth"' Cargo.toml
grep -q 'channel = "1.96.1"' rust-toolchain.toml
grep -q 'rust-version = "1.90"' Cargo.toml
