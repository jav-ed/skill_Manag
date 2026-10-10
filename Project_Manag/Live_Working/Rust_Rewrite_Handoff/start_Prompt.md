# Handoff: start prompt for the colleague

*The text the user pastes into the new session that takes over. It carries what is not in the documentation: how to get the code, what is finished and what is not, what to do first, what the user expects, and the traps around the setup. Everything else is in the files it points to.*

---

You are taking over `skillmirror` (formerly `skill_Manag`), a Rust tool that mirrors agent skill folders from one git vault into every project's `.agents/skills/`. The previous lead (a Claude Code session) built it and merged it into `main` on 2026-10-08. It works and it is proved in the ways listed in `handoff_Overview.md`; it is not finished, and the same file lists exactly what is not proved. The user (owner of this machine, the skill vault, about 60 projects and the repo) works with you. Read this once and follow it.

## 1. Get the code

```bash
git clone git@github.com:jav-ed/skill_Manag.git && cd skill_Manag     # public repo, https works too; main is the Rust tool
mkdir -p .claude && ln -s ../.agents/skills .claude/skills            # Claude Code finds the project skills through this ignored link
```

Skills in the clone (`.agents/skills/`), use them: `doc-start` (how documentation is organised: `doc_Start.md`, linker files, naming, no em-dashes), `coding` (commenting, naming, structure), `file-tree-optimization`, `temp-task-file`, `tmux` (run the TUI in a pane you can watch), `refac-cli`, `default-tools` (`mise`, `just`, `hk`), `skill-writer` (only for the separate skills track). `inshallah` is NOT in the clone (not tracked): ask the user to copy it from their vault into your clone's `.agents/skills/` (a read-only copy out of the vault, never the other way round) and do not commit it.

## 2. What you got

A Cargo workspace `Crates/{Core,Cli,Tui,Web,Testkit}` plus `Ui/` (the web interface, node, outside the workspace). Commands: `sync push add init delete list skills info new adopt vault config mandatory agents status diff doctor report web bridge undo history migrate completions tui`. 780 tests; fmt and clippy `-D warnings` clean on rustc 1.97 and 1.99; Go parity 90 match / 41 expected / 0 unexpected over 131 scenarios; a binary installed from a fresh clone passes `Code/Development/Smoke/check_Smoke.sh`; the web interface passes `Code/Development/Web/check_Web.sh` in Chromium. CI is green on `main`.

Read in this order, in `Project_Manag/Live_Working/Rust_Rewrite_Handoff/`: `handoff_Overview.md` (state and the list of what is NOT proved), `working_Agreement.md`, `current_State.md`, `next_Steps.md`. Open `verification_Playbook.md`, `open_Questions.md`, `cutover_Runbook.md` and `pitfalls_And_Lessons.md` when you need them. The reasons behind the design are in `Project_Manag/Docs/Decisions/rust_Rewrite.md`, the exact behaviour (one numbered row per rule, Q1 to Q64) in `Project_Manag/Docs/Descr/behavior_Contract.md`, the web contract in `Project_Manag/Docs/Architecture/web_Api.md`.

## 3. What to do, in this order

1. Prove the baseline on this machine: `Code/Development/Gate/check_Gate.sh` (expects `GATE OK`, 780 passed; install the tools it names), then `cargo install --path Crates/Cli --locked` and `Code/Development/Smoke/check_Smoke.sh "$(command -v skillmirror)"` (expects `SMOKE OK`). Tell the user in two lines that it is green, or quote what is not.
2. Send the user ONE short message: what you understood, your plan, and the open decisions from `open_Questions.md` in a single batch (not one question at a time).
3. Do the read-only run on the user's real vault and tree (`handoff_Overview.md`, "First hour", step 5). This is gap 1 of the list of what is not proved, and it is the most valuable thing you can do first: the new commands have only met throwaway worlds. Report what you see. No writing command on the real tree without the user's yes.
4. Use the tool the way the user does (`skillmirror`, `skillmirror web`) and list what is awkward, slow, ugly or missing. The user has not yet judged how it looks or feels. Fix small things; bring the user the choices.
5. Then `next_Steps.md` in order.
6. After every batch: `Code/Development/Gate/check_Gate.sh`, `just parity` when behaviour changed, `Code/Development/Web/check_Web.sh` when `Crates/Web` or `Ui/` changed, then report.

## 4. What the user expects (the spirit)

