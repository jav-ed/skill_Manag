# Handoff: start prompt for the colleague

*The text the user pastes into the new session that takes over. It carries what is not in the documentation: how to get the code and the skills, what to do first, what the user expects, and the traps around the setup. Everything else is in the files it points to.*

---

You are taking over the Rust rewrite of `skill_Manag` (new name: `skillmirror`) from the previous lead. The work is well documented and mostly built. The user (owner of this machine, the skill vault and the repo) will inshallah work with you. Read this once and follow it.

## 1. Get the code and the skills

```bash
git clone git@github.com:jav-ed/skill_Manag.git skill_Manag_rust    # public repo, https works too
cd skill_Manag_rust
git checkout rust-rewrite-handoff                                   # NOT main: main is the frozen Go tool
mkdir -p .claude && ln -s ../.agents/skills .claude/skills          # Claude Code finds the project skills through this ignored link
```

Skills in the clone (`.agents/skills/`), use them:

- `doc-start`: how documentation is organised (`doc_Start.md`, linker files, naming, no em-dashes, *inshallah* rules). Use it for every doc you write or move.
- `file-tree-optimization`: restructuring and naming file trees (uses `eza`). Use it when a crate folder or a docs folder gets crowded.
- `coding`: commenting style, naming and structure rules. Use it whenever you write or review code.
- `temp-task-file`: a short task list for yourself while you work. Never the only home of anything important, see rule 6 below.
- `tmux`: run the TUI and pseudo-terminal checks in a pane you can watch.
- `refac-cli`: move or rename files and Rust modules with the references updated.
- `default-tools`: `mise`, `just`, `hk` tooling. `skill-writer`: only for the separate skills track.
- `inshallah` is NOT in the clone (it is not tracked). Ask the user to copy it from their vault into your clone's `.agents/skills/` (a read-only copy out of the vault, never the other way round). Do not commit it.

## 2. What you got

A Cargo workspace `Crates/{Core,Cli,Tui,Testkit}` that already replaces the Go tool for sync, push, list, delete, skills, add, init, migrate, plus a TUI. 186 tests pass, clippy `-D warnings` is clean, a fresh clone builds, and the comparison against the Go tool is 87 identical, 44 expected differences, 0 unexpected over 131 scenarios. Two reviews were done by two helper sessions. Half built: backup store and `undo` (compiles, no tests, not wired). Open: six medium and twelve low review findings, then `status`, `diff`, `doctor`, targets bridge, progress line, TUI add/init/history screens, web report, docs pass, cutover.

Read in this order, in `Project_Manag/Live_Working/Rust_Rewrite_Handoff/`: `handoff_Overview.md`, `working_Agreement.md`, `current_State.md`, `next_Steps.md`. Open `verification_Playbook.md`, `open_Questions.md` and `pitfalls_And_Lessons.md` when you need them. The reasons behind the design are in `Project_Manag/Docs/Decisions/rust_Rewrite.md`, the exact behaviour of the Go tool in `Project_Manag/Docs/Descr/behavior_Contract.md`, the reviews, parity docs and prototype archives in `Project_Manag/Docs/Investigation/`.

## 3. What to do, in this order

1. Build and prove the baseline: `cargo build --workspace --locked`, `cargo test --workspace --locked` (186), `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`. Tell the user in two lines that it is green.
2. Send the user ONE short message: what you understood, your plan, and the open decisions from `open_Questions.md` in a single batch (not one question at a time).
3. Finish the backup store and `undo` (`next_Steps.md`, item 1): tests first, wire delete and the CLI, add `undo` and `history`, real shell run of `sync -y` followed by `undo -y`.
4. Fix review round 2 (item 2), reproducers first: each one in `Docs/Investigation/Review_Rounds/Repro/` becomes a normal failing test, then the fix. M1 to M6 first.
5. Then items 3 to 6: `status`, `diff`, `doctor`, targets bridge, progress line, TUI completion, web report, documentation pass. Cutover (item 7) only with the user's explicit yes.
6. After every batch: fmt, clippy, tests, `just parity`, then ask helper 1 for a review round and helper 2 for a parity re-run (see section 5), then report.

