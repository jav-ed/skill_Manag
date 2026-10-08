#!/usr/bin/env bash
# run_Scenarios.sh - record golden outputs of the frozen Go skill_Manag (commit c7310f9).
#
#   run_Scenarios.sh [-o OUTDIR] [-l] [scenario ...]
#     -o OUTDIR   where to write the golden tree (default: $PARITY_WORK/golden, see paths_Parity.sh)
#     -l          list scenario names and exit
#     scenario    run only these (default: all)
#
# Every scenario builds a FRESH fixture in a temp dir (outside any git repo):
#   <TMP>/vault     copy of internal/testdata/vault, git init + add + commit
#   <TMP>/projects  copy of internal/testdata/projects (+ empty .git dirs like internal/fixture_test.go)
#   <TMP>/home      HOME and XDG_CONFIG_HOME, with the vault pointer file inside
# The tools run with `env -i`, no controlling TTY (setsid) and stdin from /dev/null, so
# the real ~/.config/skill_Manag and the real vault/scan root are never reached.
#
# Per step it records into OUTDIR/<scenario>/<NN>_<label>/:
#   cmd  stdout  stderr  exit_code  projects.manifest
# and per scenario: DESCRIPTION, rest.manifest (vault files, home, anything else outside projects/).
# Temp paths are normalized to <TMP>; the per-project blocks of the Go dry-run output are
# sorted by project path (Go prints them in random map order).
#
# Two tools are driven:
#   go   $PARITY_WORK/skill_go      the unmodified Go binary (everything non-interactive)
#   drv  $PARITY_WORK/skill_go_drv  TTY-free driver for sync-apply and push (see README.md)
#
# Env knobs: ORACLE_OUT, ORACLE_GO_BIN, ORACLE_DRV_BIN, ORACLE_KEEP_UNSORTED=1 (also write
# stdout.unsorted, used by check_Determinism.sh).
set -u
umask 022

# shellcheck source=paths_Parity.sh
source "$(dirname "${BASH_SOURCE[0]}")/paths_Parity.sh"
GO_BIN=${ORACLE_GO_BIN:-$PARITY_WORK/skill_go}
DRV_BIN=${ORACLE_DRV_BIN:-$PARITY_WORK/skill_go_drv}
OUT=${ORACLE_OUT:-$PARITY_WORK/golden}
KEEP_UNSORTED=${ORACLE_KEEP_UNSORTED:-0}
FIXTURES=$PARITY_WORK/src/internal/testdata
TOOLS=$PARITY_CODE/oracle_tools.py

die() { echo "run_Scenarios: $*" >&2; exit 1; }

# ---------------------------------------------------------------- scenario helpers
# State used by the scenario functions (set by fresh_fixture):
#   FX V P H   fixture dir, vault, projects root, home dir
#   PA PB PC PD   the four fixture projects
#   FLAGS      (--vault V --root P)

desc() { DESC=$1; }

gitq() {
	env -i PATH="$PATH" HOME="$H" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
		GIT_AUTHOR_NAME=oracle GIT_AUTHOR_EMAIL=oracle@example.invalid \
		GIT_COMMITTER_NAME=oracle GIT_COMMITTER_EMAIL=oracle@example.invalid \
		git -c commit.gpgsign=false -c init.defaultBranch=main "$@"
}
vgit() { gitq -C "$V" "$@"; }
vault_commit() { vgit add -A && vgit commit -qm "${1:-change}"; }

# skill dir of a project: sk "$PA" coding
sk() { printf '%s/.agents/skills/%s' "$1" "$2"; }

skip_if_root() { [ "$(id -u)" = 0 ] && skip "needs a non-root user (permission bits are ignored for root)"; }

fresh_fixture() {
	FX=$WORK/$SCEN
	rm -rf "$FX"
	mkdir -p "$FX/home/.config/skill_Manag"
	V=$FX/vault P=$FX/projects H=$FX/home
	cp -a "$FIXTURES/vault" "$V"
	cp -a "$FIXTURES/projects" "$P"
	# stable modes regardless of the checkout's umask
	find "$V" "$P" -type d -exec chmod 755 {} +
	find "$V" "$P" -type f -exec chmod 644 {} +
	# empty .git next to every .agents, as internal/fixture_test.go addGitDirs does
	find "$P" -type d -name .agents -prune | while IFS= read -r a; do mkdir -p "$(dirname "$a")/.git"; done
	# the checked-in config.yaml points root at the REAL scan root: always repoint it
	sed -i "s|^root:.*|root: $P|" "$V/config.yaml"
	vgit init -q && vault_commit init >/dev/null
	printf '%s\n' "$V" >"$H/.config/skill_Manag/vault"
	PA=$P/org/team/project-a PB=$P/org/team/project-b PC=$P/org/project-c
	PD=$P/standalone/deep/nested/project-d
	FLAGS=(--vault "$V" --root "$P")
	ENV_EXTRA=() RUN_CWD= RUN_QUIET= DESC=
}

# a second, different vault + root so env/flag/pointer precedence is visible in the output
make_decoy() {
	mkdir -p "$FX/decoy/vault/decoyskill" "$FX/decoy/root/proj/.agents/skills/decoyskill"
	echo decoy >"$FX/decoy/vault/decoyskill/SKILL.md"
	echo old >"$FX/decoy/root/proj/.agents/skills/decoyskill/SKILL.md"
	printf 'root: %s\n' "$FX/decoy/root" >"$FX/decoy/vault/config.yaml"
}

# write a config.yaml into the vault (stdin = body)
set_config() { cat >"$V/config.yaml"; }

norm() { python3 "$TOOLS" normalize --tmp "$FX" "$@"; }

