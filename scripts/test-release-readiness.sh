#!/usr/bin/env sh
set -eu

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

script="$tmp/scripts/validate-release-readiness.sh"

mkdir -p "$tmp/scripts" "$tmp/release-notes" "$tmp/security/pentest" "$tmp/sbom"
cp scripts/validate-release-readiness.sh "$script"

cd "$tmp"
git init -q
git config user.email "release-readiness@example.invalid"
git config user.name "Release Readiness Test"
printf 'fixture\n' >README.md
git add README.md
git commit -q -m "fixture"

head_commit="$(git rev-parse HEAD)"

assert_fails_with() {
    expected="$1"
    shift

    if "$@" >"$tmp/stdout" 2>"$tmp/stderr"; then
        echo "expected command to fail: $*" >&2
        exit 1
    fi

    if ! grep -q "$expected" "$tmp/stderr"; then
        echo "expected stderr to contain: $expected" >&2
        echo "actual stderr:" >&2
        cat "$tmp/stderr" >&2
        exit 1
    fi
}

write_release_notes() {
    version="$1"
    printf '# Release %s\n' "$version" >"release-notes/RELEASE_NOTES_${version}.md"
}

write_sbom() {
    printf '{"spdxVersion":"SPDX-2.3"}\n' >sbom/eth.spdx.json
}

write_pentest() {
    tag="$1"
    commit="$2"
    cat >"security/pentest/${tag}.md" <<EOF
Status: PASS
Commit: ${commit}
Tester: Release Readiness Test
Scope: Fixture release metadata.
Date: 2026-06-19
EOF
}

assert_fails_with "usage: scripts/validate-release-readiness.sh vX.Y.Z" \
    "$script" "0.2.0"

git tag v9.9.9
assert_fails_with "tag already exists locally: v9.9.9" "$script" "v9.9.9"

printf 'scratch\n' >PENTEST.md
assert_fails_with "root PENTEST.md is temporary scratch input" \
    "$script" "v0.2.0"
rm PENTEST.md

assert_fails_with "missing release notes: release-notes/RELEASE_NOTES_0.2.0.md" \
    "$script" "v0.2.0"

write_release_notes "0.2.0"
assert_fails_with "missing or empty SBOM: sbom/eth.spdx.json" \
    "$script" "v0.2.0"

write_sbom
assert_fails_with "missing pentest report: security/pentest/v0.2.0.md" \
    "$script" "v0.2.0"

write_pentest "v0.2.0" "0000000000000000000000000000000000000000"
assert_fails_with "pentest report commit 0000000000000000000000000000000000000000 does not match HEAD" \
    "$script" "v0.2.0"

write_pentest "v0.2.0" "$head_commit"
"$script" "v0.2.0"
