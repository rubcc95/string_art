#!/usr/bin/env bash
# Lint del workspace de string_art en Android/Termux.
#
#   ./lint.sh syntax   # Nivel 0: parseo de todos los .rs con rustfmt. Segundos, sin deps.
#   ./lint.sh core     # Nivel 1: clippy sobre los crates sin hilos (geometría, rejilla, color).
#   ./lint.sh threads  # Nivel 2: el cálculo CON hilos (`CondSend`/`CondSync` = Send/Sync).
#   ./lint.sh local    # Nivel 3: el cálculo SIN hilos (traits vacíos: vale cualquier Anchor).
#   ./lint.sh all      # syntax + core + threads + local.
#
# Los niveles 2 y 3 se compilan en invocaciones separadas a propósito: dentro de
# una sola, cargo unifica features y solo se compilaría una de las dos caras.
#
# Variables: JOBS=1 (jobs de cargo), FIX=1 (aplica cargo clippy --fix en el nivel core).
set -uo pipefail
cd "$(dirname "$0")" || exit 1

CORE=(string_art_geometry string_art_grid string_art_color)

# --- Arreglos del entorno Android -------------------------------------------
# 1) rustc enlaza con -lunwind, que aquí solo existe como $PREFIX/lib/libunwind.a
#    y el linker no lo busca por defecto.
if [ -n "${PREFIX:-}" ] && [ -e "$PREFIX/lib/libunwind.a" ]; then
	export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-L$PREFIX/lib"
fi
# 2) cargo-clippy no sabe ejecutar su propio clippy-driver en este entorno;
#    usarlo como wrapper del compilador hace exactamente lo mismo.
if [ -x "${PREFIX:-}/bin/clippy-driver" ]; then
	export RUSTC_WORKSPACE_WRAPPER="$PREFIX/bin/clippy-driver"
fi
export CARGO_BUILD_JOBS="${JOBS:-1}"
export CARGO_TERM_COLOR=never

# Android no tiene /tmp; mktemp necesita un directorio escribible.
: "${TMPDIR:=${HARNESS_SCRATCH:-$PWD/.harness/tmp}}"
mkdir -p "$TMPDIR"
export TMPDIR

rust_sources() {
	find . -name '*.rs' -not -path './.git/*' -not -path './.harness/*' -not -path './target/*'
}

# --- Nivel 0: solo parseo ------------------------------------------------
level_syntax() {
	local out err files
	out=$(mktemp) err=$(mktemp)
	# shellcheck disable=SC2046
	files=$(rust_sources)
	echo "Parseando $(echo "$files" | wc -l) ficheros con $(rustfmt -V)..."
	# rustfmt --check devuelve 1 también por simples diferencias de formato,
	# así que los errores reales de sintaxis se leen en stderr.
	echo "$files" | xargs rustfmt --edition 2024 --check >"$out" 2>"$err"
	local parse_errors fmt_lines
	parse_errors=$(grep -cE '^error(\[|:)' "$err" 2>/dev/null | head -1)
	parse_errors=${parse_errors:-0}
	fmt_lines=$(wc -l <"$out")
	if [ "$parse_errors" -gt 0 ]; then
		echo "ERRORES DE SINTAXIS: $parse_errors"
		grep -E '^error' -A3 "$err" | head -40
		return 1
	fi
	echo "Sintaxis OK: 0 errores de parseo."
	echo "Formato: $fmt_lines líneas de diferencia con rustfmt (no bloquea)."
	rm -f "$out" "$err"
}

# --- Nivel 1..3: clippy ----------------------------------------------------
# level_clippy <etiqueta> <features|""> <crate>...
level_clippy() {
	local label=$1 feats=$2
	shift 2
	local crates=("$@")
	local log
	log=$(mktemp)
	local fix=""
	[ "${FIX:-0}" = "1" ] && fix="--fix --allow-dirty --allow-staged"

	echo "clippy [$label] sobre: ${crates[*]}${feats:+ (features: $feats)} (JOBS=$CARGO_BUILD_JOBS)"
	local args=() fargs=() c
	for c in "${crates[@]}"; do args+=(-p "$c"); done
	[ -n "$feats" ] && fargs+=(--features "$feats")
	local start=$SECONDS
	# shellcheck disable=SC2086
	cargo check "${args[@]}" --all-targets "${fargs[@]}" $fix >"$log" 2>&1
	local rc=$?
	echo "Tiempo: $((SECONDS - start))s"

	local errors warnings
	errors=$(grep -cE '^error' "$log")
	warnings=$(grep -E '^warning: [a-z]' "$log" | wc -l)
	echo "Errores: $errors | Avisos: $warnings"
	if [ "$warnings" -gt 0 ]; then
		echo "--- avisos por tipo:"
		grep -E '^warning: [a-z]' "$log" | sed 's/^warning: //' | sort | uniq -c | sort -rn
		echo "--- detalle:"
		grep -E '^warning: [a-z]' -A2 "$log" | grep -E '^warning:|--> ' | paste - - 2>/dev/null | head -40
	fi
	[ "$errors" -gt 0 ] && grep -E '^error' -A5 "$log" | head -40
	echo "Log completo: $log"
	[ "$rc" -eq 0 ]
}

# Las dos caras del cálculo van en invocaciones separadas: con `-p string_art`
# solo, `string_art_sync` hereda `default-features = false` y se compila el
# camino monohilo; con `--features threads` se compila el de rayon.
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
	echo "uso: $0 {syntax|core|threads|local|all}" >&2
	exit 2
	;;
esac
