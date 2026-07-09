#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/validate-release-readiness.sh v0.50.9
scripts/check_latest_tools.sh
scripts/check_ethereum_upstream.py
scripts/check_runtime_dependency_policy.py
scripts/check_optional_boundary_policy.py
python3 scripts/test-runtime-dependency-policy.py
python3 scripts/test-optional-boundary-policy.py
mkdir -p target
cargo tree -p eth -e normal > target/release_0_50_9_default_runtime_tree.txt
cargo tree -p eth --no-default-features --features evm -e normal > target/release_0_50_9_evm_tree.txt
cargo tree -p eth --no-default-features --features evm-core -e normal > target/release_0_50_9_evm_core_tree.txt
cargo tree -p eth -e features --all-features > target/release_0_50_9_all_features_tree.txt
scripts/release_crates.py --check
cargo test -p eth-valkyoth-evm-core bn254
cargo test -p eth-valkyoth-evm-core
cargo test --release -p eth-valkyoth-evm-core miller_loop_wall_time_budget_smoke -- --ignored --nocapture
cargo test --release -p eth-valkyoth-evm-core bn254_pairing_final_exponentiation_wall_time_budget_smoke -- --ignored --nocapture
cargo test -p eth-valkyoth-evm
cargo check -p eth --features evm-core
cargo check -p eth --features evm
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