# run <label> <go|drv> args...   (honours ENV_EXTRA, RUN_CWD, RUN_QUIET; resets them after)
run() {
	local label=$1 kind=$2 bin name
	shift 2
	case $kind in
	go) bin=$GO_BIN name=skill_go ;;
	drv) bin=$DRV_BIN name=skill_go_drv ;;
	*) die "bad tool kind $kind" ;;
	esac
	local cwd=${RUN_CWD:-$FX}
	local -a envv=(PATH="$PATH" HOME="$H" XDG_CONFIG_HOME="$H/.config" LANG=C.UTF-8 TERM=dumb
		NO_COLOR=1 GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null)
	local -a extra=("${ENV_EXTRA[@]}")
	local -a args=("$@") plan=()
	local orig="$kind"
	for e in "$@"; do orig="$orig $(printf '%q' "$e")"; done
	# hook for the parity runner: oracle_translate may replace bin/name/args/env and ask for a
	# second "plan" invocation (TR_PLAN) or skip the step (TR_SKIP); oracle_pre_run syncs fixture state
	TR_SKIP=
	if declare -F oracle_translate >/dev/null; then
		TR_BIN= TR_NAME= TR_ARGS=() TR_PLAN=() TR_ENV=()
		oracle_translate "$kind" "$@"
		bin=$TR_BIN name=$TR_NAME
		args=("${TR_ARGS[@]}") plan=("${TR_PLAN[@]}") extra=("${TR_ENV[@]}")
	fi
	declare -F oracle_pre_run >/dev/null && oracle_pre_run
	envv+=("${extra[@]}")
	local code
	if [ -z "$RUN_QUIET" ] && [ -n "$TR_SKIP" ]; then
		STEP=$((STEP + 1))
		mkdir -p "$OUT/$SCEN/$(printf '%02d' "$STEP")_$label"
		echo "$TR_SKIP" >"$OUT/$SCEN/$(printf '%02d' "$STEP")_$label/SKIPPED"
		echo "$orig" | norm >"$OUT/$SCEN/$(printf '%02d' "$STEP")_$label/orig_cmd"
		ENV_EXTRA=() RUN_CWD=
		return 0
	fi
	[ -n "${ORACLE_RECORD_PRE:-}" ] && python3 "$TOOLS" manifest "$P" --tmp "$FX" | norm >"$WORK/pre.manifest"
	# setsid: no controlling terminal even when this script is started from one;
	# timeout: a hanging scenario is recorded as exit 124 instead of blocking the run
	if [ ${#plan[@]} -gt 0 ] && [ -z "$RUN_QUIET" ]; then
		(cd "$cwd" && timeout --kill-after=2 30 setsid -w env -i "${envv[@]}" "$bin" "${plan[@]}" \
			</dev/null >"$WORK/plan.raw" 2>"$WORK/plan.err")
		echo $? >"$WORK/plan.code"
	else
		: >"$WORK/plan.raw"
		rm -f "$WORK/plan.code"
	fi
	(cd "$cwd" && timeout --kill-after=2 30 setsid -w env -i "${envv[@]}" "$bin" "${args[@]}" \
		</dev/null >"$WORK/out.raw" 2>"$WORK/err.raw")
	code=$?
	ENV_EXTRA=() RUN_CWD=
	[ -n "$RUN_QUIET" ] && return 0

	STEP=$((STEP + 1))
	local d
	d=$OUT/$SCEN/$(printf '%02d' "$STEP")_$label
	mkdir -p "$d"
	{
		printf '%s' "$name"
		printf ' %q' "${args[@]}"
		printf '\n'
		for e in "${extra[@]}"; do printf 'env: %s\n' "$e"; done
		printf 'cwd: %s\n' "$cwd"
		declare -F oracle_translate >/dev/null && printf 'orig: %s\n' "$orig"
	} | norm >"$d/cmd"
	if [ -e "$WORK/plan.code" ]; then
		norm <"$WORK/plan.raw" >"$d/plan.json"
		cp "$WORK/plan.code" "$d/plan_exit_code"
		printf ' %q' "${plan[@]}" | norm >"$d/plan_cmd"
	fi
	[ -n "${ORACLE_RECORD_PRE:-}" ] && cp "$WORK/pre.manifest" "$d/pre.manifest"
	norm --sort-blocks <"$WORK/out.raw" >"$d/stdout"
	[ "$KEEP_UNSORTED" = 1 ] && norm <"$WORK/out.raw" >"$d/stdout.unsorted"
	norm <"$WORK/err.raw" >"$d/stderr"
	echo "$code" >"$d/exit_code"
	python3 "$TOOLS" manifest "$P" --tmp "$FX" | norm >"$d/projects.manifest"
}
# same, but nothing is recorded (set-up steps such as "bring projects in line with the vault")
qrun() { RUN_QUIET=1 run "$@"; RUN_QUIET=; }
prelude_sync() { qrun prelude drv sync "${FLAGS[@]}"; }

skip() {
	mkdir -p "$OUT/$SCEN"
	echo "$1" >"$OUT/$SCEN/SKIPPED"
	finish
	exit 0
}

finish() {
	mkdir -p "$OUT/$SCEN"
	[ -n "$DESC" ] || echo "run_Scenarios: scenario $SCEN has no desc" >&2
	printf '%s\n' "$DESC" >"$OUT/$SCEN/DESCRIPTION"
	python3 "$TOOLS" manifest "$FX" --skip-top projects --skip-name .git --tmp "$FX" | norm >"$OUT/$SCEN/rest.manifest"
}

# =================================================================== scenarios: CLI surface
sc_cli_help_root() { desc "root --help text"; run help go --help; }
sc_cli_help_delete() { desc "delete --help text"; run help go delete --help; }
sc_cli_help_list() { desc "list --help text"; run help go list --help; }
sc_cli_unknown_flag() { desc "unknown flag: error + usage on stderr, exit 1"; run bogus go --bogus; }
sc_cli_unknown_subcommand() {
	desc "unknown subcommand with otherwise valid flags"
	run bogus go frobnicate "${FLAGS[@]}"
}
sc_cli_delete_too_many_args() {
	desc "delete with two positional args (MaximumNArgs(1))"
	run two_args go delete a b --root "$P"
}
sc_cli_list_rejects_dry_run() {
	desc "--dry-run is a root-only flag; list does not know it"
	run list_dry go list --dry-run "${FLAGS[@]}"
}
sc_cli_dry_run_with_delete_subcommand_flag() {
	desc "delete --dry-run is delete's own flag, not the root one; combined with a name it previews"
	run delete_dry go delete coding --dry-run --root "$P"
}

# =================================================================== scenarios: TTY-only flows via the real binary
# Sync-apply, push, list, interactive delete, menu and setup all run a Bubble Tea
# program. With no controlling TTY the binary fails; recorded so the failure mode is known.
sc_tty_menu_no_tty() { desc "root without --dry-run opens the menu TUI: fails without a TTY"; run menu go "${FLAGS[@]}"; }
sc_tty_list_no_tty() { desc "list TUI without a TTY"; run list go list "${FLAGS[@]}"; }
sc_tty_delete_interactive_no_tty() { desc "delete without a name (interactive TUI) without a TTY"; run delete go delete --root "$P"; }
sc_tty_setup_no_tty_missing_pointer() {
	desc "no flags, no env, no pointer file: first-time Setup TUI is needed, fails without a TTY"
	rm "$H/.config/skill_Manag/vault"
	run setup go --dry-run
}

# =================================================================== scenarios: config resolution
sc_cfg_pointer_and_vault_config() {
	desc "no flags/env: vault from pointer file, root from <vault>/config.yaml"
	run dry go --dry-run
}
sc_cfg_flags_only_no_pointer() {
	desc "both flags, no pointer file"
	rm "$H/.config/skill_Manag/vault"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_cfg_env_only() {
	desc "SKILL_MANAG_VAULT / SKILL_MANAG_ROOT, no flags, no pointer file"
	rm "$H/.config/skill_Manag/vault"
	ENV_EXTRA=(SKILL_MANAG_VAULT="$V" SKILL_MANAG_ROOT="$P")
	run dry go --dry-run
}
sc_cfg_env_beats_pointer_and_config() {
	desc "env beats the pointer file (decoy vault) and config.yaml root (decoy root)"
	make_decoy
	printf '%s\n' "$FX/decoy/vault" >"$H/.config/skill_Manag/vault"
	ENV_EXTRA=(SKILL_MANAG_VAULT="$V" SKILL_MANAG_ROOT="$P")
	run dry go --dry-run
}
sc_cfg_flags_beat_env() {
	desc "flags beat env (env points at the decoy vault/root)"
	make_decoy
	ENV_EXTRA=(SKILL_MANAG_VAULT="$FX/decoy/vault" SKILL_MANAG_ROOT="$FX/decoy/root")
	run dry go --dry-run "${FLAGS[@]}"
}
sc_cfg_flag_vault_env_root() {
	desc "--vault flag with SKILL_MANAG_ROOT env while config.yaml root points at the decoy root"
	make_decoy
	sed -i "s|^root:.*|root: $FX/decoy/root|" "$V/config.yaml"
	ENV_EXTRA=(SKILL_MANAG_ROOT="$P")
	run dry go --dry-run --vault "$V"
}
sc_cfg_env_vault_flag_root() {
	desc "SKILL_MANAG_VAULT env with --root flag; pointer file points at the decoy"
	make_decoy
	printf '%s\n' "$FX/decoy/vault" >"$H/.config/skill_Manag/vault"
	ENV_EXTRA=(SKILL_MANAG_VAULT="$V")
	run dry go --dry-run --root "$P"
}
sc_cfg_root_from_vault_config_only() {
	desc "--vault only; root comes from <vault>/config.yaml"
	rm "$H/.config/skill_Manag/vault"
	run dry go --dry-run --vault "$V"
}
sc_cfg_env_empty_values_ignored() {
	desc "empty SKILL_MANAG_* env vars count as unset (pointer file + config.yaml are used)"
	ENV_EXTRA=(SKILL_MANAG_VAULT= SKILL_MANAG_ROOT=)
	run dry go --dry-run
}
sc_cfg_pointer_trailing_whitespace() {
	desc "pointer file content is TrimSpace'd"
	printf '  %s  \n\n' "$V" >"$H/.config/skill_Manag/vault"
	run dry go --dry-run
}
sc_cfg_pointer_missing_no_flags() {
	desc "no pointer file, nothing else: tries the Setup TUI"
	rm "$H/.config/skill_Manag/vault"
	run dry go --dry-run
}
sc_cfg_pointer_empty_file() {
	desc "empty pointer file behaves like a missing one"
	: >"$H/.config/skill_Manag/vault"
	run dry go --dry-run
}
sc_cfg_pointer_nonexistent_dir() {
	desc "pointer to a dir that does not exist, --root given"
	printf '%s\n' "$FX/nope" >"$H/.config/skill_Manag/vault"
	run dry go --dry-run --root "$P"
}
sc_cfg_vault_flag_nonexistent() {
	desc "--vault to a dir that does not exist"
	run dry go --dry-run --vault "$FX/nope" --root "$P"
}
sc_cfg_vault_is_a_file() {
	desc "--vault points at a regular file"
	echo x >"$FX/afile"
	run dry go --dry-run --vault "$FX/afile" --root "$P"
}
sc_cfg_root_missing_everywhere() {
	desc "vault has no root in config.yaml and no --root/env: Setup TUI needed"
	printf 'mandatory:\n  - coding\n' | set_config
	run dry go --dry-run --vault "$V"
}
sc_cfg_vault_without_config_yaml() {
	desc "vault has no config.yaml (silently ignored), --root given"
	rm "$V/config.yaml"
	run dry go --dry-run --vault "$V" --root "$P"
}
sc_cfg_config_yaml_malformed() {
	desc "config.yaml is not valid YAML: read error is ignored silently, --root given"
	printf 'root: [unclosed\n  mandatory: : :\n' | set_config
	run dry go --dry-run --vault "$V" --root "$P"
}
sc_cfg_relative_paths() {
	desc "relative --vault/--root with cwd=<TMP>: printed project paths stay relative"
	RUN_CWD=$FX
	run dry go --dry-run --vault vault --root projects
}
sc_cfg_root_nonexistent() {
	desc "--root to a dir that does not exist: walk errors are skipped, no error"
	run dry go --dry-run --vault "$V" --root "$FX/nope"
}
sc_cfg_root_is_a_file() {
	desc "--root points at a regular file"
	echo x >"$FX/afile"
	run dry go --dry-run --vault "$V" --root "$FX/afile"
}
sc_cfg_root_is_symlink() {
	desc "--root is a symlink to the projects dir: without trailing slash nothing is found, with one it is"
	ln -s "$P" "$FX/projects_link"
	run no_slash go --dry-run --vault "$V" --root "$FX/projects_link"
	run with_slash go --dry-run --vault "$V" --root "$FX/projects_link/"
}
sc_cfg_vault_empty_dir() {
	desc "vault dir without any skill dir: 'No skills found in vault.'"
	mkdir "$FX/emptyvault"
	run dry go --dry-run --vault "$FX/emptyvault" --root "$P"
}
sc_cfg_vault_skill_is_symlink() {
	desc "a vault skill that is a symlink to a dir is not a master skill (DirEntry.IsDir is false)"
	mv "$V/tmux" "$FX/tmux_real" && ln -s "$FX/tmux_real" "$V/tmux"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_cfg_list_root_missing() {
	desc "list with no root anywhere: non-TUI error before the TUI starts"
	printf 'mandatory:\n  - coding\n' | set_config
	run list go list
}
sc_cfg_delete_root_missing() {
	desc "delete <name> with no root anywhere: non-TUI error"
	printf 'mandatory:\n  - coding\n' | set_config
	run delete go delete coding
}

# =================================================================== scenarios: scanning (dry-run through the real binary)
sc_scan_baseline_dry_run() { desc "dry-run on the fixture, explicit flags"; run dry go --dry-run "${FLAGS[@]}"; }
sc_scan_exclude_dirs() {
	desc "config.yaml exclude_dirs: [team] hides project-a and project-b"
	printf 'root: %s\nexclude_dirs:\n  - team\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_paths_relative() {
	desc "exclude_paths with a path relative to the scan root"
	printf 'root: %s\nexclude_paths:\n  - org/project-c\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_paths_absolute() {
	desc "exclude_paths with an absolute path"
	printf 'root: %s\nexclude_paths:\n  - %s\n' "$P" "$P/standalone" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_paths_unclean() {
	desc "exclude_paths with trailing slash and ./ prefix (filepath.Clean)"
	printf 'root: %s\nexclude_paths:\n  - org/team/\n  - ./standalone\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_dirs_and_paths_combined() {
	desc "exclude_dirs and exclude_paths together"
	printf 'root: %s\nexclude_dirs:\n  - project-a\nexclude_paths:\n  - standalone\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_dirs_from_env() {
	desc "SKILL_MANAG_EXCLUDE_DIRS env (whitespace separated) feeds exclude_dirs"
	ENV_EXTRA=(SKILL_MANAG_EXCLUDE_DIRS="team standalone")
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_dirs_name_skills() {
	desc "exclude_dirs: [skills] skips every .agents/skills dir, so nothing is found"
	printf 'root: %s\nexclude_dirs:\n  - skills\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_dirs_name_of_root() {
	desc "exclude_dirs matching the scan root's own name skips the whole walk"
	printf 'root: %s\nexclude_dirs:\n  - projects\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_paths_root_itself() {
	desc "exclude_paths: ['.'] excludes the scan root itself"
	printf 'root: %s\nexclude_paths:\n  - .\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_paths_outside_root() {
	desc "exclude_paths pointing outside the scan root has no effect"
	printf 'root: %s\nexclude_paths:\n  - ../elsewhere\n  - /nonexistent/abs\n' "$P" | set_config
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_builtin_skipdirs() {
	desc "decoy .agents/skills/coding below built-in skip dirs are ignored; a hidden non-listed dir is scanned"
	local n
	for n in node_modules vendor dist build out target .next .nuxt .venv __pycache__ .tox .pytest_cache .cache .turbo .parcel-cache .git; do
		mkdir -p "$PA/$n/.agents/skills/coding"
		echo x >"$PA/$n/.agents/skills/coding/SKILL.md"
	done
	mkdir -p "$P/.hidden_proj/.agents/skills/coding"
	echo x >"$P/.hidden_proj/.agents/skills/coding/SKILL.md"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_nested_and_misplaced_skills() {
	desc "skills dirs not under .agents, and .agents/skills nested inside a skill, are not targets"
	mkdir -p "$PA/src/skills/coding" "$(sk "$PA" coding)/.agents/skills/doc-start" "$P/org/agents/skills/coding"
	echo x >"$PA/src/skills/coding/SKILL.md"
	echo x >"$(sk "$PA" coding)/.agents/skills/doc-start/SKILL.md"
	echo x >"$P/org/agents/skills/coding/SKILL.md"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_symlinks_in_projects() {
	desc "symlinked .agents dir and symlinked skill dir are not followed / not matched"
	mkdir -p "$FX/elsewhere/.agents/skills/coding" "$FX/elsewhere2/coding" "$P/linked_agents" "$P/linked_skill/.agents/skills"
	echo x >"$FX/elsewhere/.agents/skills/coding/SKILL.md"
	echo x >"$FX/elsewhere2/coding/SKILL.md"
	ln -s "$FX/elsewhere/.agents" "$P/linked_agents/.agents"
	ln -s "$FX/elsewhere2/coding" "$P/linked_skill/.agents/skills/coding"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_skill_name_is_a_file() {
	desc "a regular file named like a skill inside .agents/skills is not a target"
	mkdir -p "$P/filecase/.agents/skills"
	echo x >"$P/filecase/.agents/skills/coding"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_no_matching_skills() {
	desc "scan root without any installed skill: 'No matching skills found in any project.'"
	run dry go --dry-run --vault "$V" --root "$P/no-skills"
}
sc_scan_unreadable_skills_dir() {
	skip_if_root
	desc "chmod 000 on project-b's .agents/skills: walk/ReadDir errors are skipped silently"
	chmod 000 "$PB/.agents/skills"
	run dry go --dry-run "${FLAGS[@]}"
	chmod 755 "$PB/.agents/skills"
}
sc_scan_dry_run_untracked_in_vault() {
	desc "git vault: untracked and gitignored files in a vault skill do not count in '(N files)'"
	echo x >"$V/coding/untracked.md"
	echo x >"$V/coding/ignored.log"
	echo '*.log' >"$V/coding/.gitignore" && vgit add coding/.gitignore && vgit commit -qm ig
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_dry_run_vault_not_git() {
	desc "vault is not a git repo: fallback walk counts untracked files, skips symlinks and skipDirs"
	rm -rf "$V/.git"
	echo x >"$V/coding/untracked.md"
	ln -s SKILL.md "$V/coding/link.md"
	mkdir -p "$V/coding/build" "$V/coding/node_modules/x" "$V/coding/keep"
	echo x >"$V/coding/build/a.md"
	echo x >"$V/coding/node_modules/x/b.md"
	echo x >"$V/coding/keep/c.md"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_dry_run_symlinks_in_git_vault() {
	desc "git vault: tracked symlinks count as files in '(N files)'"
	ln -s SKILL.md "$V/coding/link.md"
	ln -s nowhere "$V/coding/dangling.md"
	vault_commit symlinks
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_vault_skill_with_zero_tracked_files() {
	desc "a vault skill whose files are all untracked counts '(0 files)'"
	mkdir -p "$V/newskill" "$(sk "$PA" newskill)"
	echo x >"$V/newskill/SKILL.md"
	echo old >"$(sk "$PA" newskill)/SKILL.md"
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_vault_hidden_dir_is_a_skill() {
	desc "any vault subdir, including dot-dirs, is a master skill; a project's .agents/skills/.hidden matches"
	mkdir -p "$V/.hidden" "$(sk "$PA" .hidden)"
	echo x >"$V/.hidden/SKILL.md"
	echo old >"$(sk "$PA" .hidden)/SKILL.md"
	vault_commit hidden
	run dry go --dry-run "${FLAGS[@]}"
}
sc_scan_relative_root_dot() {
	desc "--root . with cwd inside the projects dir"
	RUN_CWD=$P
	run dry go --dry-run --vault "$V" --root .
}

# =================================================================== scenarios: delete (real binary, non-interactive)
sc_del_one_skill_all_projects() {
	desc "delete coding: removed from every project that has it"
	run del go delete coding --root "$P"
}
sc_del_one_skill_one_project() {
	desc "delete coding --project project-a"
	run del go delete coding --project "$PA" --root "$P"
}
sc_del_dry_run_all() { desc "delete coding --dry-run"; run del go delete coding --dry-run --root "$P"; }
sc_del_dry_run_one_project() {
	desc "delete coding --project --dry-run"
	run del go delete coding --project "$PA" --dry-run --root "$P"
}
sc_del_all_skills_of_project() {
	desc "delete every skill of project-b one by one with --project; then dry-run sync and a missing-skill delete"
	local s
	for s in $(ls "$PB/.agents/skills" | LC_ALL=C sort); do
		run "del_$s" go delete "$s" --project "$PB" --root "$P"
	done
	run dry_after go --dry-run "${FLAGS[@]}"
}
sc_del_all_skills_of_project_via_names() {
	desc "delete by name (all projects) for every skill project-b has: other projects lose them too"
	local s
	for s in $(ls "$PB/.agents/skills" | LC_ALL=C sort); do
		run "del_$s" go delete "$s" --root "$P"
	done
}
sc_del_not_found() { desc "delete a skill no project has"; run del go delete no-such-skill --root "$P"; }
sc_del_with_exclusions() {
	desc "delete coding with exclude_dirs: [team]: only projects outside team are touched"
	printf 'root: %s\nexclude_dirs:\n  - team\n' "$P" | set_config
	run del go delete coding --root "$P"
}
sc_del_project_nonexistent() {
	desc "delete --project <missing dir>: reports 'deleted from', exit 0, creates nothing"
	run del go delete coding --project "$FX/nope" --root "$P"
}
sc_del_project_lacks_skill() {
	desc "delete refac-cli --project project-a (project has no such skill): still reports deleted"
	run del go delete refac-cli --project "$PA" --root "$P"
}
sc_del_project_flag_ignores_exclusions_and_root() {
	desc "--project works without any root and ignores exclude_dirs"
	printf 'exclude_dirs:\n  - team\n' | set_config
	rm -f "$H/.config/skill_Manag/vault"
	run del go delete coding --project "$PA" --vault "$V"
}
sc_del_empty_name_with_project() {
	desc "delete '' --project: Join drops the empty name, RemoveAll wipes the whole .agents/skills dir"
	run del go delete "" --project "$PA" --root "$P"
}
sc_del_empty_name_without_project() {
	desc "delete '' (no --project): finds nothing"
	run del go delete "" --root "$P"
}
sc_del_dot_name_with_project() {
	desc "delete . --project: RemoveAll on a path ending in '.'"
	run del go delete . --project "$PA" --root "$P"
}
sc_del_dotdot_name_with_project() {
	desc "delete .. --project: Join cleans to <project>/.agents, which RemoveAll deletes"
	run del go delete .. --project "$PA" --root "$P"
}
sc_del_traversal_name_with_project() {
	desc "delete ../../src --project: path traversal removes <project>/src (no name validation)"
	run del go delete ../../src --project "$PA" --root "$P"
}
sc_del_subpath_name_with_project() {
	desc "delete default-tools/Hk --project: removes only that subfolder of the skill"
	run del go delete default-tools/Hk --project "$PA" --root "$P"
}
sc_del_readonly_skill_by_name() {
	skip_if_root
	desc "read-only skill dir, delete by name: error line on stderr but exit 0 and 'N project(s) updated.'"
	chmod 555 "$(sk "$PA" coding)"
	run del go delete coding --root "$P"
	chmod 755 "$(sk "$PA" coding)"
}
sc_del_readonly_skill_with_project() {
	skip_if_root
	desc "read-only skill dir, delete --project: error returned, exit 1"
	chmod 555 "$(sk "$PA" coding)"
	run del go delete coding --project "$PA" --root "$P"
	chmod 755 "$(sk "$PA" coding)"
}
sc_del_skill_dir_is_symlink() {
	desc "a symlinked skill dir is not found by name; with --project the link itself is removed, target kept"
	mkdir -p "$FX/real_coding"
	echo x >"$FX/real_coding/SKILL.md"
	rm -rf "$(sk "$PD" coding)" && ln -s "$FX/real_coding" "$(sk "$PD" coding)"
	run by_name go delete coding --root "$P"
	run with_project go delete coding --project "$PD" --root "$P"
}

# =================================================================== scenarios: sync apply (driver, mirrors the TUI)
sc_drv_sync_dry_run() {
	desc "driver sync --dry-run: shows the stale files each target would lose; tree unchanged"
	run sync drv sync --dry-run "${FLAGS[@]}"
}
sc_drv_sync_baseline() {
	desc "first sync of the stale fixture: projects become a mirror of the vault for matching skills"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_noop() {
	desc "second sync right after a sync: tree identical; also records whether file mtimes get rewritten"
	prelude_sync
	find "$P" -type f -exec touch -d '2001-01-01 00:00:00 UTC' {} +
	run sync drv sync "${FLAGS[@]}"
	local changed=0 f
	for f in "$(sk "$PA" coding)/SKILL.md" "$(sk "$PD" astro)/SKILL.md"; do
		[ "$(stat -c %Y "$f")" != 978307200 ] && changed=$((changed + 1))
	done
	printf 'files_with_rewritten_mtime=%s (of 2 sampled; Go removes and rewrites every file on every sync)\n' "$changed" \
		>"$OUT/$SCEN/observations.txt"
}
sc_drv_sync_vault_file_changed_committed() {
	desc "vault file edited and committed: project copies get the new content"
	prelude_sync
	echo "appended line" >>"$V/coding/SKILL.md" && vault_commit edit
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_changed_uncommitted() {
	desc "vault file edited but NOT committed: git lists the path, disk content is copied"
	prelude_sync
	echo "uncommitted line" >>"$V/coding/SKILL.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_added_committed() {
	desc "new tracked file in a vault skill appears in every project copy"
	prelude_sync
	echo new >"$V/coding/languages/python.md" && vault_commit add
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_added_staged_only() {
	desc "new file that is only git-added (not committed) is synced (ls-files --cached)"
	prelude_sync
	echo new >"$V/coding/languages/python.md" && vgit add coding/languages/python.md
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_added_untracked() {
	desc "new untracked file in a vault skill is NOT synced"
	prelude_sync
	echo new >"$V/coding/languages/python.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_removed_git_rm() {
	desc "file git-rm'd and committed in the vault: mirror deletion in the projects"
	prelude_sync
	vgit rm -q coding/languages/markdown.md && vgit commit -qm rm
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_removed_git_rm_staged() {
	desc "file git-rm'd (staged, not committed): gone from ls-files --cached, so removed from projects"
	prelude_sync
	vgit rm -q coding/languages/markdown.md
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_deleted_from_disk_only() {
	desc "tracked file rm'd from disk but still in the index: copy fails AFTER the project copy was wiped"
	prelude_sync
	rm "$V/coding/languages/markdown.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_dir_removed() {
	desc "whole subdir git-rm'd and committed in the vault"
	prelude_sync
	vgit rm -rq coding/languages && vgit commit -qm rmdir
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_file_renamed() {
	desc "git mv + commit: old name disappears, new name appears"
	prelude_sync
	vgit mv coding/languages/markdown.md coding/languages/md.md && vgit commit -qm mv
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_skill_removed() {
	desc "a whole skill removed from the vault: projects keep their stale copy (no longer a master skill)"
	prelude_sync
	vgit rm -rq coding && vgit commit -qm rmskill
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_skill_zero_tracked_files() {
	desc "vault skill with only untracked files: matching project copies are wiped and not recreated"
	mkdir -p "$V/newskill" "$(sk "$PA" newskill)"
	echo x >"$V/newskill/SKILL.md"
	echo old >"$(sk "$PA" newskill)/SKILL.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_local_edits() {
	desc "local project edits: edited file reset, extra file/dir removed, deleted file restored"
	prelude_sync
	local s
	s=$(sk "$PA" coding)
	echo "local edit" >>"$s/SKILL.md"
	echo extra >"$s/EXTRA.md"
	mkdir -p "$s/emptydir" "$s/newdir" && echo n >"$s/newdir/n.md"
	rm "$s/languages/js_Ts.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_untracked_in_vault_and_local_copy() {
	desc "untracked vault file whose twin exists in a project copy: twin is removed as stale"
	prelude_sync
	echo vaultonly >"$V/coding/draft.md"
	echo localtwin >"$(sk "$PA" coding)/draft.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_symlink_to_file() {
	desc "tracked symlink -> file in the vault is copied as a regular file with the target's content"
	prelude_sync
	ln -s SKILL.md "$V/coding/link.md" && vault_commit link
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_symlink_to_dir() {
	desc "tracked symlink -> dir: copy fails (is a directory) after the project copy was wiped"
	prelude_sync
	ln -s languages "$V/coding/linkdir" && vault_commit linkdir
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_symlink_dangling() {
	desc "tracked dangling symlink: copy fails after the project copy was wiped"
	prelude_sync
	ln -s nowhere "$V/coding/dangling.md" && vault_commit dangling
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_symlink_outside_vault() {
	desc "tracked symlink to a file outside the vault: its content is copied into projects"
	prelude_sync
	echo secret >"$FX/outside.md"
	ln -s "$FX/outside.md" "$V/coding/ext.md" && vault_commit ext
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_exec_bit() {
	desc "disk modes of vault files are copied (755/600/664/444); dest dirs are always 755; twice for idempotence"
	prelude_sync
	echo '#!/bin/sh' >"$V/coding/run.sh"
	echo p >"$V/coding/private.md"
	echo g >"$V/coding/group.md"
	echo r >"$V/coding/readonly.md"
	vault_commit modes
	chmod 755 "$V/coding/run.sh"
	chmod 600 "$V/coding/private.md"
	chmod 664 "$V/coding/group.md"
	chmod 444 "$V/coding/readonly.md"
	chmod 700 "$V/coding/languages"
	run first drv sync "${FLAGS[@]}"
	run second drv sync "${FLAGS[@]}"
}
sc_drv_sync_exec_bit_index_only() {
	desc "exec bit only in the git index (disk mode 644): projects get 644"
	prelude_sync
	echo '#!/bin/sh' >"$V/coding/run.sh"
	vault_commit run
	vgit update-index --chmod=+x coding/run.sh
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_vault_not_git() {
	desc "vault is not a git repo: untracked files are synced, symlinks and skipDirs (build, node_modules) are not"
	rm -rf "$V/.git"
	echo x >"$V/coding/untracked.md"
	ln -s SKILL.md "$V/coding/link.md"
	mkdir -p "$V/coding/build" "$V/coding/node_modules/x" "$V/coding/keep" "$V/coding/.hiddendir"
	echo x >"$V/coding/build/a.md"
	echo x >"$V/coding/node_modules/x/b.md"
	echo x >"$V/coding/keep/c.md"
	echo x >"$V/coding/.hiddendir/d.md"
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_skilldir_named_build_in_git_vault() {
	desc "git vault: a tracked subdir named build/ IS synced (contrast with the not-a-git-repo scenario)"
	mkdir -p "$V/coding/build" && echo x >"$V/coding/build/a.md"
	vault_commit build
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_filename_space() {
	desc "tracked file with a space in its name"
	prelude_sync
	echo x >"$V/coding/with space.md" && vault_commit space
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_filename_unicode_default_git() {
	desc "tracked non-ASCII filename with default core.quotepath=true: git prints an escaped quoted name, copy fails"
	prelude_sync
	echo x >"$V/coding/ünï.md" && vault_commit unicode
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_filename_unicode_quotepath_off() {
	desc "same file with core.quotepath=false (GIT_CONFIG_COUNT env): copy works"
	prelude_sync
	echo x >"$V/coding/ünï.md" && vault_commit unicode
	ENV_EXTRA=(GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.quotepath GIT_CONFIG_VALUE_0=false)
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_filename_double_quote() {
	desc "tracked file whose name contains a double quote: git quotes/escapes it, copy fails"
	prelude_sync
	echo x >"$V/coding/a\"b.md" && vault_commit quote
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_readonly_project_dir() {
	skip_if_root
	desc "project copy dir is read-only (555): RemoveAll fails midway for that target; other targets still sync"
	prelude_sync
	echo more >>"$V/coding/SKILL.md" && vault_commit more
	chmod 555 "$(sk "$PA" coding)"
	run sync drv sync "${FLAGS[@]}"
	chmod 755 "$(sk "$PA" coding)"
}
sc_drv_sync_exclude_dirs() {
	desc "sync apply honours config.yaml exclude_dirs: [team]"
	printf 'root: %s\nexclude_dirs:\n  - team\n' "$P" | set_config
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_exclude_paths() {
	desc "sync apply honours config.yaml exclude_paths"
	printf 'root: %s\nexclude_paths:\n  - org\n' "$P" | set_config
	run sync drv sync "${FLAGS[@]}"
}
sc_drv_sync_does_not_create_new_installs() {
	desc "a vault skill missing in a project is never installed by sync (opt-in by presence)"
	run sync drv sync "${FLAGS[@]}"
	[ ! -e "$(sk "$PA" astro)" ] && echo "astro not installed in project-a: confirmed" >"$OUT/$SCEN/observations.txt"
}

# =================================================================== scenarios: push (driver, mirrors the TUI)
push_extras() {
	mkdir -p "$P/extra/empty-skills/.agents/skills" "$P/extra/agents-only/.agents" "$P/extra/no-agents/src"
	mkdir -p "$P/extra/empty-skills/.git" "$P/extra/agents-only/.git"
}
sc_drv_push_mandatory() {
	desc "push mandatory [doc-start, coding]: installs/overwrites in every project with .agents/skills, incl. an empty one"
	push_extras
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_mandatory_missing_in_vault() {
	desc "mandatory lists a skill that is not in the vault: it is ignored"
	push_extras
	printf 'root: %s\nmandatory:\n  - ghost\n  - coding\n' "$P" | set_config
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_mandatory_empty() {
	desc "mandatory: [] -> nothing to push"
	printf 'root: %s\nmandatory: []\n' "$P" | set_config
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_mandatory_absent() {
	desc "no mandatory key in config.yaml -> nothing to push"
	printf 'root: %s\n' "$P" | set_config
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_mandatory_none_in_vault() {
	desc "mandatory names only skills the vault lacks -> nothing to push"
	printf 'root: %s\nmandatory:\n  - ghost\n' "$P" | set_config
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_after_delete_all() {
	desc "delete all skills of project-b, then push: mandatory skills come back into the emptied .agents/skills"
	local s
	for s in $(ls "$PB/.agents/skills" | LC_ALL=C sort); do
		qrun del go delete "$s" --project "$PB" --root "$P"
	done
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_exclude_paths() {
	desc "push honours exclude_paths"
	printf 'root: %s\nmandatory:\n  - coding\nexclude_paths:\n  - standalone\n' "$P" | set_config
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_vault_not_git() {
	desc "push from a vault that is not a git repo (walk fallback)"
	rm -rf "$V/.git"
	echo x >"$V/coding/untracked.md"
	run push drv push "${FLAGS[@]}"
}
sc_drv_push_does_not_write_vault_config() {
	desc "push never rewrites config.yaml (only the TUI edit overlay does); rest.manifest shows the vault unchanged"
	run push drv push "${FLAGS[@]}"
}

# =================================================================== scenarios added for the second parity round
sc_cfg_vault_git_zero_skills() {
	desc "git vault whose commit holds no skill dirs (only config.yaml): Go still sees .git as a 'skill', so no 'No skills found' message"
	local d
	for d in "$V"/*/; do vgit rm -rq "$(basename "$d")"; done
	vgit commit -qm empty
	run dry go --dry-run "${FLAGS[@]}"
	run sync drv sync "${FLAGS[@]}"
}
sc_cfg_config_root_empty() {
	desc "root: \"\" or a bare root: in config.yaml counts as unset (Setup TUI needed); an explicit --root still wins"
	printf 'root: ""\n' | set_config
	run empty_string go --dry-run
	printf 'root:\n' | set_config
	run null go --dry-run
	printf 'root: ""\n' | set_config
	run flag_wins go --dry-run --root "$P"
}
sc_scan_exclude_paths_nonexistent() {
	desc "exclude_paths entries that do not exist (relative, then absolute): Go ignores them"
	printf 'root: %s\nexclude_paths:\n  - no/such/dir\n' "$P" | set_config
	run relative go --dry-run "${FLAGS[@]}"
	printf 'root: %s\nexclude_paths:\n  - %s\n' "$P" "$FX/nope" | set_config
	run absolute go --dry-run "${FLAGS[@]}"
}
sc_scan_exclude_empty_entries() {
	desc "empty entries in exclude_paths / exclude_dirs: Go ignores them (an empty exclude_paths entry is skipped, an empty dir name never matches)"
	printf 'root: %s\nexclude_paths:\n  - ""\n' "$P" | set_config
	run paths go --dry-run "${FLAGS[@]}"
	printf 'root: %s\nexclude_dirs:\n  - ""\n' "$P" | set_config
	run dirs go --dry-run "${FLAGS[@]}"
}
sc_cfg_root_is_symlink_delete() {
	desc "delete coding through a scan root that is a symlink: without trailing slash Go finds nothing, with one it deletes"
	ln -s "$P" "$FX/projects_link"
	run no_slash go delete coding --root "$FX/projects_link"
	run with_slash go delete coding --root "$FX/projects_link/"
}
sc_del_project_symlinked_skill_dangling() {
	desc "delete --project where the skill folder is a dangling symlink: Go removes the link"
	rm -rf "$(sk "$PD" coding)" && ln -s "$FX/nowhere" "$(sk "$PD" coding)"
	run del go delete coding --project "$PD" --root "$P"
}
sc_del_project_through_symlinked_agents() {
	desc "delete --project where .agents is a symlink to a dir elsewhere: Go's RemoveAll follows it and deletes inside the target"
	mv "$PD/.agents" "$FX/real_agents" && ln -s "$FX/real_agents" "$PD/.agents"
	run del go delete coding --project "$PD" --root "$P"
}
sc_del_project_through_symlinked_skills_dir() {
	desc "delete --project where .agents/skills is a symlink to a dir elsewhere: Go deletes inside the target"
	mv "$PD/.agents/skills" "$FX/real_skills" && ln -s "$FX/real_skills" "$PD/.agents/skills"
	run del go delete coding --project "$PD" --root "$P"
}

# =================================================================== main
oracle_main() {
	LIST_ONLY=0
	SELECTED=()
	while [ $# -gt 0 ]; do
		case $1 in
		-o) OUT=$(mkdir -p "$2" && cd "$2" && pwd) && shift 2 ;;
		-l) LIST_ONLY=1; shift ;;
		-h | --help) sed -n '2,28p' "$0"; exit 0 ;;
		-*) die "unknown option $1" ;;
		*) SELECTED+=("$1"); shift ;;
		esac
	done

	ALL=($(declare -F | awk '$3 ~ /^sc_/ {sub(/^sc_/, "", $3); print $3}' | LC_ALL=C sort))
	if [ "$LIST_ONLY" = 1 ]; then printf '%s\n' "${ALL[@]}"; exit 0; fi
	[ ${#SELECTED[@]} -gt 0 ] || SELECTED=("${ALL[@]}")

	command -v setsid >/dev/null || die "setsid (util-linux) is required"
	command -v python3 >/dev/null || die "python3 is required"
	command -v git >/dev/null || die "git is required"
	[ -x "$GO_BIN" ] || die "missing $GO_BIN (run build_Oracle.sh)"
	[ -x "$DRV_BIN" ] || die "missing $DRV_BIN (run build_Oracle.sh)"
	[ -d "$FIXTURES/vault" ] || die "missing fixtures $FIXTURES (run build_Oracle.sh: it creates the worktree)"

	WORK=$(mktemp -d "${TMPDIR:-/tmp}/oracle.XXXXXX")
	case $WORK in /tmp/* | "${TMPDIR:-/tmp}"/*) ;; *) die "unexpected temp dir $WORK" ;; esac
	# the vault must not sit inside another git repo, or `git ls-files` would see that repo
	if git -C "$WORK" rev-parse --git-dir >/dev/null 2>&1; then die "temp dir $WORK is inside a git repo; set TMPDIR elsewhere"; fi
	cleanup() { chmod -R u+rwX "$WORK" 2>/dev/null; rm -rf -- "$WORK"; }
	trap cleanup EXIT

	# never delete a directory that is not an oracle output tree (ORACLE_OUT typo guard)
	if [ -n "$(ls -A "$OUT" 2>/dev/null)" ] && [ ! -f "$OUT/INDEX.tsv" ] && ! compgen -G "$OUT/*/DESCRIPTION" >/dev/null; then
		die "refusing to replace $OUT: not empty and not an oracle output tree (no INDEX.tsv)"
	fi
	rm -rf "$OUT"
	mkdir -p "$OUT"
	for name in "${SELECTED[@]}"; do
		declare -F "sc_$name" >/dev/null || die "no such scenario: $name"
		(
			SCEN=$name STEP=0
			fresh_fixture
			"sc_$name"
			finish
		)
	done

	# index: one line per scenario with its steps and exit codes
	{
		for name in "${SELECTED[@]}"; do
			d=$OUT/$name
			if [ -f "$d/SKIPPED" ]; then
				printf '%s\tSKIPPED\t%s\n' "$name" "$(cat "$d/SKIPPED")"
				continue
			fi
			codes=
			for s in "$d"/[0-9][0-9]_*; do
				[ -d "$s" ] || continue
				codes="$codes $(basename "$s" | cut -d_ -f2-)=$(cat "$s/exit_code")"
			done
			printf '%s\t%s\t%s\n' "$name" "$(cat "$d/DESCRIPTION")" "${codes# }"
		done
	} >"$OUT/INDEX.tsv"
	echo "run_Scenarios: ${#SELECTED[@]} scenarios -> $OUT"
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then oracle_main "$@"; fi
