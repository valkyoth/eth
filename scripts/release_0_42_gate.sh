#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
scripts/check_ethereum_upstream.py
scripts/check_runtime_dependency_policy.py
scripts/check_optional_boundary_policy.py
python3 scripts/test-runtime-dependency-policy.py
python3 scripts/test-optional-boundary-policy.py
mkdir -p target
cargo tree -p eth -e normal > target/release_0_42_default_runtime_tree.txt
cargo tree -p eth --no-default-features --features evm -e normal > target/release_0_42_evm_tree.txt
cargo tree -p eth --no-default-features --features evm-core -e normal > target/release_0_42_evm_core_tree.txt
cargo tree -p eth -e features --all-features > target/release_0_42_all_features_tree.txt
scripts/release_crates.py --check
cargo test -p eth-valkyoth-evm-core
cargo test -p eth-valkyoth-evm
cargo check -p eth --features evm-core
cargo check -p eth --features evm
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
