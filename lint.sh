#!/usr/bin/env bash
# Lint for the string_art workspace.
#
#   ./lint.sh syntax   # Level 0: parse all .rs files with rustfmt. Seconds, no deps.
#   ./lint.sh core     # Level 1: clippy on the crates without threads (geometry, grid, color).
#   ./lint.sh threads  # Level 2: the computation WITH threads (`CondSend`/`CondSync` = Send/Sync).
#   ./lint.sh local    # Level 3: the computation WITHOUT threads (empty traits: any Anchor works).
#   ./lint.sh all      # syntax + core + threads + local.
#
# Levels 2 and 3 are compiled in separate invocations on purpose: within
# a single one, cargo unifies features and only one of the two sides would be compiled.
#
# Variables: JOBS=<n> (cargo jobs), FIX=1 (runs cargo clippy --fix on the core level).
set -uo pipefail
cd "$(dirname "$0")" || exit 1

CORE=(string_art_geometry string_art_grid string_art_color)

export CARGO_TERM_COLOR=never
if [ -n "${JOBS:-}" ]; then
	export CARGO_BUILD_JOBS="$JOBS"
fi

rust_sources() {
	find . -name '*.rs' -not -path './.git/*' -not -path './target/*'
}

# --- Level 0: parsing only ------------------------------------------------
level_syntax() {
	local out err files
	out=$(mktemp) err=$(mktemp)
	# shellcheck disable=SC2046
	files=$(rust_sources)
	echo "Parsing $(echo "$files" | wc -l) files with $(rustfmt -V)..."
	# rustfmt --check also returns 1 for simple formatting differences,
	# so real syntax errors are read from stderr.
	echo "$files" | xargs rustfmt --edition 2024 --check >"$out" 2>"$err"
	local parse_errors fmt_lines
	parse_errors=$(grep -cE '^error(\[|:)' "$err" 2>/dev/null | head -1)
	parse_errors=${parse_errors:-0}
	fmt_lines=$(wc -l <"$out")
	if [ "$parse_errors" -gt 0 ]; then
		echo "SYNTAX ERRORS: $parse_errors"
		grep -E '^error' -A3 "$err" | head -40
		return 1
	fi
	echo "Syntax OK: 0 parse errors."
	echo "Format: $fmt_lines lines of difference from rustfmt (does not block)."
	rm -f "$out" "$err"
}

# --- Levels 1..3: clippy ----------------------------------------------------
# level_clippy <label> <features|""> <crate>...
level_clippy() {
	local label=$1 feats=$2
	shift 2
	local crates=("$@")
	local log
	log=$(mktemp)
	local fix=""
	[ "${FIX:-0}" = "1" ] && fix="--fix --allow-dirty --allow-staged"

	echo "clippy [$label] on: ${crates[*]}${feats:+ (features: $feats)}"
	local args=() fargs=() c
	for c in "${crates[@]}"; do args+=(-p "$c"); done
	[ -n "$feats" ] && fargs+=(--features "$feats")
	local start=$SECONDS
	# shellcheck disable=SC2086
	cargo check "${args[@]}" --all-targets "${fargs[@]}" $fix >"$log" 2>&1
	local rc=$?
	echo "Time: $((SECONDS - start))s"

	local errors warnings
	errors=$(grep -cE '^error' "$log")
	warnings=$(grep -E '^warning: [a-z]' "$log" | wc -l)
	echo "Errors: $errors | Warnings: $warnings"
	if [ "$warnings" -gt 0 ]; then
		echo "--- warnings by type:"
		grep -E '^warning: [a-z]' "$log" | sed 's/^warning: //' | sort | uniq -c | sort -rn
		echo "--- detail:"
		grep -E '^warning: [a-z]' -A2 "$log" | grep -E '^warning:|--> ' | paste - - 2>/dev/null | head -40
	fi
	[ "$errors" -gt 0 ] && grep -E '^error' -A5 "$log" | head -40
	echo "Full log: $log"
	[ "$rc" -eq 0 ]
}

# The two sides of the computation go in separate invocations: with `-p string_art`
# alone, `string_art_sync` inherits `default-features = false` and the
# single-threaded path is compiled; with `--features threads` the rayon one is compiled.
level_threads() {
	local rc=0
	level_clippy threads threads string_art_sync || rc=1
	echo
	level_clippy threads threads string_art || rc=1
	return $rc
}

level_local() {
	local rc=0
	level_clippy local "" string_art_sync || rc=1
	echo
	level_clippy local "" string_art || rc=1
	return $rc
}

case "${1:-core}" in
syntax) level_syntax ;;
core) level_clippy core "" "${CORE[@]}" ;;
threads) level_threads ;;
local) level_local ;;
all)
	level_syntax || exit 1
	echo
	level_clippy core "" "${CORE[@]}" || exit 1
	echo
	level_threads || exit 1
	echo
	level_local || exit 1
	;;
*)
	echo "usage: $0 {syntax|core|threads|local|all}" >&2
	exit 2
	;;
esac
