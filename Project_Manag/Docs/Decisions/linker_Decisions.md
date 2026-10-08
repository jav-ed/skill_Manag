# linker_Decisions

Decisions that shape the project and the reasons behind them, written so a later reader does not have to re-derive them. Each file states what was decided, what was rejected, where a research report was overruled, and which rulings still wait for the user. Evidence and measurements stay in the Investigation reports; a decision file only links to them.

## Docs

- [Rewrite in Rust as skillmirror](rust_Rewrite.md): the name, the Cargo workspace layout, the transition from Go, the chosen crates for the engine, CLI, TUI and web view, the feature scope (build now, later, never), licence consequences, the phase plan, and the list of rulings still open. Open it before adding a dependency, moving code between crates, or questioning why a library was chosen or rejected.
