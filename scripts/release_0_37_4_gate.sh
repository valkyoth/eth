#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
scripts/check_ethereum_upstream.py
scripts/check_runtime_dependency_policy.py
python3 scripts/test-runtime-dependency-policy.py
mkdir -p target
cargo tree -p eth -e normal > target/release_0_37_4_default_runtime_tree.txt
cargo tree -p eth -e features --all-features > target/release_0_37_4_all_features_tree.txt
scripts/release_crates.py --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo deny check
cargo audit
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
