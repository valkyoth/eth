#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/check_doc_links.sh
scripts/check_release_plan.sh
scripts/test-release-plan.sh
python3 scripts/check_roadmap_map.py
python3 scripts/test-roadmap-map.py
scripts/test-check-latest-tools.sh
scripts/test-check-latest-crates.py
if ! cmp -s README.md crates/eth/README.md; then
    echo "README.md and crates/eth/README.md must remain identical" >&2
    diff -u README.md crates/eth/README.md >&2 || true
    exit 1
fi
python3 scripts/test-readmes.py
python3 scripts/test-check-packages.py
python3 scripts/check_readmes.py --test
scripts/validate-release-metadata.sh
python3 scripts/test-sbom-compare.py
scripts/generate-sbom.sh --check
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/release_crates.py --check
scripts/sync_spec_sources.py --lock-only
python3 scripts/test-sync-spec-sources.py
python3 scripts/test-ethereum-upstream.py
scripts/check_ethereum_upstream.py --local-only
scripts/run_execution_fixtures.py --check
scripts/run_differential_tests.py --check
python3 scripts/test-run-modexp-client-differential.py
scripts/materialize_fuzz_seeds.py --check
python3 scripts/test-workspace-dependency-policy.py
scripts/check_runtime_dependency_policy.py
python3 scripts/test-runtime-dependency-policy.py
scripts/check_optional_boundary_policy.py
python3 scripts/test-optional-boundary-policy.py
python3 scripts/test-release-metadata.py
python3 scripts/test-release-train.py
python3 scripts/test-release-gate.py
python3 scripts/test-release-dependencies.py
python3 scripts/test-release-crates.py
scripts/test-release-readiness.sh
python3 scripts/check_packages.py
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --manifest-path fuzz/Cargo.toml
