#!/usr/bin/env sh
set -eu

test ! -f PENTEST.md
test -f LICENSE
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
test -f docs/spec-source-policy.md
grep -q 'execution_specs_repo' spec-lock.toml
grep -q 'local_reference_store_env' spec-lock.toml
grep -q 'local_reference_store_default' spec-lock.toml
if grep -q 'spec_required = true' spec-lock.toml; then
    grep -Eq 'execution_specs_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'execution_tests_rev = "[0-9a-f]{40}"' spec-lock.toml
    grep -Eq 'eips_rev = "[0-9a-f]{40}"' spec-lock.toml
fi
grep -q 'EUPL-1.2' Cargo.toml
grep -q 'repository = "https://github.com/valkyoth/eth"' Cargo.toml
grep -q 'channel = "1.96.0"' rust-toolchain.toml
grep -q 'rust-version = "1.90"' Cargo.toml
