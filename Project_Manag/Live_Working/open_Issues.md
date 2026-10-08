# open_Issues

Active items. Detail and the design for each live in the [Rust rewrite handoff](Rust_Rewrite_Handoff/handoff_Overview.md); the ordered list is [next steps, section A](Rust_Rewrite_Handoff/next_Steps.md).

The Rust tool is on `main` (merged 2026-10-08, pull request 1). What follows is what is left.

| Item | State | Where |
|---|---|---|
| Run the new commands on the user's real vault and tree, read-only | not done: only throwaway worlds so far | [next steps A1](Rust_Rewrite_Handoff/next_Steps.md) |
| Judge how the terminal and web interfaces look and feel | not done: verified by tests and screenshots the lead read, not by the user | [next steps A2](Rust_Rewrite_Handoff/next_Steps.md) |
| CI steps still to add | `just check-features`, `just ui-check` (needs node 22.12 in the job) and optionally the Chromium check are not in `.github/workflows/ci.yml`: the session that wrote them could not push a workflow file (no `workflows` permission). A person adds them to the `check-deps` job. Until then `Code/Development/Gate/check_Gate.sh` holds them | [next steps A3](Rust_Rewrite_Handoff/next_Steps.md), [Code/Development/Gate/README.md](../../Code/Development/Gate/README.md) |
| Tag `go-oracle` at `c7310f9` | not pushed (needs the user's yes) | [next steps A4](Rust_Rewrite_Handoff/next_Steps.md) |
| Fresh review of the backup and undo code and of the web server | not done: rounds 3 and 4 were self reviews | [next steps A5](Rust_Rewrite_Handoff/next_Steps.md) |
| Other machines: run as a normal user, another browser, a small machine | not done | [next steps A6](Rust_Rewrite_Handoff/next_Steps.md) |
| Decisions that wait for the user | listed with defaults | [open questions](Rust_Rewrite_Handoff/open_Questions.md) |
| Promote the handoff folder, then delete it and the branch `rust-rewrite-handoff` | not started; deleting needs the user's yes | [cutover runbook](Rust_Rewrite_Handoff/cutover_Runbook.md) |
| Provenance lock | deferred by the user | [next steps D](Rust_Rewrite_Handoff/next_Steps.md) |
| Skills track (three edited skills to review and copy to the vault) | waiting for the user | [next steps C](Rust_Rewrite_Handoff/next_Steps.md) |
