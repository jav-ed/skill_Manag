# translate.sh - argument/env translation layer Go oracle -> skillmirror (sourced, not executed).
#
# run_Scenarios.sh calls oracle_translate/oracle_pre_run from run() when they exist. The Go
# scenarios stay untouched; this file maps each Go invocation onto the Rust grammar:
#
#   Go tool call                               Rust call                              plan call (for the set comparison)
#   -----------------------------------------  -------------------------------------  ----------------------------------
#   go  --dry-run [flags]                      sync --dry-run [flags]                 sync --dry-run --json --all [flags]
#   drv sync [flags]                           sync --yes [flags]                     sync --dry-run --json --all [flags]
#   drv sync --dry-run [flags]                 sync --dry-run [flags]                 sync --dry-run --json --all [flags]
#   drv push [flags]                           push --yes [flags]                     push --dry-run --json --all [flags]
#   go  delete ARGS...                         delete ARGS... --yes                   (none)
#   go  list ARGS... / --help / --bogus /      same arguments, unchanged              (none)
#       unknown subcommand / flags only (menu)
#
# Environment: SKILL_MANAG_VAULT -> SKILLMIRROR_VAULT, SKILL_MANAG_ROOT -> SKILLMIRROR_ROOT.
# Every other SKILL_MANAG_* variable is passed through UNCHANGED on purpose: the Rust tool
# treats it as a hard error (rust_Rewrite.md, section 3) and that is what the scenario must show.
#
# Vault pointer file: Go reads $HOME/.config/skill_Manag/vault, Rust reads
# $XDG_CONFIG_HOME/skillmirror/vault. oracle_pre_run copies the Go pointer (bytes as they are,
# including whitespace or empty content) or removes the Rust one when the scenario removed it.

RUST_BIN=${PARITY_RUST_BIN:?PARITY_RUST_BIN must point at the skillmirror copy under test}

oracle_pre_run() {
	mkdir -p "$H/.config/skillmirror"
	if [ -e "$H/.config/skill_Manag/vault" ]; then
		cp "$H/.config/skill_Manag/vault" "$H/.config/skillmirror/vault"
	else
		rm -f "$H/.config/skillmirror/vault"
	fi
}

oracle_translate() {
	local kind=$1 first rest x dry mode
	shift
	TR_BIN=$RUST_BIN TR_NAME=skillmirror TR_ARGS=() TR_PLAN=() TR_ENV=()
	for x in "${ENV_EXTRA[@]}"; do
		case $x in
		SKILL_MANAG_VAULT=*) TR_ENV+=("SKILLMIRROR_VAULT=${x#*=}") ;;
		SKILL_MANAG_ROOT=*) TR_ENV+=("SKILLMIRROR_ROOT=${x#*=}") ;;
		*) TR_ENV+=("$x") ;;
		esac
	done

	if [ "$kind" = drv ]; then
		mode=$1
		shift
		dry=0 rest=()
		for x in "$@"; do
			if [ "$x" = --dry-run ]; then dry=1; else rest+=("$x"); fi
		done
		if [ $dry = 1 ]; then TR_ARGS=("$mode" --dry-run "${rest[@]}"); else TR_ARGS=("$mode" --yes "${rest[@]}"); fi
		TR_PLAN=("$mode" --dry-run --json --all "${rest[@]}")
		return
	fi

	first=${1:-}
	case $first in
	--dry-run)
		shift
		TR_ARGS=(sync --dry-run "$@")
		TR_PLAN=(sync --dry-run --json --all "$@")
		;;
	delete)
		TR_ARGS=("$@" --yes)
		;;
	*)
		TR_ARGS=("$@")
		;;
	esac
}
