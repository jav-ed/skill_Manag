# front_Ends

How the three front ends use Core. None of them contains engine logic: each reads from `ops`, shows, asks and calls back.

## Cli (`Crates/Cli`)

`main.rs` parses the grammar (`args.rs`), `commands/mod.rs` dispatches, and every command is one file in `commands/` that opens a context, calls one or two `ops` functions and hands rows to `output/`.

- `commands/context.rs`: `open` (settings, vault, scan with the stderr spinner) and `scan_root`.
- `commands/pipeline.rs`: the shared flow of the writing commands: show the plan, ask (never without `--yes` on a pipe), take a backup run, apply, show the result. `Hooks` lets `add` and `init` make a folder before the first write and the configured links after it.
- `commands/{mirror,install,delete,backup,bridge,status,diff,doctor,report,list,skills,migrate}.rs`: one command each.
- `output/`: the only place that prints. Text through `anstream` (colour off for pipes and `NO_COLOR`), JSON documents with `serde`, the scan spinner (`progress.rs`), lossy paths for JSON (`lossy.rs`).
- `report.rs` and `exit.rs`: `CliError` with its hint, and the exit codes.

## Tui (`Crates/Tui`)

A pure state machine: `App::handle(Event)` takes a key, a mouse event or a job report and changes state; `ui/` draws state and records every clickable rectangle in a `HitMap`; nothing in `app/` touches the terminal.

- Screens (`screens/`): `menu`, `work` (sync, push, delete, list, add, init as phases `Loading`, `Select`, `Planning`, `Confirm`, `Diffing`, `Diff`, `Running`, `Done`), `history` (runs, question, results), `place` (folder picker and name for add and init), `setup` (the wizard).
- Jobs (`jobs.rs`, `jobs_install.rs`, `undo.rs`): every disk read or write runs on its own thread and answers over one `mpsc` channel (`event.rs`). Each answer carries a job id; the app keeps the id it waits for and drops any other answer (`app/jobs_events.rs`). A page that works out a plan applies exactly that plan.
- Discipline: the header arrow, keys and a second job are refused while a job writes; the first Ctrl-C during a write only warns.
- Input: `backend.rs` on `ratatui-termina`, own key tables in `binding.rs` that also produce the help text, `tui-input` for text fields, `nucleo-matcher` for the fuzzy filter.
- Tests (`src/tests/`): synthetic events against a `TestBackend` with `insta` snapshots, plus real terminal tests in `Crates/Cli/tests/interface*.rs`.

## Web (`Crates/Web`)

`render_report(&ReportData, generated_at)` turns the data of `ops::report_data` into one HTML string with `maud`: `report/mod.rs` (page, header, counts), `matrix.rs`, `tree.rs`, `skill.rs` (cards and diffs), `problems.rs`, plus `report.css` and `report.js` inserted as they are. Every value is escaped by `maud`, the page forbids any request through its Content-Security-Policy, and a marker comment after the doctype lets a later run replace the file safely. The CLI side is `Crates/Cli/src/commands/report.rs`. A loopback server with the security minimum of the decision record (section 5) would join this crate behind a cargo feature; it is not started.
