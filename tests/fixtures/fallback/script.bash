#!/usr/bin/env bash
set -euo pipefail

get_exclusion_list() {
    printf '%s\n' \
        target \
        .jj \
        .git
}

calculate_excluded_size() {
    local total=0
    for path in "$@"; do
        total=$((total + path))
    done
    echo "$total"
}

format_bytes() {
    printf '%sB\n' "$1"
}
