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

`server/` holds the one place where an async runtime exists (`axum` 0.8 with only the `http1`, `json` and `tokio` features, `tokio` with a two-thread runtime); `skillmirror-core`, the CLI and the TUI never see it (`just check-deps`). The Cli enables it through its default feature `web`; `--no-default-features` builds without it and the command then says so. `serve(ServeConfig, ready)` blocks until Ctrl-C or the idle time.

- `guard.rs`: the middleware every request passes. Host must be `127.0.0.1:PORT` or `localhost:PORT` (a rebound name is refused); a foreign `Origin` or a `Sec-Fetch-Site` of `cross-site` or `same-site` is refused (`none` only for a plain read); the link carries a one-time token that is traded for a session cookie (`HttpOnly`, `SameSite=Strict`) and never works again; everything else needs that cookie; a change must be a POST with `Content-Type: application/json` and the header `X-Skillmirror: 1`, which a cross-site form cannot send. Secrets are compared in constant time. Every answer carries a Content-Security-Policy without `unsafe-inline`, `nosniff`, `no-referrer`, `DENY` framing and `no-store`; there is no CORS.
- `state.rs`: the shared state: configuration, token and sessions, the snapshot (vault, one scan, the comparison of every project; dropped after each write, taken afresh for each plan), stored plans (`plans.rs`: single use, 8 at most, 15 minutes), jobs (`jobs.rs`: one write at a time, results kept for polling).
- `pages/`: server-rendered pages with `maud` (overview, sync, push, history, doctor, settings, and the static report served as it is with its own policy). The script `assets/app.js` only adds behaviour and builds everything it shows with text nodes.
- `api/`: `POST /api/plan` (sync or push, chosen skills, optionally one project; a fresh look at the disk; the answer holds the rows and their diffs), `POST /api/apply` (takes the stored plan out and runs it as a job through `ops::run_plan`: backup, drift guard and per-folder failure as everywhere), `GET /api/job/{id}`, `POST /api/undo-plan` (applied through `/api/apply` too), `POST /api/rescan`. Without `--allow-write` every one of them but the rescan answers 403.

Tests: `server/tests/` drive the router in process (guard, pages with names that are markup, the whole plan-apply-undo flow, a plan used twice, a folder edited after the plan, a second change while one runs); `Crates/Cli/tests/web.rs` starts the real command and talks to it over a socket (loopback bind read from `/proc/net/tcp`, the token once, a wrong Host, Ctrl-C, the idle time); `Code/Development/Web/check_Web.sh` drives it all in Chromium.
