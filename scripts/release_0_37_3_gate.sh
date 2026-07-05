#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
scripts/check_ethereum_upstream.py
mkdir -p target
cargo tree -p eth --no-default-features > target/release_0_37_3_default_tree.txt
if grep -Eq '(^|[[:space:]])(k256|sha3) v' target/release_0_37_3_default_tree.txt; then
    echo "default eth graph must not include k256 or sha3" >&2
    exit 1
fi
cargo tree -p eth -e features --no-default-features
cargo tree -p eth -e features --all-features
cargo test -p eth-valkyoth-verify --all-features
cargo clippy -p eth-valkyoth-verify --all-targets --all-features -- -D warnings
cargo tree -e features --workspace
scripts/release_crates.py --check
scripts/sync_spec_sources.py --lock-only
python3 scripts/test-sync-spec-sources.py
scripts/run_execution_fixtures.py --check
scripts/run_differential_tests.py --check
scripts/run_differential_tests.py
scripts/materialize_fuzz_seeds.py --check
cargo check --manifest-path fuzz/Cargo.toml
cargo test -p eth --all-features
cargo clippy -p eth --all-targets --all-features -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
