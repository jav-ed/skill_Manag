# linker_Investigation

Research that was done to answer a question before building, with the evidence kept so it can be re-checked. Findings are dated and versioned; libraries and tools in this area move quickly, so re-verify a version or a licence before relying on a number older than a few months.

## Docs

- [Rust stack research](Rust_Stack/linker_Rust_Stack.md): six reports that compare libraries and tools for the Rust rewrite (engine, CLI and quality tooling, TUI, web view, prior art among skill managers, name and distribution), plus the requirements they were judged against. Open it to see why a crate was chosen or rejected, how a number was measured, or which crates are blocked by the licence.
- [Review rounds](Review_Rounds/linker_Review_Rounds.md): two adversarial reviews of the Rust rewrite (Core, then everything written after it), each finding with severity, location, scenario, fix and status, the test that closed each fixed one, the review method, and runnable reproducers for the open ones. Open it before fixing a known bug or to check whether a behavior is already reported.
- [Prototypes](Prototypes/linker_Prototypes.md): source archives of the throwaway programs behind the terminal-library decision: the three probes that reproduce the crossterm input stall and check termina and the theme query, and the first TUI prototype with its tests. Open it to re-run the stall reproduction or to check a terminal-library claim after a version bump.
- [Parity oracle](Parity_Oracle/linker_Parity_Oracle.md): how the frozen Go tool is rebuilt and driven as a reference, the 131 comparison scenarios, the table of 44 expected divergences by contract number, the latest parity report, and the proposal for testing features that have no Go counterpart. Open it before changing observable behaviour or before the cutover.
