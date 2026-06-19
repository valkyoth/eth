#!/usr/bin/env sh
set -eu

tag="${1:-}"
case "$tag" in
    v[0-9]*.[0-9]*.[0-9]*) ;;
    *)
        echo "usage: scripts/validate-release-readiness.sh vX.Y.Z" >&2
        exit 2
        ;;
esac

version="${tag#v}"
release_notes="release-notes/RELEASE_NOTES_${version}.md"
pentest_report="security/pentest/${tag}.md"

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    echo "tag already exists locally: ${tag}" >&2
    exit 1
fi

if [ -f PENTEST.md ]; then
    echo "root PENTEST.md is temporary scratch input and must be removed" >&2
    exit 1
fi

if [ ! -f "$release_notes" ]; then
    echo "missing release notes: ${release_notes}" >&2
    exit 1
fi

if [ ! -s sbom/eth.spdx.json ]; then
    echo "missing or empty SBOM: sbom/eth.spdx.json" >&2
    exit 1
fi

if [ ! -f "$pentest_report" ]; then
    echo "missing pentest report: ${pentest_report}" >&2
    exit 1
fi

grep -q '^Status: PASS$' "$pentest_report"
grep -Eq '^Commit: [0-9a-f]{40}$' "$pentest_report"
grep -Eq '^Tester: .+' "$pentest_report"
grep -Eq '^Scope: .+' "$pentest_report"
grep -Eq '^Date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$pentest_report"

head_commit="$(git rev-parse HEAD)"
report_commit="$(sed -n 's/^Commit: //p' "$pentest_report")"
if [ "$report_commit" != "$head_commit" ]; then
    echo "pentest report commit ${report_commit} does not match HEAD ${head_commit}" >&2
    exit 1
fi
