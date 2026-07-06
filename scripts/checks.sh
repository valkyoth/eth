#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/check_doc_links.sh
scripts/validate-release-metadata.sh
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/release_crates.py --check
scripts/sync_spec_sources.py --lock-only
python3 scripts/test-sync-spec-sources.py
python3 scripts/test-ethereum-upstream.py
scripts/check_ethereum_upstream.py --local-only
scripts/run_execution_fixtures.py --check
scripts/run_differential_tests.py --check
scripts/materialize_fuzz_seeds.py --check
python3 scripts/test-workspace-dependency-policy.py
scripts/check_runtime_dependency_policy.py
python3 scripts/test-runtime-dependency-policy.py
scripts/check_optional_boundary_policy.py
python3 scripts/test-optional-boundary-policy.py
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
    eth-valkyoth-evm-core \
    eth-valkyoth-signer \
    eth-valkyoth-testkit \
    eth-valkyoth-verify; do
    case "$package" in
    eth-valkyoth-primitives)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-codec.path="crates/eth-valkyoth-codec"'
        ;;
    eth-valkyoth-hash | eth-valkyoth-signer)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-primitives.path="crates/eth-valkyoth-primitives"'
        ;;
    eth-valkyoth-protocol)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-codec.path="crates/eth-valkyoth-codec"' \
            --config 'patch.crates-io.eth-valkyoth-primitives.path="crates/eth-valkyoth-primitives"' \
            --config 'patch.crates-io.eth-valkyoth-hash.path="crates/eth-valkyoth-hash"'
        ;;
    eth-valkyoth-sanitization)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-derive.path="crates/eth-valkyoth-derive"'
        ;;
    eth-valkyoth-derive)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-codec.path="crates/eth-valkyoth-codec"' \
            --config 'patch.crates-io.eth-valkyoth-primitives.path="crates/eth-valkyoth-primitives"'
        ;;
    eth-valkyoth-verify)
        cargo package -p "$package" --allow-dirty \
            --config 'patch.crates-io.eth-valkyoth-codec.path="crates/eth-valkyoth-codec"' \
            --config 'patch.crates-io.eth-valkyoth-primitives.path="crates/eth-valkyoth-primitives"' \
            --config 'patch.crates-io.eth-valkyoth-hash.path="crates/eth-valkyoth-hash"' \
            --config 'patch.crates-io.eth-valkyoth-protocol.path="crates/eth-valkyoth-protocol"'
        ;;
    *)
        cargo package -p "$package" --allow-dirty
        ;;
    esac
done
cargo package -p eth --allow-dirty \
    --config 'patch.crates-io.eth-valkyoth-codec.path="crates/eth-valkyoth-codec"' \
    --config 'patch.crates-io.eth-valkyoth-primitives.path="crates/eth-valkyoth-primitives"' \
    --config 'patch.crates-io.eth-valkyoth-hash.path="crates/eth-valkyoth-hash"' \
    --config 'patch.crates-io.eth-valkyoth-protocol.path="crates/eth-valkyoth-protocol"' \
    --config 'patch.crates-io.eth-valkyoth-evm.path="crates/eth-valkyoth-evm"' \
    --config 'patch.crates-io.eth-valkyoth-evm-core.path="crates/eth-valkyoth-evm-core"' \
    --config 'patch.crates-io.eth-valkyoth-reth.path="crates/eth-valkyoth-reth"' \
    --config 'patch.crates-io.eth-valkyoth-rpc.path="crates/eth-valkyoth-rpc"' \
    --config 'patch.crates-io.eth-valkyoth-sanitization.path="crates/eth-valkyoth-sanitization"' \
    --config 'patch.crates-io.eth-valkyoth-signer.path="crates/eth-valkyoth-signer"' \
    --config 'patch.crates-io.eth-valkyoth-testkit.path="crates/eth-valkyoth-testkit"' \
    --config 'patch.crates-io.eth-valkyoth-verify.path="crates/eth-valkyoth-verify"'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --manifest-path fuzz/Cargo.toml
