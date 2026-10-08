# Handoff: verification playbook

*Every way the lead proved a change works, what each layer can and cannot catch, and a battery of destructive probes to run against anything that writes. Open it before you call something done. The rule of the project: a green test suite is the minimum, not the proof.*

## The layers

| Layer | Run with | Catches | Cannot catch |
|---|---|---|---|
| Format and lint | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | style, panics in product code (`unwrap`, indexing), unreachable `pub`, pedantic smells | behaviour |
| Core unit tests | `cargo test -p skillmirror-core --locked` | planning, apply, delete, config, scan, vault rules, with real directories in temp folders | terminal behaviour, front-end wiring |
| CLI integration tests | `cargo test -p skillmirror --locked` | flags, exit codes, JSON, refusing to write without `--yes`, config errors, install flows | what a human sees in a terminal |
| TUI synthetic tests | `cargo test -p skillmirror-tui --locked` | state machine, key and mouse routing, rendering (insta snapshots in `Crates/Tui/src/tests/snapshots/`) | real terminal bytes, resize storms, input floods |
| Real-PTY tests | `Crates/Cli/tests/interface.rs` (portable-pty + vt100) | start-up, quitting cleanly, mouse, zero idle CPU, raw-mode restore | visual taste |
| Web server tests | `cargo test -p skillmirror-web --features server --locked` (the router driven in process: `guard`, `read`, `flow`, `undo`, `files`, `idle`) | who gets in, the JSON shapes (pinned for `Ui/`), plan then apply then job, a plan used twice, undo, the built pages having no inline script | what a browser does with it |
| Browser check of the web interface | `Code/Development/Web/check_Web.sh` (Chromium through Playwright; README there) | the link and cookie, counters, a plan with its diff and apply, push, undo, the skills page, theme, refusals a page script can provoke, requests leaving the server, console errors and CSP violations | other browsers, small phones |
| Browser check of the report | `Code/Development/Report/check_Report.sh` (same method) | the report's filter, theme button, diff anchors, no outside requests, no console errors, screenshots to read | print layout, browsers other than Chromium |
| Smoke of the built binary | `Code/Development/Smoke/check_Smoke.sh [binary]` | the whole user path on the artifact (authoring, install, drift, sync, undo and redo, adopt, scoped add, the web server over HTTP with its refusals and a plan/apply/job round trip) | taste; the real tree |
| Mutation check | `Code/Development/Gate/check_Mutation.sh FILE 'sed' <cargo test args>` | a guard no test would miss: break it, a test must fail. Needs the right `--features` or it tests nothing | guards nobody mutated |
| Fresh clone | `git clone` into an empty folder, then `cargo build --locked`, `cargo test --locked`, `Code/Development/Web/build_Ui.sh --check`, `cargo install --path Crates/Cli --locked` | files that exist only in your working tree (git-ignored generated files, a stale binary, a missing lockfile entry) | everything else |
| Stress under load | run one test binary many times with busy loops in the background (see pitfalls: the Ctrl-C race failed 6 of 120 runs this way and 0 of 300 after the fix) | races and timing assumptions that a quiet machine hides | |
| Go parity oracle | `just parity` (harness in `Code/Development/Parity/`, docs in `Docs/Investigation/Parity_Oracle/`) | any unplanned behaviour change versus the Go tool over 131 scenarios | new features (no Go counterpart) |
| Review rounds | `Docs/Investigation/Review_Rounds/` | races, odd filesystems, UI under odd sizes, anything the author did not think of | nothing is guaranteed; run again after fixes |
| Real data, read-only | below | surprises in the user's real tree | writes (never run them) |

