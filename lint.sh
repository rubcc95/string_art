#!/usr/bin/env bash
# Checks for the string_art workspace.
set -uo pipefail
cd "$(dirname "$0")" || exit 1

export CARGO_TERM_COLOR=never
if [ -n "${JOBS:-}" ]; then export CARGO_BUILD_JOBS="$JOBS"; fi

case "${1:-all}" in
syntax)
    find . -name '*.rs' -not -path './.git/*' -not -path './target/*' -print0 |
        xargs -0 rustfmt --edition 2024 --check
    ;;
core)
    cargo check -p string_art --all-targets
    ;;
backends)
    cargo check -p string_art_serial -p string_art_rayon --all-targets
    ;;
gpui)
    cargo check -p string_art_gpui --all-targets
    ;;
all)
    find . -name '*.rs' -not -path './.git/*' -not -path './target/*' -print0 |
        xargs -0 rustfmt --edition 2024 --check || exit 1
    cargo check -p string_art --all-targets &&
        cargo check -p string_art_serial -p string_art_rayon --all-targets &&
        cargo check -p string_art_gpui --all-targets
    ;;
*)
    echo "usage: $0 {syntax|core|backends|gpui|all}" >&2
    exit 2
    ;;
esac
