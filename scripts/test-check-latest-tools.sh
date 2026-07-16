#!/usr/bin/env sh
set -eu

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

toolchain_file="$tmp_dir/rust-toolchain.toml"
manifest_file="$tmp_dir/channel-rust-stable.toml"

cat >"$toolchain_file" <<'EOF'
[toolchain]
channel = "1.97.1"
EOF

cat >"$manifest_file" <<'EOF'
manifest-version = "2"

[pkg.rust]
version = "1.97.1 (fixture)"

[pkg.rust.target.example]
available = true
EOF

RUST_TOOLCHAIN_FILE="$toolchain_file" \
RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
CHECK_LATEST_TOOLS_RUST_ONLY=1 \
    scripts/check_latest_tools.sh

cat >"$toolchain_file" <<'EOF'
[toolchain]
channel = "1.97.0"
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    CHECK_LATEST_TOOLS_RUST_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "stale pinned Rust version was accepted" >&2
    exit 1
fi

cat >"$manifest_file" <<'EOF'
manifest-version = "2"
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    CHECK_LATEST_TOOLS_RUST_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "missing stable Rust version was accepted" >&2
    exit 1
fi

echo "latest Rust tool check tests passed"
