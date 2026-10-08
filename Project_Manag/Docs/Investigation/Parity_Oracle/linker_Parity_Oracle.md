# linker_Parity_Oracle

The Go tool at commit `c7310f9` recorded as a reproducible oracle, and the evidence of how `skillmirror` compares to it: 131 fixture scenarios, 87 MATCH, 44 EXPECTED-DIVERGENCE, 0 UNEXPECTED in the last full run (round 2). The recorded outputs (11 MB) are not committed; the scripts that rebuild them are in `Code/Development/Parity/` (its `README.md` has the one-line commands). Numbers describe one build and go stale with any change to `Crates/`.

## Docs

- [Oracle setup](oracle_Setup.md): fresh clone to a green parity run (rebuild the Go oracle in a detached worktree, exact build commands, recorded binary hashes and golden digest), how the 131 scenarios and the TTY-free driver work, what each script does, how to point the harness at a new Rust binary, how to read MATCH, EXPECTED-DIVERGENCE and UNEXPECTED, how to add a scenario, the determinism rules, why golden is not committed, what must survive the Go removal at cutover, and how to verify the Rust-only features (backup and `undo`, `status`, `diff`, `doctor`) next to the oracle. Open it to run, extend or trust the oracle.
- [Divergences table](divergences.tsv): the machine-readable list of the 44 scenarios where Rust differs on purpose (scenario, contract id, which checks it covers, long reason); `compare.py` reads this file, so edit it when an intended difference is added or disappears.
- [Divergences explained](divergences_Explained.md): the same 44 grouped by behavior-contract quirk (Q2 to Q33) and design decisions, one line of reason each. Open it to learn why a scenario differs or to find which contract row decided it.
- [Parity report](parity_Report.md): the last result condensed: class counts, what was compared and what was not, changes since round 1, five observations that are not failures, and the known gaps (no scenario with a `..` scan root, driver is a stand-in for the TUI flows, Rust-only features untested here). Open it before trusting or quoting a parity number.
