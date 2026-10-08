# Handoff: overview

*Entry file of the handoff for the Rust rewrite of `skill_Manag`. It says what the project is, what state the branch is in, how to get a working build in minutes, and which file of this folder answers which question. The other files are written to be opened one at a time.*

The previous lead (a Claude Code session) stopped on 2026-10-08 at the user's request, because the user wants a colleague to finish the work. Everything the lead knew that is not visible in the code is written down in this folder. The lead can still be asked questions through the user.

## What the project is

`skill_Manag` mirrors agent skill folders from one git-tracked master vault into every project's `.agents/skills/<name>/` directory. The files are copied, never symlinked, so projects stay git-tracked and work over SSH. The Go implementation on `main` is slow (a dry run took 4.8 to 7 s in the last comparison and up to 13 s with a cold cache). The branch `rust-rewrite-handoff` (see "Branches" below) holds a Rust rewrite named `skillmirror` that is faster (scan about 0.7 s), safer (staged writes, atomic directory swap, hard errors) and has more features (nested vault groups, profiles, `add`, `init`, a TUI with mouse and fuzzy filter). The user wants it finished: feature rich, verified properly, then cut over (Go removed in one commit).

## State in one table

| Area | State |
|---|---|
| Core engine (`Crates/Core`) | done and reviewed twice; 107 unit tests |
| CLI (`Crates/Cli`): `sync push list delete skills add init migrate completions tui` | done; 40 integration tests incl. 3 real-PTY tests |
| TUI (`Crates/Tui`): menu, sync, push, delete, list, setup wizard, mouse, fuzzy filter | done; 39 tests incl. snapshot tests |
| Parity against the Go tool (oracle, 131 scenarios) | 87 match, 44 expected divergences, 0 unexpected |
| Backup store and `undo` | done: store, apply and delete keep the old copy, CLI `undo` and `history`, TUI jobs keep backups; 54 new tests |
| Review round 2 findings (6 medium, 12 low, 1 info) | all fixed or decided on 2026-10-08, each with a test that was red first; a third review round is the next check |
| `status`, `diff`, `doctor`, targets bridge, registry cache, `indicatif` progress | not started |
| Web report, TUI add/init/history screens | not started |
| Docs pass (README, architecture docs), cutover | not started |

Totals at hand-off: 186 tests pass (297 after the work of 2026-10-08), `cargo clippy --workspace --all-targets -- -D warnings` is clean, `cargo fmt --check` is clean, no Rust file is over 300 lines of code (my own count; `tokei` is not installed, so `just loc-gate` has never run).

## Branches

- `rust-rewrite-handoff`: the branch to clone and continue on. It holds the Rust workspace, the docs and this handoff. The user chose a separate branch on 2026-10-08 to keep the history clean.
- `rust-rewrite`: the lead's local branch at the same code without the handoff commits. It was never pushed; ignore it.
- `main`: the Go implementation, frozen until cutover. Its local tip `c7310f9` ("skills added") was pushed together with the handoff branch because it is the base of the branch.
- CI (`.github/workflows/ci.yml`) triggers on pushes to `main` and `rust-rewrite` and on pull requests, so pushing `rust-rewrite-handoff` alone does not run it. Add the branch name to the workflow, or open a pull request, to get the first CI run (expect fixes).

## What happens to this handoff branch

The branch `rust-rewrite-handoff` is a transfer vehicle, not a permanent home. The user's plan (2026-10-08): the colleague reads everything here, implements the backlog, and afterwards deletes the branch to keep the repository clean. So that nothing is lost when it goes:

- **Lasting knowledge lives in `Project_Manag/Docs/`** and stays: the decision record, the behavior contract, the research reports, the review rounds, the parity oracle docs and the prototype archives (`Docs/Investigation/`). Nothing important is kept only in `Scratch/` (gitignored, never reaches a clone).
- **This folder (`Live_Working/Rust_Rewrite_Handoff/`) is transient.** When its backlog is done, first promote what is still true into permanent places (the verification playbook and pitfalls into `Docs/Setup/` or `Docs/Descr/`, the architecture summary into `Docs/Architecture/`, open decisions into `Docs/Decisions/`), then delete the folder and the `open_Issues.md` rows that point into it.
- **Delete the branch last, and ask the user first.** Local: `git branch -d rust-rewrite-handoff` after it is merged or its commits are on the branch that replaces it. Remote: `git push origin --delete rust-rewrite-handoff`. Both are outward actions that need the user's explicit yes.

