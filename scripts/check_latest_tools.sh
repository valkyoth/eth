#!/usr/bin/env sh
set -eu

workflow_dir="${GITHUB_WORKFLOW_DIR:-.github/workflows}"
ci_file="${CI_WORKFLOW_FILE:-${workflow_dir}/ci.yml}"
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
    check_cargo_tool_version "$tool" "$pinned"
}

check_cargo_tool_version() {
    tool="$1"
    pinned="$2"
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

workflow_files() {
    for workflow in "$workflow_dir"/*.yml "$workflow_dir"/*.yaml; do
        if [ -f "$workflow" ]; then
            printf '%s\n' "$workflow"
        fi
    done
}

checkout_uses() {
    ruby scripts/check_action_pins.rb "$workflow_dir" --list-checkouts
}

checkout_pin_lines() {
    while IFS= read -r workflow; do
        sed -n 's/.*uses: actions\/checkout@\([0-9a-f]\{40\}\) # \(v[0-9][0-9.]*\).*/\1 \2/p' "$workflow"
    done <<EOF
$(workflow_files)
EOF
}

check_checkout_pin_format() {
    checkout_count="$(checkout_uses | wc -l | tr -d ' ')"
    parsed_count="$(checkout_pin_lines | wc -l | tr -d ' ')"

    if [ "$checkout_count" -eq 0 ]; then
        echo "no actions/checkout use found in ${workflow_dir}" >&2
        exit 1
    fi

    if [ "$checkout_count" -ne "$parsed_count" ]; then
        echo "every actions/checkout use must be pinned to a full SHA with a semver tag comment" >&2
        exit 1
    fi
}

check_all_actions_sha_pinned() {
    if ! command -v ruby >/dev/null 2>&1; then
        echo "ruby with the standard YAML parser is required for Action pin validation" >&2
        exit 1
    fi
    ruby scripts/check_action_pins.rb "$workflow_dir"
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
    check_checkout_pin_format
    latest_tag="$(latest_checkout_tag)"

    if [ -z "$latest_tag" ]; then
        echo "could not determine latest actions/checkout tag" >&2
        exit 1
    fi

    latest_sha="$(checkout_tag_sha "$latest_tag")"
    if [ -z "$latest_sha" ]; then
        echo "could not determine SHA for actions/checkout ${latest_tag}" >&2
        exit 1
    fi

    while IFS=' ' read -r pinned_sha pinned_tag; do
        if [ "$pinned_tag" != "$latest_tag" ]; then
            echo "actions/checkout is not latest: pinned ${pinned_tag}, latest ${latest_tag}" >&2
            exit 1
        fi

        if [ "$pinned_sha" != "$latest_sha" ]; then
            echo "actions/checkout ${latest_tag} SHA mismatch: pinned ${pinned_sha}, latest ${latest_sha}" >&2
            exit 1
        fi
    done <<EOF
$(checkout_pin_lines)
EOF
}

check_latest_rust

if [ "${CHECK_LATEST_TOOLS_RUST_ONLY:-0}" = "1" ]; then
    exit 0
fi

if [ "${CHECK_LATEST_TOOLS_ACTION_PINS_ONLY:-0}" = "1" ]; then
    check_all_actions_sha_pinned
    check_checkout_pin_format
    exit 0
fi

check_cargo_tool cargo-deny
check_cargo_tool cargo-audit
check_cargo_tool cargo-sbom
check_cargo_tool_version cargo-fuzz 0.13.2
check_all_actions_sha_pinned
check_checkout_action