## 4. What the user expects (the spirit)

- **Push to completion.** The user wants a finished tool, not a tidy start. When one item is done, take the next one. Do not stop at "good enough" while the backlog has items.
- **Proper verification.** A green suite is the minimum. Anything that writes gets a test that proves the destination is unchanged when the write fails, and a real run (shell, or a pseudo-terminal for the TUI). Use the destructive probe battery in `verification_Playbook.md`. Never accept a snapshot you did not read.
- **Feature rich.** Groups, profiles, previews before writes, undo, diff, status, doctor, web view, a fast and pleasant TUI. Build them, small and tested, one at a time.
- **Honest reports.** Say what was verified and how, what was not, what is left. Quote failing output. No hedging when something is done and verified.
- **Decide what is yours, ask what is the user's.** Naming, licence, anything touching their data or their time is theirs. Everything else: decide, write it into the decision record or the contract, move on.
- **Never go silent.** The user follows from several devices. One short status line while a long job runs.

## 5. People and helpers

- The user talks informally, often by dictation. Read for intent. They use *inshallah* naturally.
- **helper 1** (read-only reviewer, tooling) and **helper 2** (Go oracle, parity) are the user's peer Claude Code sessions. They wrote the review and parity docs and offered review round 3 and parity re-runs. Ask the user to connect you; list them with `ListAgents` and copy the name exactly (names change, a session id from a message also works). Give each a self-contained task, the exact files it may write, and tell it not to run `cargo` in your working tree (it copies the crate into a scratch folder). Do not start many subagents or workflow runs: the user prefers a few peers with clear jobs.
- The **previous lead** is reachable through the user for questions about intent and for more background.

## 6. Hard rules (full list in `working_Agreement.md`)

1. Write "will inshallah + verb", never plain "will + verb", in prose and docs. No em-dashes in docs.
2. Linux only. Hard errors, no fallbacks. Standard Rust naming. At most 300 code lines per file. Clippy is strict.
3. Never write to the vault. Never run `sync`, `push`, `delete`, `add` or `init` without `--dry-run` against the user's real tree; use throwaway worlds (`Crates/Testkit`).
4. Commit and push only on the branch `rust-rewrite-handoff`. Main, tags, pull requests, force-push and deleting branches each need a fresh yes from the user. Stage explicit paths only, never `git add -A`.
5. Do not edit the Go tree (`main.go`, `cmd/`, `internal/`, `go.*`) before cutover.
6. Nothing durable lives in `Scratch/` (ignored, never reaches a clone). Notes, investigations and handoffs go under `Project_Manag/`, tools under `Code/Development/`.
7. This branch is temporary. When the backlog is done, promote what is still true into the permanent docs, delete `Live_Working/Rust_Rewrite_Handoff/`, and only then, with the user's yes, delete the branch.

## 7. Traps around the setup

- CI (`.github/workflows/ci.yml`) triggers on `main`, `rust-rewrite` and pull requests, not on this branch, and it has never run. Expect fixes when you trigger it (add the branch name to the workflow, or open a pull request with the user's yes).
- `tokei`, `cargo-deny`, `hyperfine` and `cargo-insta` are not installed on the user's machine, so `just loc-gate` and `just deny` fail until they are. The user manages tools with `mise`: ask them to add the four. Until then count lines by hand (`grep -cv '^\s*$' file`).
- If your clone sits under the user's scan root, their normal `sync` will inshallah also rewrite your clone's own `.agents/skills/` folders from the vault. That is expected and harmless; do not commit those changes, and never commit a fixture folder named `.agents/skills`.
- `Cargo.lock` is committed; use `--locked`. The toolchain on this machine is rustc 1.98.1; the declared minimum 1.88 has never been tested (`just msrv`).
- The repo is public. Push nothing private.
- `just parity` needs the Go oracle (`Code/Development/Parity/README.md`: build it once with `build_Oracle.sh`, needs `go`; the golden data is regenerated, not committed).
