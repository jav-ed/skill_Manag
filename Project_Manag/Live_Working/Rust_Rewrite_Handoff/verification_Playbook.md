# Handoff: verification playbook

*Every way the lead proved a change works, what each layer can and cannot catch, and a battery of destructive probes to run against anything that writes. Open it before you call something done. The rule of the project: a green test suite is the minimum, not the proof.*

## The layers

| Layer | Run with | Catches | Cannot catch |
|---|---|---|---|
| Format and lint | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | style, panics in product code (`unwrap`, indexing), unreachable `pub`, pedantic smells | behaviour |
| Core unit tests | `cargo test -p skillmirror-core --locked` (107) | planning, apply, delete, config, scan, vault rules, with real directories in temp folders | terminal behaviour, front-end wiring |
| CLI integration tests | `cargo test -p skillmirror --locked` (40) | flags, exit codes, JSON, refusing to write without `--yes`, config errors, install flows | what a human sees in a terminal |
| TUI synthetic tests | `cargo test -p skillmirror-tui --locked` (39) | state machine, key and mouse routing, rendering (insta snapshots in `Crates/Tui/src/tests/snapshots/`) | real terminal bytes, resize storms, input floods |
| Real-PTY tests | `Crates/Cli/tests/interface.rs` (portable-pty + vt100) | start-up, quitting cleanly, mouse, zero idle CPU, raw-mode restore | visual taste |
| Go parity oracle | `Code/Development/Parity/` (see the parity docs) | any unplanned behaviour change versus the Go tool over 131 scenarios | new features (no Go counterpart) |
| Review rounds | `Docs/Investigation/Review_Rounds/` | races, odd filesystems, UI under odd sizes, anything the author did not think of | nothing is guaranteed; run again after fixes |
| Real data, read-only | below | surprises in the user's real tree | writes (never run them) |

The whole local gate is `just check` (fmt-check, clippy, nextest, loc-gate, check-deps); it needs `tokei` and `jq` for the line gate. Without them run the first three layers by hand and count lines (`grep -cv '^\s*$' file`, comments included, is a safe over-estimate of tokei's code count).

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
target/release/skillmirror list --json | jq length   # installed folders (436 at hand-off)
time target/release/skillmirror sync --dry-run       # compare with the Go dry run
target/release/skillmirror sync --check; echo $?     # 0 clean, 1 drift
```

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