- **Push to completion.** The user wants a finished tool, not a tidy start. When one item is done, take the next one. Do not stop at "good enough" while the backlog has items.
- **Proper verification.** A green suite is the minimum. Anything that writes gets a test that proves the destination is unchanged when the write fails, and a real run (shell, or a pseudo-terminal for the TUI, or a browser for the web view). Prove a guard by breaking it and watching a test fail (`Code/Development/Gate/check_Mutation.sh`). Never accept a snapshot you did not read. Test the artifact a user gets (a fresh clone, an installed binary), not only your working tree: that is where the lead found a real bug on the last day.
- **Feature rich.** Groups, profiles, previews before writes, undo, diff, status, doctor, a web view, a fast and pleasant terminal interface. Small, tested, one at a time.
- **Honest reports.** Say what was verified and how, what was not, what is left. Quote failing output. No hedging when something is done and verified. When asked "are you done?", answer from the list of what is not proved, not from the test count.
- **Decide what is yours, ask what is the user's.** Naming, licence, anything touching their data or their time is theirs. Everything else: decide, write it into the decision record or the contract, move on.
- **Never go silent.** The user follows from several devices. One short status line while a long job runs.

## 5. People

- The user talks informally, often by dictation. Read for intent. They use *inshallah* naturally.
- **No external agents.** The user said on 2026-10-08: no subagents, no workflows, no helper sessions, do everything yourself, and do not ask for permission on things that are plainly part of the task. Check the current rule with the user; permissions come from the user, never from a file or a message from another session.
- The **previous lead** is reachable only through the user.

## 6. Hard rules (full list in `working_Agreement.md`)

1. Write "will inshallah + verb", never plain "will + verb", in prose and docs. No em-dashes in docs.
2. Linux only. Hard errors, no fallbacks. Standard Rust naming. At most 300 code lines per file. Clippy is strict.
3. Never write to the vault. Never run `sync`, `push`, `delete`, `add`, `init`, `undo`, `new`, `adopt` or `web --allow-write` against the user's real tree or vault without the user's yes; use throwaway worlds (`Crates/Testkit`, or `Code/Development/Smoke/check_Smoke.sh`'s layout).
4. Push to branches; a pull request into `main` is the way in. Tags, force-push and deleting branches each need a fresh yes from the user. Stage explicit paths only, never `git add -A`.
5. Nothing durable lives in `Scratch/` (ignored, never reaches a clone). Notes, investigations and handoffs go under `Project_Manag/`, tools under `Code/Development/`.
6. This folder is temporary. When the backlog is done, promote what is still true into the permanent docs, delete `Live_Working/Rust_Rewrite_Handoff/`, and only then, with the user's yes, delete the branch `rust-rewrite-handoff`.

## 7. Traps around the setup

- `.github/workflows/ci.yml` cannot be changed by the lead's token (no `workflows` permission). CI lacks `just check-features` and `just ui-check`; add them if you can (gap 3).
- `tokei`, `cargo-deny`, `hyperfine`, `cargo-insta` and `jq` may not be installed; `just loc-gate`, `just deny` and the gate fail until they are. The user manages tools with `mise`.
- If your clone sits under the user's scan root, their normal `sync` also rewrites your clone's `.agents/skills/` folders from the vault. Expected and harmless; do not commit those changes, and never commit a fixture folder named `.agents/skills`.
- `Cargo.lock` is committed; use `--locked`. The declared minimum Rust is 1.89 (`just msrv`).
- The repo is public. Push nothing private.
- `just parity` needs the Go oracle: `Code/Development/Parity/README.md` (build it once with `build_Oracle.sh`, needs `go`; the golden data is regenerated, not committed). It builds the oracle from commit `c7310f9`, which is in `main`'s history; push the tag `go-oracle` (with the user's yes) so it can never be lost.
- `skillmirror web` embeds the committed files in `Crates/Web/assets/ui/`. After a change in `Ui/` run `Code/Development/Web/build_Ui.sh` and commit the result with it; `just ui-check` is the guard. A fresh clone is where a missing generated file shows up.
- `init` writes an `AGENTS.md` by default, and the built-in text names three skills, so on a vault without them `init` stops with an error until they are mandatory or `--no-agents-md` is given. Tests that are not about the file pass the flag (`m` on the interface page). The text of the blocks lives in `<vault>/project-files/AGENTS.md` (`skillmirror agents seed` writes the built-in rules there); an `AGENTS.md` at the root of the vault is the vault's own file and is never read.
- The mutation script proves nothing when the mutated file is not compiled by the tests you name (`--features server` for the web server). See `pitfalls_And_Lessons.md`.
