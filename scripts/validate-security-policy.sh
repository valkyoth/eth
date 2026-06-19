#!/usr/bin/env sh
set -eu

grep -R '#!\[forbid(unsafe_code)\]' crates >/dev/null
grep -q 'unknown-git = "deny"' deny.toml
grep -q 'unknown-registry = "deny"' deny.toml
grep -q 'panic = "abort"' Cargo.toml
grep -q 'CodeQL default setup' SECURITY.md
test -f docs/github-security-settings.md
grep -q 'CodeQL analysis default setup is active' docs/github-security-settings.md
test -f docs/secret-handling-policy.md
