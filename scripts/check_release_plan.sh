#!/usr/bin/env sh
set -eu

plan=${1:-docs/RELEASE_PLAN.md}

if [ ! -f "$plan" ]; then
    echo "release plan does not exist: $plan" >&2
    exit 1
fi

awk -v plan="$plan" '
function problem(message) {
    printf "%s:%d: %s: %s\n", plan, start, version, message > "/dev/stderr"
    problems++
}

function finish_release( normalized, required) {
    if (version == "") {
        return
    }

    releases++
    if (status_count != 1) {
        problem("must contain exactly one Status section")
    } else if (!status_content) {
        problem("Status section must not be empty")
    }
    if (goal_count != 1) {
        problem("must contain exactly one Goal section")
    } else if (!goal_content) {
        problem("Goal section must not be empty")
    }
    if (deliverables_count != 1) {
        problem("must contain exactly one Deliverables section")
    } else if (!deliverables_content) {
        problem("Deliverables section must not be empty")
    }
    if (verification_count != 1) {
        problem("must contain exactly one Verification section")
    } else if (!verification_content) {
        problem("Verification section must not be empty")
    }
    if (exit_count != 1) {
        problem("must contain exactly one Exit criteria section")
    } else if (!exit_content) {
        problem("Exit criteria section must not be empty")
    }

    normalized = exit_text
    gsub(/`/, "", normalized)
    gsub(/[[:space:]]+/, " ", normalized)
    required = version " implementation stop reached. Run pentest for this exact commit."
    sub(/^[[:space:]]+/, "", normalized)
    sub(/[[:space:]]+$/, "", normalized)
    if (length(normalized) < length(required) ||
        substr(normalized, length(normalized) - length(required) + 1) != required) {
        problem("Exit criteria must end with the version-specific exact-commit pentest stop")
    }
}

function reset_release() {
    section = ""
    status_count = goal_count = deliverables_count = verification_count = exit_count = 0
    status_content = 0
    goal_content = deliverables_content = verification_content = exit_content = 0
    exit_text = ""
    section_order = 0
}

/^#{2,3} v[^ ]+ - / {
    finish_release()
    version = $2
    start = NR
    reset_release()
    if (seen[version]++) {
        problem("duplicate release heading")
    }
    next
}

/^#{2,3} v/ {
    printf "%s:%d: malformed release heading: %s\n", plan, NR, $0 > "/dev/stderr"
    problems++
    next
}

version != "" && /^## / {
    finish_release()
    version = ""
    section = ""
    next
}

version != "" && /^Status:/ {
    if (section_order != 0) problem("Status must appear before Goal")
    status_count++
    section_line = $0
    sub(/^Status:[[:space:]]*/, "", section_line)
    if (section_line != "") status_content = 1
    section = ""
    next
}
version != "" && /^Goal:/ {
    if (section_order != 0) problem("Goal must be the first required section")
    section_order = 1
    goal_count++
    section = "goal"
    section_line = $0
    sub(/^Goal:[[:space:]]*/, "", section_line)
    if (section_line != "") goal_content = 1
    next
}
version != "" && /^Deliverables:/ {
    if (section_order != 1) problem("Deliverables must follow Goal")
    section_order = 2
    deliverables_count++
    section = "deliverables"
    section_line = $0
    sub(/^Deliverables:[[:space:]]*/, "", section_line)
    if (section_line != "") deliverables_content = 1
    next
}
version != "" && /^Verification:/ {
    if (section_order != 2) problem("Verification must follow Deliverables")
    section_order = 3
    verification_count++
    section = "verification"
    section_line = $0
    sub(/^Verification:[[:space:]]*/, "", section_line)
    if (section_line != "") verification_content = 1
    next
}
version != "" && /^Exit criteria:/ {
    if (section_order != 3) problem("Exit criteria must follow Verification")
    section_order = 4
    exit_count++
    section = "exit"
    section_line = $0
    sub(/^Exit criteria:[[:space:]]*/, "", section_line)
    if (section_line != "") {
        exit_content = 1
        exit_text = exit_text " " section_line
    }
    next
}

version != "" && $0 !~ /^[[:space:]]*$/ {
    if (section == "goal") goal_content = 1
    if (section == "deliverables") deliverables_content = 1
    if (section == "verification") verification_content = 1
    if (section == "exit") exit_content = 1
}
version != "" && section == "exit" {
    exit_text = exit_text " " $0
}

END {
    finish_release()
    if (releases == 0) {
        print plan ": no release milestones found" > "/dev/stderr"
        problems++
    }
    if (problems) {
        exit 1
    }
    printf "validated %d release plan milestones\n", releases
}
' "$plan"