## First hour

```bash
git clone git@github.com:jav-ed/skill_Manag.git && cd skill_Manag
git checkout rust-rewrite-handoff
cargo build --workspace --locked
cargo test --workspace --locked            # 186 tests; the first build takes a few minutes
cargo clippy --workspace --all-targets --locked -- -D warnings
target/debug/skillmirror --help
```

Skills in a fresh clone: `.agents/skills/` holds `coding`, `default-tools`, `doc-start`, `file-tree-optimization`, `refac-cli`, `remote-helper`, `skill-writer`, `temp-task-file` and `tmux`. Two things are missing because they are not in git. First, Claude Code reads skills through the ignored symlink `.claude/skills`; create it with `mkdir -p .claude && ln -s ../.agents/skills .claude/skills`. Second, the `inshallah` skill (needed by the doc-start writing rules) lives only in the user's vault; ask the user to copy `inshallah` from the vault into the clone's `.agents/skills/` (a read-only copy from the vault, never the other way round) and do not commit it.

Then open a real terminal (not a pipe) and run `target/debug/skillmirror tui --vault <a throwaway vault> --root <a throwaway folder>` to feel the TUI. A throwaway world is easy to make: `Crates/Testkit/src/lib.rs` (`World::standard`) builds one for tests, and the same layout works by hand (a git repo with `coding/SKILL.md`, a `web/astro/SKILL.md`, a root folder with projects that have `.agents/skills/coding/`).

Never point the tool at the user's real vault with a writing command while learning. The read-only commands are fine (`skills`, `list`, `sync --dry-run`, `--check`).

## Which file answers what

- [Start prompt](start_Prompt.md): the text the user pastes into the colleague's session: how to clone and set up the skills, the first tasks in order, the spirit the user expects, helpers, hard rules and traps around the setup. Open it to see what the colleague was told on day one.
- [Working agreement](working_Agreement.md): who the user is, the standing rules they gave, what "done" means to them, how to talk to them, and the spirit the lead worked in (push to completion, verify properly, feature rich). Open it before the first message to the user.
- [Current state](current_State.md): the architecture as built, the invariants that keep data safe, the command surface and exit codes, measured numbers, tool availability, known gaps. Open it before changing code.
- [Next steps](next_Steps.md): the ordered backlog with a "done when" for each item: finish backup and `undo`, the review-2 fixes, the rest of phase 5, phases 6 to 8, and the separate skills track. Open it to pick the next piece of work.
- [Verification playbook](verification_Playbook.md): every way to prove a change works: unit and integration tests, snapshot tests, PTY runs, the Go parity oracle, read-only checks on the real vault, review reproducers, what each layer cannot catch. Open it before calling anything done.
- [Open questions](open_Questions.md): decisions that belong to the user, with the lead's default for each. Open it before deciding something that changes behaviour, naming, licence or the user's files.
- [Pitfalls and lessons](pitfalls_And_Lessons.md): mistakes already made once, tool quirks, and traps in this repo. Open it when something behaves strangely.

Related material outside this folder: [the decision record](../../Docs/Decisions/rust_Rewrite.md) (why each crate and layout choice), [the behavior contract](../../Docs/Descr/behavior_Contract.md) (exact behaviour of the Go tool, quirks tagged KEEP, CHANGE, UNCLEAR, and the Q31 to Q33 rulings), [the research reports](../../Docs/Investigation/Rust_Stack/linker_Rust_Stack.md), the review rounds under `Docs/Investigation/Review_Rounds/` and the parity oracle under `Docs/Investigation/Parity_Oracle/`.

## Who is who

- **The user**: owner of the project, the vault and about 60 projects on their machine. Decides naming, licence, anything that touches their data, and when to commit or push to anything other than the agreed branch.
- **The previous lead**: wrote the engine, CLI, TUI and most tests. Gone from the code, still reachable through the user.
- **Helper sessions**: two peer Claude Code sessions the user opened, "helper 1" (read-only reviewer, tooling) and "helper 2" (Go oracle and parity). They wrote the behavior contract, the CI and `deny.toml`, both review rounds and the parity harness. Their knowledge is written out in the Investigation folder. They may or may not still exist; ask the user.
- **You**: the colleague. The user will inshallah tell you what they expect first; this folder is the background.
