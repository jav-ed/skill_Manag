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

- Screens (`screens/`): `menu`, `work` (sync, push, delete, list, skills, add, init as phases `Loading`, `Select`, `Planning`, `Confirm`, `Diffing`, `Diff`, `Running`, `Done`), `history` (runs, question, results), `place` (folder picker and name for add and init), `setup` (the wizard).
- Jobs (`jobs.rs`, `jobs_install.rs`, `undo.rs`): every disk read or write runs on its own thread and answers over one `mpsc` channel (`event.rs`). Each answer carries a job id; the app keeps the id it waits for and drops any other answer (`app/jobs_events.rs`). A page that works out a plan applies exactly that plan.
- Discipline: the header arrow, keys and a second job are refused while a job writes; the first Ctrl-C during a write only warns.
- Input: `backend.rs` on `ratatui-termina`, own key tables in `binding.rs` that also produce the help text, `tui-input` for text fields, `nucleo-matcher` for the fuzzy filter.
- Tests (`src/tests/`): synthetic events against a `TestBackend` with `insta` snapshots, plus real terminal tests in `Crates/Cli/tests/interface*.rs`.

## Web (`Crates/Web`)

`render_report(&ReportData, generated_at)` turns the data of `ops::report_data` into one HTML string with `maud`: `report/mod.rs` (page, header, counts), `matrix.rs`, `tree.rs`, `skill.rs` (cards and diffs), `problems.rs`, plus `report.css` and `report.js` inserted as they are. Every value is escaped by `maud`, the page forbids any request through its Content-Security-Policy, and a marker comment after the doctype lets a later run replace the file safely. The CLI side is `Crates/Cli/src/commands/report.rs`. 

### The local server (cargo feature `server`, `skillmirror web`)

`server/` holds the one place where an async runtime exists (`axum` 0.8 with only the `http1`, `json` and `tokio` features, `tokio` with a two-thread runtime); `skillmirror-core`, the CLI and the TUI never see it (`just check-deps`). The Cli enables it through its default feature `web`; `--no-default-features` builds without it and the command then says so. `serve(ServeConfig, ready)` blocks until Ctrl-C or the idle time. The interface it serves is a separate project, [`Ui/`](../../../Ui/README.md); the two meet in the JSON API ([web_Api.md](web_Api.md)) and in the built files.

- `guard.rs`: the middleware every request passes. Host must be `127.0.0.1:PORT` or `localhost:PORT` (a rebound name is refused); a foreign `Origin` or a `Sec-Fetch-Site` of `cross-site` or `same-site` is refused (`none` only for a plain read); the link carries a one-time token that is traded for a session cookie (`HttpOnly`, `SameSite=Strict`) and never works again; everything else needs that cookie; a change must be a POST with `Content-Type: application/json` and the header `X-Skillmirror: 1`, which a cross-site form cannot send. Secrets are compared in constant time. Every answer carries a Content-Security-Policy without `unsafe-inline` (`default-src 'none'`, scripts and styles from the server only), `nosniff`, `no-referrer`, `DENY` framing and `no-store`; there is no CORS.
- `files.rs` and `build.rs`: the interface files (`Crates/Web/assets/ui/`, built by `Ui/` and committed) are turned into a table of `include_bytes!` at build time; the router serves a name from the table, a folder's `index.html`, or 404. No address reaches anything but the table, and a build has no run-time files and needs no node.
- `state.rs`: the shared state: configuration, token and sessions, the snapshot (vault, one scan, the comparison of every project; dropped after each write, taken afresh for each plan), stored plans (`plans.rs`: single use, 8 at most, 15 minutes), jobs (`jobs.rs`: one write at a time, results kept for polling).
- `api/`: `read.rs` and `read_more.rs` (the reads), `plan.rs` (a fresh look at the disk, the rows with their diffs), `apply.rs` (takes the stored plan out and runs it as a job through `ops::run_plan`: backup, drift guard and per-folder failure as everywhere; the job and the polling), `undo.rs`. Without `--allow-write` the changing endpoints answer 403.
- `report.rs`: `/report`, the static report served with its own policy.

Tests: `server/tests/` drive the router in process (`guard`: who gets in; `read`: the JSON contract; `flow` and `undo`: plan, apply, undo, a plan used twice, a folder edited after the plan, a second change while one runs; `files`: the served files, no inline script or style in any built page, no way out of the table; `idle`); `Crates/Cli/tests/web.rs` starts the real command and talks to it over a socket (loopback bind read from `/proc/net/tcp`, the token once, a wrong Host, Ctrl-C, the idle time); `Code/Development/Web/check_Web.sh` drives the interface in Chromium (a project named like markup, plan, diff, apply, push, undo, the skills page, the theme, the refusals, policy violations).
