# Cutover runbook

*How the Go tool was replaced by the Rust tool on `main`, what is still open, and the way back. Every step that leaves the machine and was not given (a tag, a force-push, deleting a branch) needs the user's explicit yes.*

## Status on 2026-10-08

Done:

- The Go tree is deleted (`main.go`, `cmd/`, `internal/`, `go.mod`, `go.sum`, `styles/`), with the Go lines of `.gitignore` and the Cargo comment. `Docs/Architecture/go_Legacy.md` and the contract sections that cite Go files stay: they are the record of the old behaviour, and their citations name commit `c7310f9`.
- Verified before the merge, on the head that was merged (`41de505`): the gate (`Code/Development/Gate/check_Gate.sh`: 637 tests, fmt and clippy on 1.97 and 1.99, deny, loc-gate, features, check-deps), `just msrv`, a `dist` build, `just parity` against the head's own build (90 match, 41 expected, 0 unexpected), the Chromium check, a fresh clone (build, tests, `Ui` check), `cargo install --path Crates/Cli --locked` into an empty root, and `Code/Development/Smoke/check_Smoke.sh` on that installed binary (all checks pass).
- Merged into `main` as pull request 1 with a merge commit (`b2e11fc`), CI green on the head and on `main`. The user asked for the merge ("push it and merge it to the main") after being told what was and was not verified.
- README install section tells `git clone` and `just install`; the work-in-progress banner is gone.

Not done:

1. **The tag `go-oracle`.** `git tag go-oracle c7310f9`, then `git push origin go-oracle` (outward; needs the user's yes). The parity harness builds the oracle from `c7310f9`, which is reachable from `main` today; the tag protects it from a future history rewrite. Afterwards edit the sentence in `Docs/Architecture/go_Legacy.md` that says "once the maintainer has pushed it".
2. **Promote and delete this folder** (below).
3. **Delete the branch `rust-rewrite-handoff`** (below). It is merged; local work branches named `worktree-wf_*` that may exist in a clone were never pushed and can be deleted with `git branch -D` once you have looked at them.

## Promote the handoff and clean up

`Live_Working/Rust_Rewrite_Handoff/` is a transfer vehicle. When the backlog in [next_Steps.md](next_Steps.md) section A is done, move what is still true into permanent places and delete the folder:

- verification playbook and pitfalls: `Docs/Setup/` or `Docs/Descr/`;
- current state and the command table: mirrored in the README and `Docs/Architecture/`; check that they agree and delete the duplicate;
- open questions the user has not answered: `Docs/Decisions/`;
- `Live_Working/open_Issues.md` stays and points at what is left; remove the rows that point into the deleted folder.

Delete the branch last, and only after asking: locally `git branch -d rust-rewrite-handoff`, on the remote `git push origin --delete rust-rewrite-handoff`.

## The way back

The Go tool is not lost: `git checkout c7310f9` (or the tag, once pushed) holds it, and `git worktree add --detach /tmp/go c7310f9` gives a build tree. A bad merge on `main` is undone with `git revert -m 1 b2e11fc`. The backups of the Rust tool are in `~/.local/state/skillmirror/backups/`; `skillmirror history` lists the runs and `skillmirror undo` brings one back (a second `undo` redoes it).
