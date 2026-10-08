# Cutover runbook

The steps that replace the Go tool on `main` by the Rust tool. Every step that leaves the machine (a tag, a push to `main`, deleting a branch) needs the user's explicit yes; the checks and the local commit do not. Status on 2026-10-08: the commit that deletes the Go tree is done, and the user asked for the merge into `main`, which is made from pull request #1 with a merge commit after a fresh-clone check (see "Merge"). The `go-oracle` tag was not pushed (not asked for), so `c7310f9` stays reachable only through `main`'s history; the promotion of this folder and the branch deletion are still open.

## Before

1. The pull request of `rust-rewrite-handoff` is green (CI: fmt, clippy on 1.99, tests, loc-gate, check-deps, deny) and the head is the commit to merge.
2. Run the gate locally: format and clippy on 1.97 and 1.99, `just loc-gate`, `just check-deps`, `just deny`, all tests.
3. `just parity` shows 90 match, 41 expected divergences and 0 unexpected (131 scenarios; 87, 44 and 0 when not run as root). The harness builds the Go oracle from commit `c7310f9`, so it needs that commit to stay reachable: do step 4 first.
4. Tag the Go commit so the oracle can always be rebuilt: `git tag go-oracle c7310f9`, then `git push origin go-oracle` (outward). Decide whether the golden data (about 11 MB, regenerable with `run_Scenarios.sh`) stays out of the repository; the recommendation is out, and it is out today (`Scratch/` is ignored).
5. Read the PR description and the README install section once more; they name the branch.

## The commit

Done on the branch (the user allowed it): the Go tree is deleted, `main.go`, `cmd/`, `internal/`, `go.mod`, `go.sum`, `styles/`. The same commit did what this list says:

- `Cargo.toml` line 1 (the comment about Go sources) goes.
- `.gitignore`: the Go lines (`skill_Manag`, `skill_manag`, `*.test`, `*.out`, `go.work*`) go; keep `target/`.
- `Docs/Architecture/go_Legacy.md` and the contract sections that cite Go files stay: they are the record of the old behaviour, and the citations name commit `c7310f9` (tag `go-oracle`).
- README: the install section tells `git clone` and `just install` without `git checkout rust-rewrite-handoff`; the first line drops the work-in-progress banner when the owner agrees.
- `doc_Start.md`: drop the "Go sources, until the cutover" entry points.
- `git worktree remove --force Scratch/Oracle/src` on any machine that has the oracle worktree (`build_Oracle.sh` makes it again).
- Build and install check: `cargo build --profile dist`, `cargo install --path Crates/Cli --locked`, `skillmirror doctor`, `skillmirror --version`. Checked on 2026-10-08 on the branch: the dist build takes about 1.5 minutes and gives a 4.8 MB binary that passes `doctor` and a `sync --dry-run`.

## Merge

Merge into `main` with a merge commit (`git merge --no-ff rust-rewrite-handoff`), push `main` (outward), take the pull request out of draft or close it with the merge. The user decides which of the two.

## Promote the handoff and clean up

`Live_Working/Rust_Rewrite_Handoff/` is a transfer vehicle: when the backlog is done, move what is still true into permanent places and delete the folder (the plan is in `handoff_Overview.md`, section "What happens to this handoff branch"):

- verification playbook and pitfalls: `Docs/Setup/` or `Docs/Descr/`;
- current state and the command table: already mirrored in the README and `Docs/Architecture/`;
- open questions the user has not answered: `Docs/Decisions/`;
- `open_Issues.md` stays and points at what is left.

Delete the branch last, and only after asking: locally `git branch -d rust-rewrite-handoff`, on the remote `git push origin --delete rust-rewrite-handoff`.

## The way back

The Go tool is not lost: `git checkout go-oracle` (or `c7310f9`) holds it. A bad merge on `main` is undone with `git revert -m 1 <merge commit>`. The backups of the Rust tool are in `~/.local/state/skillmirror/backups/`; `skillmirror undo` brings a run back.
