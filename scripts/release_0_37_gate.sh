#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
scripts/release_crates.py --check
scripts/sync_spec_sources.py --lock-only
python3 scripts/test-sync-spec-sources.py
scripts/run_execution_fixtures.py --check
scripts/run_differential_tests.py --check
scripts/run_differential_tests.py
scripts/materialize_fuzz_seeds.py --check
cargo check --manifest-path fuzz/Cargo.toml
cargo test -p eth --all-features
cargo test -p eth-valkyoth-evm
cargo clippy -p eth --all-targets --all-features -- -D warnings
cargo clippy -p eth-valkyoth-evm --all-targets -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
