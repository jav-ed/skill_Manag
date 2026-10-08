# open_Issues

Active items. Detail and the design for each live in the [Rust rewrite handoff](Rust_Rewrite_Handoff/next_Steps.md).

| Item | State | Where |
|---|---|---|
| Backup store, `undo`, `history` | done and reviewed (rounds 2 and 3); a fresh pair of eyes is still welcome | [current state](Rust_Rewrite_Handoff/current_State.md) |
| `status`, `diff`, `doctor`, `bridge`, scan progress line | done (contract Q41 to Q45) | [next steps, item 3](Rust_Rewrite_Handoff/next_Steps.md) |
| Interface: scan problems, history, add, init, changes page | done (Q46 to Q49) | [next steps, item 4](Rust_Rewrite_Handoff/next_Steps.md) |
| HTML report (`report`) and the local web server (`web`, write mode behind `--allow-write`) | done (Q50, Q56) | [next steps, item 5](Rust_Rewrite_Handoff/next_Steps.md) |
| Docs pass | done on 2026-10-08 (README, architecture, concept, doc_Start, Astro leftovers removed) | [next steps, item 6](Rust_Rewrite_Handoff/next_Steps.md) |
| Cutover from Go to Rust | not started, needs the user's explicit yes | [next steps, item 7](Rust_Rewrite_Handoff/next_Steps.md) |
| Decisions that wait for the user | listed | [open questions](Rust_Rewrite_Handoff/open_Questions.md) |
| CI steps still to add | `just check-features` and `just ui-check` (the second needs node 22.12 in the job) are not in `.github/workflows/ci.yml`: the session that wrote them could not push a workflow file (the GitHub App has no `workflows` permission). A person adds them to the `check-deps` job. Until then `Scratch/gate.sh`-style local runs hold them | [Code/Development/Web/README.md](../../Code/Development/Web/README.md) |
| Web interface (`Ui/`) | done, driven in Chromium; the contract with the server is `Docs/Architecture/web_Api.md` | [front_Ends.md](../Docs/Architecture/front_Ends.md) |