The whole local gate is `Code/Development/Gate/check_Gate.sh` (fmt and clippy on two toolchains, loc-gate, check-features, check-deps, deny, every test; ends with `GATE OK`); `just check` is the shorter form. Both need `tokei` and `jq` for the line gate. Without them run the first layers by hand and count lines (`grep -cv '^\s*$' file`, comments included, is a safe over-estimate of tokei's code count).

## Snapshot tests

Snapshots are accepted by running with `INSTA_UPDATE=always` and then reading `git diff Crates/Tui/src/tests/snapshots`. Never accept a snapshot you did not read: a snapshot test that is regenerated blindly proves nothing.

## Real terminal checks

1. PTY tests drive the real binary: see `Crates/Cli/tests/common/pty.rs` for the helper (spawn, wait for text, send keys, read the vt100 screen).
2. By hand: run the TUI in a throwaway world inside `tmux` (the `tmux` skill explains the capture-pane routine): open every screen, resize the window to something tiny (the draw fuzz in review round 2 ran 19 screens from 1 to 50 columns and 1 to 16 rows without a panic; keep it that way), paste a long path into the wizard, sweep the mouse quickly (this is what stalled crossterm), press Ctrl-C and `q` during a run.
3. Check the terminal after exit: `stty -a` shows raw mode off, the mouse no longer reports, the cursor is visible, the alternate screen is gone. A panic must restore it too (`backend.rs` installs a hook).

## Read-only checks on the real vault and tree

The vault path is in [internal repo paths](../../Docs/Setup/internal_Repo_Paths.md); the scan root is `root:` in its `config.yaml`. Use the release build for timings.

```bash
cargo build --release --locked
export SKILLMIRROR_VAULT=<vault path> SKILLMIRROR_ROOT=<scan root>
target/release/skillmirror skills                    # vault view: groups and skills
target/release/skillmirror list --json | jq length   # installed folders (436 at the earlier measurement)
time target/release/skillmirror sync --dry-run       # compare with the Go dry run
target/release/skillmirror status; echo $?           # 0 clean, 1 drift
target/release/skillmirror info <skill>              # one skill from every side
target/release/skillmirror doctor                    # also reads every skill header
target/release/skillmirror web                       # read-only without --allow-write
```

This run on the real data has not been done since the new commands were added (handoff_Overview, gap 1).

Never run `sync`, `push`, `delete`, `add` or `init` without `--dry-run` there. If a check needs a write, build a throwaway world instead (`World::standard` in `Crates/Testkit`, or the same layout by hand).

Compare with Go: build the oracle (parity docs, `oracle_Setup.md`) and diff the `--dry-run` listing of both tools on the same tree; any difference must map to a contract Q-id.

## Destructive probe battery (run for every new write path)

Build each case in a temp world and assert that the destination is byte-identical afterwards unless the case says otherwise.

1. `.agents` is a symlink; `.agents/skills` is a symlink; the skill folder is a symlink; the project itself is a symlink.
2. The destination parent is read-only; the destination folder is read-only; one file inside it is read-only.
3. A file in the destination is edited between plan and apply (same size, different bytes; same size and restored mtime).
4. A new file appears in the destination between plan and apply.
5. A skill name that is `..`, `.`, empty, contains `/`, starts with `.`, or is not valid UTF-8.
6. The vault is not a git repo; a skill file is untracked; a tracked path is a symlink; the index has unmerged entries; the vault is empty.
7. The skill has zero tracked files; deep nesting (more than 4 group levels); duplicate skill names across groups.
8. Two runs at once on the same tree (race test loops: 100 rounds, count failures).
9. `kill -9` during apply: afterwards only `.stage-*` or `.trash-*` leftovers exist, `skills/` has no half skill, and the next run reports the leftovers as warnings.
10. Full disk (a small tmpfs: `mount -t tmpfs -o size=64k` needs root, or `ulimit -f` with a big file) and an unwritable state directory (for the backup store).
11. Config: malformed YAML, unknown key, empty `root:`, `exclude_paths` pointing nowhere, CRLF line endings, BOM, comments inside lists, a symlinked `config.yaml`.
12. Non-terminal use: stdin closed, stdout piped, `NO_COLOR`, `TERM=dumb`.

## Reviews

`Docs/Investigation/Review_Rounds/review_Method.md` records how helper 1 reviewed (isolated copy of the repo, reproducers as tests, draw fuzz, race loops). After each batch of fixes ask for another round; round 2 found six medium issues that the author's own tests had missed, which is the reason the rule exists.

## Reporting

End every piece of work with: what was verified and how (command and result), what was not verified and why, what is left. Quote failing output as it is.
