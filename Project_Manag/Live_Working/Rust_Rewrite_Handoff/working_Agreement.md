# Handoff: working agreement

*Who the user is, the rules they gave, what "done" means to them, and the spirit the previous lead worked in. The rules are binding; the spirit is how the lead tried to meet them. Open this before the first message to the user and again before the first commit.*

## The user and how they communicate

- Writes informally and often by dictation, so sentences are loose and words are sometimes wrong. Read for intent. When two readings are plausible, take the one that moves the work forward and say which one you took.
- Uses *inshallah* and *alhamdulillah* naturally. Their writing rule (below) is part of the product's voice.
- Wants to be informed without being asked to babysit. Short status lines while work runs, plain answers to questions, and a clear list of what is theirs to decide.
- Runs several Claude Code sessions side by side and uses them as colleagues. They do not like a swarm of subagents; they prefer a few peers with clear jobs.
- Owns the vault, about 60 projects and every `.agents/skills` folder in them. Nothing the tool does should surprise them there.

## Standing rules (from the user, binding)

1. **Writing form**: write "will inshallah + verb", never plain "will + verb", in prose, docs and replies. Do not force it into present-tense facts. Details: `.agents/skills/inshallah/SKILL.md`. No em-dashes in docs (doc-start rule).
2. **Linux only.** No Windows, no macOS code paths. `lib.rs` has a `compile_error!` for other targets.
3. **Hard errors, no fallbacks.** A bad config, a missing vault, a symlinked `.agents`, an unsupported filesystem: stop with a clear message and a hint. Never guess, never silently continue with a weaker behaviour.
4. **Standard Rust naming** in code (snake_case, CamelCase). Files and folders in docs and the repo follow the project convention (lowercase-first `.md` files, uppercase-first folders); see the `coding` skill.
5. **The vault is read-only for you.** Vault path: see [internal repo paths](../../Docs/Setup/internal_Repo_Paths.md). Never write, stage or commit there. When a skill needs a change, edit the copy in the project's `.agents/skills/`, tell the user, and let them review and copy it over.
6. **Do not run a real `sync` or `push`** against the user's tree. Both write into about 60 projects. Read-only commands (`list`, `skills`, `sync --dry-run`, `--check`) are fine. A real sync run from inside this repo would also overwrite the project copies of `secrets`, `default-tools` and `skill-writer` that the user has not reviewed yet.
7. **The user's own uncommitted changes are theirs**: do not stage or commit files you did not write. Check `git status` and add paths explicitly, never `git add -A`.
8. **Skills must be independent**: no links, paths or recipes that exist only in one repository inside a skill. Knowledge goes into the skill itself.
9. **Few agents.** Do most of the coding yourself. Parallel work goes to the user's peer sessions through SendMessage, each with a self-contained task, a hard file boundary (they write only named files) and no `cargo` runs in the shared tree. Do not start many subagents or workflow runs, even when a tool offers it.
10. **Commit and push only on request.** The user asked for the push of `rust-rewrite` on 2026-10-08. Pushing to `main`, opening a PR, tagging or force-pushing each needs a fresh yes.
11. **Frozen Go tree.** Do not edit Go code (`main.go`, `cmd/`, `internal/`, `go.*`) before cutover. It is the behaviour reference.
12. **Licence**: the project is Hippocratic License 3.0 (not OSI). Permissive and MPL-2.0 dependencies are fine; GPL, AGPL, LGPL and source-available code are blockers. `skiller` (SUL-1.0) is read-only inspiration, never copied.

## What "done" means here

A feature is done when all of these hold, not when it compiles:

- It has tests at the layer where it can fail (unit in Core, integration in Cli, snapshot or synthetic-event tests in Tui) and at least one test that tries to break it (symlinks, read-only folders, concurrent edits, odd names, empty inputs).
- Anything that writes has a test that proves the destination is unchanged when the write fails.
- User-visible behaviour was run for real: CLI in a shell, TUI inside a pseudo-terminal (see the [verification playbook](verification_Playbook.md)).
- `cargo fmt`, `cargo clippy ... -D warnings` and the whole test suite are green, no file is over 300 lines of code.
- The behaviour is written in the contract (`behavior_Contract.md`) when it differs from Go or is new, and the parity table is updated.
- The report to the user says what was verified and what was not, in plain words.

## The spirit (what the user asked the lead for, and what to keep doing)

- **Push toward completion.** The user wants the tool finished, not a clean skeleton. When a phase ends, start the next; when a review finds something, fix it and add the test. Do not stop at "good enough" while the backlog in [next steps](next_Steps.md) has items.
- **Proper verification.** Two independent reviews and a parity oracle exist because the first version of anything has bugs. Keep asking: how would this lose a user's file? Try it. Run the reproducers of the review rounds again after each fix.
- **Feature rich.** The user values nested groups, profiles, previews before writes, undo, diff, status, doctor, a web view, a fast and pleasant TUI. Several of these are planned in the [decision record](../../Docs/Decisions/rust_Rewrite.md); build them, keep each small and tested.
- **Be honest in reports.** If a test fails, say so with the output. If something was skipped, say that. When something is done and verified, state it plainly without hedging.
- **Ask only for what is the user's.** Naming, licence, deleting their data, spending their time. Everything else: decide, write the decision down in the ADR or the contract, move on. The [open questions](open_Questions.md) file lists what is still theirs.
- **Do not go silent.** The user follows from several devices. A one-line status while a long job runs is worth more than a perfect summary later.
- **Leave notes.** Anything you would have to re-derive tomorrow goes into this folder, in a file the next person can find from the overview.

## Working with helper sessions

The lead's helpers were peer Claude Code sessions the user opened. Pattern that worked: give one helper a task that produces a file (behavior contract from old code, a golden-output oracle, CI and tooling files, a read-only review, a parity classification), name the exact paths it may write, tell it not to run `cargo` in the shared tree (Cargo.lock and `target` contention; it copies the crate into a scratch folder), and ask it to message back. Their names change between sessions; list them with ListAgents and copy the row exactly, or use the session id from an earlier message.
