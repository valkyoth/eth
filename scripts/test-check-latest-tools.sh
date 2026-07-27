#!/usr/bin/env sh
set -eu

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

toolchain_file="$tmp_dir/rust-toolchain.toml"
manifest_file="$tmp_dir/channel-rust-stable.toml"
workflow_dir="$tmp_dir/workflows"
mkdir -p "$workflow_dir"

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

cat >"$manifest_file" <<'EOF'
manifest-version = "2"

[pkg.rust]
version = "1.97.1 (fixture)"
EOF
cat >"$toolchain_file" <<'EOF'
[toolchain]
channel = "1.97.1"
EOF

cat >"$workflow_dir/ci.yml" <<'EOF'
steps:
  - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
EOF
cat >"$workflow_dir/release.yml" <<'EOF'
steps:
  - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
EOF
cat >"$workflow_dir/normal.yml" <<'EOF'
steps:
  - uses: valkyoth/normal-action@1234567890abcdef1234567890abcdef12345678
EOF
cat >"$workflow_dir/normal.yaml" <<'EOF'
steps:
  - uses: docker://example/normal@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd
EOF
RUST_TOOLCHAIN_FILE="$toolchain_file" \
RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
GITHUB_WORKFLOW_DIR="$workflow_dir" \
CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh

cat >"$workflow_dir/.hidden.yml" <<'EOF'
steps:
  - uses: attacker/example@main
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "unpinned action in hidden .yml workflow was accepted" >&2
    exit 1
fi
rm "$workflow_dir/.hidden.yml"

cat >"$workflow_dir/.hidden.yaml" <<'EOF'
steps:
  - uses: docker://attacker/image:latest
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable Docker action in hidden .yaml workflow was accepted" >&2
    exit 1
fi
rm "$workflow_dir/.hidden.yaml"

cat >"$workflow_dir/.hidden.yaml" <<'EOF'
jobs:
  build:
    container: attacker/build:latest
    steps: []
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable job container in hidden .yaml workflow was accepted" >&2
    exit 1
fi
rm "$workflow_dir/.hidden.yaml"

cat >"$workflow_dir/.hidden.yaml" <<'EOF'
jobs:
  build:
    services:
      database:
        image: attacker/database:latest
    steps: []
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable service container in hidden .yaml workflow was accepted" >&2
    exit 1
fi
rm "$workflow_dir/.hidden.yaml"

cat >"$workflow_dir/.hidden.yml" <<'EOF'
steps:
  - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "unannotated checkout in hidden workflow was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/.hidden.yml" <<'EOF'
steps:
  - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
EOF
RUST_TOOLCHAIN_FILE="$toolchain_file" \
RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
GITHUB_WORKFLOW_DIR="$workflow_dir" \
CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh
rm "$workflow_dir/.hidden.yml"

cat >"$workflow_dir/release.yml" <<'EOF'
steps:
  - uses: actions/checkout@v7.0.1
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "unpinned action in secondary workflow was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/release.yml" <<'EOF'
steps:
  - uses: actions/checkout@0000000000000000000000000000000000000000
# uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "spoofed checkout annotation was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/release.yml" <<'EOF'
jobs:
  reusable:
    uses: valkyoth/example/.github/workflows/check.yml@1234567890abcdef1234567890abcdef12345678
  checks:
    container:
      image: example/build@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    services:
      database:
        image: example/database@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
    steps:
      - { uses: "valkyoth/flow-action@abcdef1234567890abcdef1234567890abcdef12" }
      - { "uses": './local-action' }
      - uses: docker://example/image@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
EOF
cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - 'uses': 'valkyoth/quoted-action@ABCDEF1234567890ABCDEF1234567890ABCDEF12'
EOF
RUST_TOOLCHAIN_FILE="$toolchain_file" \
RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
GITHUB_WORKFLOW_DIR="$workflow_dir" \
CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh

cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - { uses: attacker/example@main }
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "unpinned flow-mapping action was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - uses: ${{ matrix.action }}
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "expression-based action reference was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - uses:
      repository: attacker/example
      ref: main
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "non-string action reference was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - uses: Actions/Checkout@abcdef1234567890abcdef1234567890abcdef12
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "noncanonical checkout casing was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
steps:
  - uses: docker://example/image:latest
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable Docker action tag was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
jobs:
  build:
    container: example/build:latest
    steps: []
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable job container tag was accepted" >&2
    exit 1
fi

cat >"$workflow_dir/extra.yaml" <<'EOF'
jobs:
  build:
    services:
      database:
        image: example/database:latest
    steps: []
EOF
if RUST_TOOLCHAIN_FILE="$toolchain_file" \
    RUST_STABLE_MANIFEST_URL="file://$manifest_file" \
    GITHUB_WORKFLOW_DIR="$workflow_dir" \
    CHECK_LATEST_TOOLS_ACTION_PINS_ONLY=1 \
    scripts/check_latest_tools.sh >/dev/null 2>&1; then
    echo "mutable service container tag was accepted" >&2
    exit 1
fi

echo "latest tool check tests passed"
