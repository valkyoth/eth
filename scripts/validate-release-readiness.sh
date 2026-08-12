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
publish_tag="${ETH_RELEASE_PUBLISH_TAG:-}"
release_stage=""
public_baseline=""
review_baseline=""

if [ -f release-crates.toml ]; then
    metadata_version="$(python3 -c 'import tomllib; print(tomllib.load(open("release-crates.toml", "rb"))["release"]["version"])')"
    release_stage="$(python3 -c 'import tomllib; print(tomllib.load(open("release-crates.toml", "rb"))["release"]["stage"])')"
    public_baseline="$(python3 -c 'import tomllib; print(tomllib.load(open("release-crates.toml", "rb"))["release"]["baseline"])')"
    review_baseline="$(python3 -c 'import tomllib; print(tomllib.load(open("release-crates.toml", "rb"))["release"]["review_baseline"])')"
    if [ "$metadata_version" != "$version" ]; then
        echo "release metadata version ${metadata_version} does not match ${version}" >&2
        exit 1
    fi
fi

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    if [ "$publish_tag" != "$tag" ]; then
        echo "tag already exists locally: ${tag}" >&2
        exit 1
    fi

    tag_commit="$(git rev-list -n 1 "$tag")"
    head_commit="$(git rev-parse HEAD)"
    if [ "$tag_commit" != "$head_commit" ]; then
        echo "publish tag ${tag} does not point at HEAD" >&2
        exit 1
    fi
elif [ -n "$publish_tag" ]; then
    echo "publish tag context requires existing tag: ${tag}" >&2
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
scripts/generate-sbom.sh --check

if [ ! -f "$pentest_report" ]; then
    echo "missing pentest report: ${pentest_report}" >&2
    exit 1
fi

if ! git cat-file -e "HEAD:${pentest_report}" 2>/dev/null; then
    echo "pentest report must be committed in tag candidate: ${pentest_report}" >&2
    exit 1
fi

grep -q '^Status: PASS$' "$pentest_report"
grep -Eq '^Reviewed-Commit: [0-9a-f]{40}$' "$pentest_report"
grep -Eq '^Tester: .+' "$pentest_report"
grep -Eq '^Scope: .+' "$pentest_report"
grep -Eq '^Date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$pentest_report"

if [ -n "$release_stage" ] && [ "$version" != "0.55.0" ]; then
    assessment="$(sed -n 's/^Assessment: //p' "$pentest_report")"
    report_baseline="$(sed -n 's/^Baseline: //p' "$pentest_report")"
    range_end="$(sed -n 's/^Range-End: //p' "$pentest_report")"
    publication="$(sed -n 's/^Publication: //p' "$release_notes")"
    if [ "$range_end" != "$tag" ]; then
        echo "pentest Range-End must be ${tag}" >&2
        exit 1
    fi
    if [ "$release_stage" = "internal" ]; then
        checkpoint="$(python3 -c 'import sys; sys.path.insert(0, "scripts"); import release_train; print("v" + release_train.next_public_checkpoint(sys.argv[1]))' "$version")"
        if [ "$assessment" != "INCREMENTAL" ] || [ "$report_baseline" != "v${review_baseline}" ]; then
            echo "internal pentest must be INCREMENTAL from v${review_baseline}" >&2
            exit 1
        fi
        if [ "$publication" != "DEFERRED TO ${checkpoint}" ]; then
            echo "internal release notes must defer publication to ${checkpoint}" >&2
            exit 1
        fi
    elif [ "$release_stage" = "public" ]; then
        if [ "$assessment" != "CUMULATIVE" ] || [ "$report_baseline" != "v${public_baseline}" ]; then
            echo "public checkpoint pentest must be CUMULATIVE from v${public_baseline}" >&2
            exit 1
        fi
        if [ "$publication" != "PENDING" ]; then
            echo "public checkpoint release notes must record Publication: PENDING" >&2
            exit 1
        fi
        milestones="$(python3 -c 'import tomllib; print(" ".join(tomllib.load(open("release-crates.toml", "rb"))["release"]["cumulative_milestones"]))')"
        for milestone in $milestones; do
            [ "$milestone" = "$version" ] && continue
            report="security/pentest/v${milestone}.md"
            if ! git cat-file -e "v${milestone}:${report}" 2>/dev/null; then
                echo "public checkpoint lacks tagged pentest report for v${milestone}" >&2
                exit 1
            fi
        done
    else
        echo "unknown release stage: ${release_stage}" >&2
        exit 1
    fi
fi

reviewed_commit="$(sed -n 's/^Reviewed-Commit: //p' "$pentest_report")"
if ! git cat-file -e "${reviewed_commit}^{commit}" 2>/dev/null; then
    echo "reviewed commit ${reviewed_commit} was not found" >&2
    exit 1
fi

head_parent="$(git rev-parse HEAD^)"
if [ "$reviewed_commit" != "$head_parent" ]; then
    echo "reviewed commit ${reviewed_commit} does not match first parent ${head_parent}" >&2
    exit 1
fi

changed_paths="$(git diff --name-only "$reviewed_commit" HEAD)"
if [ "$changed_paths" != "$pentest_report" ]; then
    echo "release report commit may only change ${pentest_report}" >&2
    echo "$changed_paths" >&2
    exit 1
fi
