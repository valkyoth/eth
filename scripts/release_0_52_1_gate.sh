#!/usr/bin/env sh
set -eu

rustc --version | grep -q '^rustc 1\.97\.0 '
scripts/checks.sh
scripts/validate-release-readiness.sh v0.52.1
scripts/check_latest_tools.sh
scripts/check_ethereum_upstream.py
scripts/check_runtime_dependency_policy.py
scripts/check_optional_boundary_policy.py
python3 scripts/test-runtime-dependency-policy.py
python3 scripts/test-optional-boundary-policy.py
mkdir -p target
cargo tree -p eth -e normal > target/release_0_52_1_default_runtime_tree.txt
cargo tree -p eth --no-default-features --features evm -e normal > target/release_0_52_1_evm_tree.txt
cargo tree -p eth --no-default-features --features evm-core -e normal > target/release_0_52_1_evm_core_tree.txt
cargo tree -p eth -e features --all-features > target/release_0_52_1_all_features_tree.txt
scripts/release_crates.py --check
cargo test -p eth-valkyoth-evm-core bls12_wire_tests --all-features
cargo test -p eth-valkyoth-evm-core --all-features
cargo check -p eth --features evm-core
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --manifest-path fuzz/Cargo.toml --bin bls12381_wire -- -D warnings
scripts/materialize_fuzz_seeds.py --check
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
