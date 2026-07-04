#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
scripts/release_crates.py --check
scripts/sync_spec_sources.py --lock-only
scripts/sync_spec_sources.py --only execution_tests
python3 scripts/test-sync-spec-sources.py
scripts/run_execution_fixtures.py --check
scripts/materialize_fuzz_seeds.py --check
cargo check --manifest-path fuzz/Cargo.toml
cargo test -p eth --all-features
scripts/run_execution_fixtures.py
cargo clippy -p eth --all-targets --all-features -- -D warnings
cargo clippy -p eth-valkyoth-codec --test execution_rlp_fixtures --features testing -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
