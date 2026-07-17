#!/usr/bin/env sh
set -eu

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

valid="$tmp_dir/valid.md"
cat >"$valid" <<'EOF'
### v0.1.0 - Valid Fixture

Status: planned.

Goal: provide a valid fixture.

Deliverables:

- one bounded deliverable.

Verification:

- one deterministic check.

Exit criteria:

- The fixture is complete.
- `v0.1.0 implementation stop reached. Run pentest for this exact commit.`
EOF

scripts/check_release_plan.sh "$valid" >/dev/null

expect_failure() {
    fixture=$1
    if scripts/check_release_plan.sh "$fixture" >/dev/null 2>&1; then
        echo "release-plan checker unexpectedly accepted $fixture" >&2
        exit 1
    fi
}

missing_goal="$tmp_dir/missing-goal.md"
sed '/^Goal:/d' "$valid" >"$missing_goal"
expect_failure "$missing_goal"

missing_status="$tmp_dir/missing-status.md"
sed '/^Status:/d' "$valid" >"$missing_status"
expect_failure "$missing_status"

wrong_order="$tmp_dir/wrong-order.md"
sed 's/^Deliverables:/Verification:/; s/^Verification:/Deliverables:/' \
    "$valid" >"$wrong_order"
expect_failure "$wrong_order"

wrong_version="$tmp_dir/wrong-version.md"
sed 's/v0\.1\.0 implementation/v0.1.1 implementation/' \
    "$valid" >"$wrong_version"
expect_failure "$wrong_version"

not_final="$tmp_dir/not-final.md"
sed '$a\- Additional work remains.' "$valid" >"$not_final"
expect_failure "$not_final"

duplicate="$tmp_dir/duplicate.md"
cp "$valid" "$duplicate"
sed -n '1,$p' "$valid" >>"$duplicate"
expect_failure "$duplicate"

echo "release plan checker tests passed"
