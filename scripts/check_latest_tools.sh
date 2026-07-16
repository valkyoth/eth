#!/usr/bin/env sh
set -eu

ci_file=".github/workflows/ci.yml"
rust_toolchain_file="${RUST_TOOLCHAIN_FILE:-rust-toolchain.toml}"
rust_stable_manifest_url="${RUST_STABLE_MANIFEST_URL:-https://static.rust-lang.org/dist/channel-rust-stable.toml}"

pinned_rust_version() {
    sed -n 's/^channel = "\([0-9][0-9.]*\)"$/\1/p' "$rust_toolchain_file" |
        head -n 1
}

latest_stable_rust_version() {
    curl -fsSL "$rust_stable_manifest_url" |
        sed -n '/^\[pkg\.rust\]$/,/^\[/ {
            s/^version = "\([0-9][0-9.]*\) .*/\1/p
        }' |
        head -n 1
}

check_latest_rust() {
    pinned="$(pinned_rust_version)"
    latest="$(latest_stable_rust_version)"

    if [ -z "$pinned" ]; then
        echo "missing pinned Rust version in ${rust_toolchain_file}" >&2
        exit 1
    fi

    if [ -z "$latest" ]; then
        echo "could not determine latest stable Rust version" >&2
        exit 1
    fi

    if [ "$pinned" != "$latest" ]; then
        echo "Rust is not latest stable: pinned ${pinned}, latest ${latest}" >&2
        exit 1
    fi
}

ci_tool_version() {
    tool="$1"
    sed -n "s/.*cargo install --locked ${tool} --version \\([0-9][^ ]*\\).*/\\1/p" "$ci_file" | head -n 1
}

latest_crate_version() {
    crate="$1"
    cargo info "$crate" | sed -n 's/^version: //p' | head -n 1
}

check_cargo_tool() {
    tool="$1"
    pinned="$(ci_tool_version "$tool")"
    latest="$(latest_crate_version "$tool")"

    if [ -z "$pinned" ]; then
        echo "missing pinned CI version for ${tool}" >&2
        exit 1
    fi

    if [ -z "$latest" ]; then
        echo "could not determine latest crates.io version for ${tool}" >&2
        exit 1
    fi

    if [ "$pinned" != "$latest" ]; then
        echo "${tool} is not latest: pinned ${pinned}, latest ${latest}" >&2
        exit 1
    fi
}

checkout_pin_line() {
    sed -n 's/.*uses: actions\/checkout@\([0-9a-f]\{40\}\) # \(v[0-9][0-9.]*\).*/\1 \2/p' "$ci_file" | head -n 1
}

check_all_actions_sha_pinned() {
    failed=0
    while IFS= read -r ref; do
        case "$ref" in
            "")
                ;;
            [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f])
                ;;
            *)
                echo "GitHub Actions ref is not a full 40-character SHA: ${ref}" >&2
                failed=1
                ;;
        esac
    done <<EOF
$(sed -n 's/^[[:space:]]*uses: [^@][^@]*@\([^[:space:]]*\).*/\1/p' "$ci_file")
EOF
    [ "$failed" -eq 0 ]
}

latest_checkout_tag() {
    git ls-remote --tags --refs https://github.com/actions/checkout.git 'refs/tags/v*' |
        sed 's#.*refs/tags/##' |
        grep -E '^v[0-9]+(\.[0-9]+)*$' |
        sort -V |
        tail -n 1
}

checkout_tag_sha() {
    tag="$1"
    git ls-remote --tags --refs https://github.com/actions/checkout.git "refs/tags/${tag}" |
        awk '{ print $1 }'
}

check_checkout_action() {
    pin_line="$(checkout_pin_line)"
    if [ -z "$pin_line" ]; then
        echo "actions/checkout must be pinned to a full SHA with a semver tag comment" >&2
        exit 1
    fi

    pinned_sha="$(printf '%s\n' "$pin_line" | awk '{ print $1 }')"
    pinned_tag="$(printf '%s\n' "$pin_line" | awk '{ print $2 }')"
    latest_tag="$(latest_checkout_tag)"

    if [ -z "$latest_tag" ]; then
        echo "could not determine latest actions/checkout tag" >&2
        exit 1
    fi

    if [ "$pinned_tag" != "$latest_tag" ]; then
        echo "actions/checkout is not latest: pinned ${pinned_tag}, latest ${latest_tag}" >&2
        exit 1
    fi

    latest_sha="$(checkout_tag_sha "$latest_tag")"
    if [ -z "$latest_sha" ]; then
        echo "could not determine SHA for actions/checkout ${latest_tag}" >&2
        exit 1
    fi

    if [ "$pinned_sha" != "$latest_sha" ]; then
        echo "actions/checkout ${latest_tag} SHA mismatch: pinned ${pinned_sha}, latest ${latest_sha}" >&2
        exit 1
    fi
}

check_latest_rust

if [ "${CHECK_LATEST_TOOLS_RUST_ONLY:-0}" = "1" ]; then
    exit 0
fi

check_cargo_tool cargo-deny
check_cargo_tool cargo-audit
check_cargo_tool cargo-sbom
check_all_actions_sha_pinned
check_checkout_action
