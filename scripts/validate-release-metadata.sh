#!/usr/bin/env sh
set -eu

test ! -f PENTEST.md
test -f LICENSE
test -f SECURITY.md
test -f CHANGELOG.md
test -x scripts/validate-release-readiness.sh
test -f release-notes/RELEASE_NOTES_0.1.0.md
test -f docs/spec-source-policy.md
grep -q 'execution_specs_repo' spec-lock.toml
grep -q 'local_reference_store_env' spec-lock.toml
grep -q 'local_reference_store_default' spec-lock.toml
grep -q 'EUPL-1.2' Cargo.toml
grep -q 'channel = "1.96.0"' rust-toolchain.toml
grep -q 'rust-version = "1.90"' Cargo.toml
