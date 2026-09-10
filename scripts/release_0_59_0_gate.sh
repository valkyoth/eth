#!/usr/bin/env sh
set -eu

if [ "$#" -gt 1 ]; then
    echo "usage: $0 [--implementation|--tag]" >&2
    exit 2
fi
case "${1:---tag}" in
    --tag)
        scripts/validate-release-metadata.sh
        scripts/validate-release-readiness.sh v0.59.0
        exit 0
        ;;
    --implementation) ;;
    *)
        echo "usage: $0 [--implementation|--tag]" >&2
        exit 2
        ;;
esac

rustc --version | grep -q '^rustc 1\.98\.1 '
scripts/checks.sh
scripts/check_latest_tools.sh
scripts/check_latest_crates.py
scripts/check_ethereum_upstream.py
cargo test -p eth-valkyoth-evm-core --all-features
cargo test -p eth-valkyoth-evm-core --release --test bls12_fp2_differential --test bls12_field_differential --test bls12_g1_differential --test bls12_g1_vectors --test bls12_g1_rejections
cargo run -p eth-valkyoth-evm-core --release --example bls12_fp2_benchmark
cargo run -p eth-valkyoth-evm-core --release --example bls12_g1_add_benchmark
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_g1_add -- -max_total_time=30 -max_len=257
cargo +nightly fuzz run bls12381_fp2 -- -max_total_time=30 -max_len=193
scripts/run_differential_tests.py --in-process
cargo deny check
cargo audit
cargo audit --file fuzz/Cargo.lock
for toolchain in 1.90.0 1.91.0 1.91.1 1.92.0 1.93.0 1.93.1 1.94.0 1.94.1 1.95.0 1.96.0 1.96.1 1.97.0 1.97.1 1.98.0; do
    cargo "+$toolchain" check --workspace --all-features
done
