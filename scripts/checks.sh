#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/check_doc_links.sh
scripts/validate-release-metadata.sh
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/release_crates.py --check
scripts/materialize_fuzz_seeds.py --check
python3 scripts/test-release-crates.py
scripts/test-release-readiness.sh
for package in \
    eth-valkyoth-codec \
    eth-valkyoth-evm \
    eth-valkyoth-primitives \
    eth-valkyoth-hash \
    eth-valkyoth-protocol \
    eth-valkyoth-reth \
    eth-valkyoth-rpc \
    eth-valkyoth-derive \
    eth-valkyoth-sanitization \
    eth-valkyoth-signer \
    eth-valkyoth-testkit \
    eth-valkyoth-verify; do
    cargo package -p "$package" --allow-dirty
done
cargo package -p eth --allow-dirty \
    --config 'patch.crates-io.eth-valkyoth-verify.path="crates/eth-valkyoth-verify"'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --manifest-path fuzz/Cargo.toml
