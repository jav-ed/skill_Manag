# linker_Architecture

Code structure of the Rust tool `skillmirror`, and of the Go tool it replaces. Open the file that matches the task; the labels say what each one answers.

- [Rust overview](rust_Overview.md): the five crates and the rule that dependencies point at Core, the flow of a write (settings, vault, scan, plan, apply), the rules the code keeps (hard errors, no write through links, backup before replace, per-target failure, no work on the interface thread), error and exit codes, threads, and the layers of tests and checks. Open it first, and before adding a command, a screen or a dependency.
- [Core modules](core_Modules.md): a table of every module of `Crates/Core` with its files and what it owns. Open it to find where a behaviour lives.
- [Front ends](front_Ends.md): how the command line, the interface and the HTML report use Core: command files, the shared write pipeline, the interface's state machine with job ids, screens, tests, and the report renderer. Open it before changing a command, a screen or the report.
- [Go legacy](go_Legacy.md): the structure of the frozen Go tool (`cmd/`, `internal/`, `styles/`). Open it only to compare a behaviour with the old tool or to read the parity harness.
