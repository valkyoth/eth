#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_latest_tools.sh
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0; do
    cargo "+$toolchain" check --workspace --all-features
done
