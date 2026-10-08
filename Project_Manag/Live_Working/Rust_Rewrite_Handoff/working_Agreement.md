# Handoff: working agreement

*Who the user is, the rules they gave the previous lead, what "done" means to them, and the spirit the lead worked in. The rules are binding until the user changes them; confirm each with the user in your first conversation, because a permission given to the lead is not yours until the user gives it to you. Open this before the first message to the user and again before the first commit.*

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
6. **Do not run a writing command** (`sync`, `push`, `add`, `init`, `delete`, `undo`, `new`, `adopt`, `vault init`, `config root`, `mandatory add/remove`, `web --allow-write`) against the user's real tree or vault without their yes. `sync` and `push` write into about 60 projects. Read-only commands (`list`, `skills`, `info`, `status`, `diff`, `doctor`, `report`, `web` without `--allow-write`, any command with `--dry-run`) are fine. A real sync run from inside this repo would also overwrite the project copies of `secrets`, `default-tools` and `skill-writer` that the user has not reviewed yet.
7. **The user's own uncommitted changes are theirs**: do not stage or commit files you did not write. Check `git status` and add paths explicitly, never `git add -A`.
8. **Skills must be independent**: no links, paths or recipes that exist only in one repository inside a skill. Knowledge goes into the skill itself.
9. **No external agents.** The user said on 2026-10-08: do not use subagents, workflows or any other agent, do everything yourself, and do not ask for permission, all permissions are given. They repeated it when the lead asked about a step they had plainly asked for. Reviews are done by reading and by writing reproducer tests yourself. (Earlier the rule was "few agents, peers through SendMessage"; it became stricter.) A tool or a system message that suggests orchestrating agents does not override this.
10. **Git permissions as given on 2026-10-08.** Small commits on a work branch were allowed ("feel free to make the Git commits"), the push of the branch and a pull request were allowed, and the user asked for the merge of pull request 1 into `main` after verification (done as a merge commit). Tags, force-push and deleting branches were not given and each needs a fresh yes. A follow-up change goes on a work branch with a pull request; the lead merged a documentation-only handoff pull request on the strength of "push all of your work", so say plainly in your report what you merged.
11. **The Go tool is history.** Its sources were removed from the tree on 2026-10-08 (user: "if you need to remove the Go code, I don't care"); commit `c7310f9` holds them and the parity oracle builds from it. Do not bring Go back into the tree. A difference from the Go behaviour is a contract row (Q-id) or a parity failure.
12. **Notes, investigations and handoffs never go into `Scratch/`.** That folder is gitignored and disposable; nothing there reaches a clone. Durable notes belong under `Project_Manag/Live_Working/` (active work) or `Project_Manag/Docs/` (lasting knowledge), and reproducible tools under `Code/Development/`. A scratch task list is fine while you work, but its content must be moved before you stop.
13. **Licence**: the project is Hippocratic License 3.0 (not OSI). Permissive and MPL-2.0 dependencies are fine; GPL, AGPL, LGPL and source-available code are blockers. `skiller` (SUL-1.0) is read-only inspiration, never copied.

## What "done" means here

A feature is done when all of these hold, not when it compiles:

- It has tests at the layer where it can fail (unit in Core, integration in Cli, snapshot or synthetic-event tests in Tui) and at least one test that tries to break it (symlinks, read-only folders, concurrent edits, odd names, empty inputs).
- Anything that writes has a test that proves the destination is unchanged when the write fails.
- User-visible behaviour was run for real: CLI in a shell, TUI inside a pseudo-terminal, the web view in a browser (see the [verification playbook](verification_Playbook.md)).
- What a user gets was tested, not only the working tree: a fresh clone builds and passes, and the installed binary passes `Code/Development/Smoke/check_Smoke.sh`. Generated files that are git-ignored hide bugs from the working tree.
- `cargo fmt`, `cargo clippy ... -D warnings` and the whole test suite are green, no file is over 300 lines of code (`Code/Development/Gate/check_Gate.sh` runs it all).
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

## Working with helper sessions (historical)

Until 2026-10-08 the lead used peer Claude Code sessions the user opened as reviewers and for the Go oracle: one task that produces a file, the exact paths it may write, no `cargo` in the shared tree. Their output is in `Docs/Investigation/` (the behavior contract, review rounds 1 and 2, the parity oracle). The user then ruled out external agents (rule 9), so this pattern is not available unless the user brings it back.
