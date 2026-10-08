# Rust stack: Web view

Recommendation: build the web view in stages and keep the server part optional. Stage 0 is a static, self-contained HTML report written by `<tool> report` (matrix, group browser, skill detail with `SKILL.md` and diffs, client-side filter, dark mode). It needs no server and has no attack surface; the prototype renderer is plain std string building (+37 KiB binary, 41 KiB output for the 60 projects x 82 skills x 395 installs baseline, 11 ms), the production renderer should use maud for automatic escaping (+9 crates, binary size unchanged, +16 CPU-s clean build). Stage 1 is `<tool> web`, a loopback server with server-rendered HTML, vanilla JS (estimate 100 to 200 lines; the prototype's live JS is 15 lines) and SSE for live progress, read-only by default; mutations only with `--allow-write` and a plan/confirm step. Build it on axum 0.8 with minimal features in its own crate behind a cargo feature `web` (off in dev builds, on in release artifacts), so CLI and TUI builds never compile tokio (checked with `cargo tree`). Measured cost of stage 1 over a bare clap binary: +949 KiB stripped (+563 KiB with fat LTO), +39 crates, +108 to +125 CPU-s clean build, 1.4 MiB idle RSS. The synchronous alternative tiny_http is rejected: two open CVEs (2026-07-29), no release since 2022. A SPA (498 npm packages in skillshare's lockfile), WASM full-stack and desktop shells (34 to 352 crates, extra CLIs and system libraries, Slint is GPL) are rejected. Security minimum, verified with curl and in Chrome against a prototype: random port on 127.0.0.1, Host allow-list, one-time launch token exchanged for an HttpOnly SameSite=Strict cookie, Sec-Fetch-Site and Origin checks on every request (load-bearing, see Security model), POST-only mutations, no CORS, CSP, read-only default. Build stage 0 now; decide on stage 1 after the TUI exists.

Related files: [Requirements](rewrite_Requirements.md): the constraints this file is judged against. [Prior art](prior_Art.md): scopes the feature (read-only first). [Engine stack](engine_Stack.md): event channel and diff crates the views consume. [TUI stack](tui_Stack.md): the front-end whose glyphs and palette the web view reuses. [CLI stack](cli_Stack.md): feature flags, CI matrix and supply-chain checks.

## Method and caveats

- Prototype workspace (scratch, not in the repo): `core_sim` (synchronous engine stand-in, std threads and `std::sync::mpsc`, emits typed events; data shaped like the baseline: 60 projects, 82 skills, 395 installs; a real `find` under the scan root gave 387 installs, 82 distinct skills), `report` (static HTML, std only), `guard` (pure admission logic, 176 lines including 4 unit tests), `webx` (axum, 158 lines), `webt` (tiny_http, 125 lines), `cli` (clap derive, features `report`, `web-axum`, `web-tiny`, `web-axum-full`).
- Profiles: `release` is opt-level 3 with `strip = true` and `debug = false` (as in `~/.cargo/config.toml`); `dist` adds `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
- Procedure: fresh target dir, registry pre-fetched, `cargo build --profile P -p cli --features F`, bash `time` for wall and CPU (user+sys), size = `stat` of the stripped binary, crates = unique external packages in `cargo tree -e normal,build --target <host>`, RSS = `VmRSS` in `/proc/PID/status` 5 s after start.
- Caveat on time: the machine ran five other research agents; load average was 9 to 38 on 12 threads. Wall times are unreliable (dist base 37 s versus 147 s for tiny_http), CPU seconds are inflated and noisy (about +-30%). Sizes and crate counts are exact. Read the time columns as order of magnitude and compare rows, not absolutes.
- Not measured: any Node/SPA build, WASM builds, desktop shells (resolved with `cargo tree` only, so crate counts but no build times).

## Option A: static self-contained HTML report

- Measured: the report crate renders matrix, sticky headers, a text filter and a status filter in 79 lines (CSS 0.9 KiB, filter JS 0.3 KiB). With sparse cells (empty cell without attributes) the 60 x 82 baseline is 41,093 B (2,713 B gzipped), generated in 11 ms including process start. First version with a `title` on every empty cell: 176,843 B, so cell markup matters more than anything else.
- Checked in Chrome from a local HTTP origin: the filter reduced 60 rows to 10 (text) and to 35 (status "modified"), no console errors. Opened from `file://` the pane showed a static snapshot only, so JS from `file://` was not exercised. Opening from `file://` has no CSP, so inline CSS and JS are fine there.
- How far it goes: everything read-only. Matrix, group tree (`<details>`), skill detail with `SKILL.md` as escaped `<pre>`, per-pair unified diffs (from `similar`) inside `<details>` so they cost bytes, not render time, status filters, sorting by column, light/dark through `prefers-color-scheme`, print stylesheet. Actions become copy-to-clipboard buttons holding the exact CLI command (`<tool> sync coding --project ...`), so the browser never writes.
- Limits: no live progress, no apply, no confirm flow, stale the moment it is written (stamp it with vault commit and scan time), embedded diffs grow with drift (estimate only: 120 non-ok pairs x 2 KiB = 240 KiB, not measured).
- Templating (benchmark in the Measurements section; same matrix table rendered by each crate). Auto-escaping is a security requirement, not a style choice: skill names, paths and `SKILL.md` text come from disk, and in a server that can delete files one forgotten `esc()` is XSS with full privileges. maud and askama escape by default (`PreEscaped` is the explicit opt-out, [maud docs](https://docs.rs/maud/latest/maud/)). The static report and the live page share one renderer, so pick the escaping renderer at stage 0, not at stage 1. Result: maud costs +9 crates and +16 CPU-s with no size change and checks markup at compile time; askama costs +18 crates and +44 CPU-s, also compile time; minijinja and tera each add 1.2 to 1.5 MiB of binary and +87 to +89 CPU-s and only find template errors at run time. Choose maud (typed, no template files to embed); askama is the second choice if designers should edit real HTML templates.
- Markdown for `SKILL.md`: start with `<pre>`. If rendering is wanted later, pulldown-cmark 0.13.4 (MIT, 4 crates without default features) does not sanitize: its [push_html page](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/html/fn.push_html.html) has no word on raw HTML, so map `Event::Html` and `Event::InlineHtml` to text before rendering, and keep CSP `script-src 'self'` as the second line of defence. comrak 0.56.0 (BSD-2-Clause, 22 crates) escapes raw HTML by default but costs 5 times the crates.
- Verdict: yes, build it first. It is the same renderer functions stage 1 serves, so nothing is thrown away.

## Option B: loopback server, server-rendered HTML

### Server choice (hello-world prototypes, release profile, clean build)

| Server (hello world on 127.0.0.1:0, one route) | Crates | tokio | Clean build CPU-s (wall s) | Stripped size | Delta over empty bin | Rebuild after edit (wall s) |
|---|---|---|---|---|---|---|
| empty `main` | 0 | no | 0.6 (1.429) | 344 KiB | - | 1.020 |
| tiny_http 0.12.0 | 5 | no | 19.0 (8) | 670 KiB | +326 KiB | 1.461 |
| astra 0.4.0 | 18 | yes | 41.1 (23) | 742 KiB | +398 KiB | 5.272 |
| hyper 1.12.0 (+hyper-util, http-body-util, bytes) | 19 | yes | 60.8 (44) | 840 KiB | +496 KiB | 5.417 |
| axum 0.8.9, minimal features | 38 | yes | 116.3 (55) | 1,063 KiB | +719 KiB | 13.898 |
| rouille 3.6.2 | 84 | no | 190.5 (76) | 761 KiB | +417 KiB | 3.875 |
| actix-web 4.15.0, minimal | 103 | yes | 293.6 (69) | 1,739 KiB | +1,395 KiB | 7.459 |
| poem 3.1.12 | 88 | yes | 304.6 (80) | 3,660 KiB | +3,316 KiB | 5.390 |
| salvo 1.0.1 | 112 | yes | 367.8 (102) | 1,438 KiB | +1,094 KiB | 9.694 |
| rocket 0.5.1 | 116 | yes | 528.6 (148) | 3,124 KiB | +2,780 KiB | 29.304 |

- axum 0.8.9 (MIT, last release 2026-04-14, 128 M recent downloads, 66 commits from 27 authors in the last 90 days, hyper 1.12.0 released 2026-10-06): the default. rust-lang's own mdBook serves with axum 0.8.9 behind a default-on cargo feature `serve` (`Repos/mdBook/Cargo.toml:108-129`) from a plain thread running `#[tokio::main]` (`Repos/mdBook/src/cmd/serve.rs:117-144`), reloads through `tokio::sync::broadcast` fed by a synchronous watcher, and opens the browser with `opener`. Pre-1.0: `axum/CHANGELOG.md` lists several unreleased breaking changes on main (serve future type, Router fallbacks, `tower-log` no longer default), so pin `0.8` and expect one migration to 0.9.
- hyper 1.12.0 directly (MIT, 19 crates, +496 KiB in hello form): dufs does this (`Repos/dufs/Cargo.toml:17-19`) and ships upload and delete behind `--allow-upload`, `--allow-delete`, `--allow-all` (`Repos/dufs/README.md:64-70`). Costs half of axum (60.8 versus 116.3 CPU-s, +496 versus +719 KiB) but means hand-written routing, SSE body plumbing and graceful shutdown. Keep as the measured fallback; the `guard` and `report` split makes the swap local.
- tiny_http 0.12.0 (MIT OR Apache-2.0, 5 crates, no tokio, 15 M recent downloads): technically the smallest and it worked: same 14-step test script passed, SSE needs `Request::into_writer` plus manual `flush` because responses go through a 1 KiB `BufWriter` (`Repos/tiny-http/src/client.rs:63`, `Repos/tiny-http/src/request.rs:390`), handlers can be unit tested with the public `TestRequest` (`Repos/tiny-http/src/lib.rs:121`), pool starts at 4 threads and grows (`Repos/tiny-http/src/util/task_pool.rs:30`). Rejected on health: `git log -1` on master is 2023-05-16, 76 open issues, and CVE-2026-66752 (request smuggling through any `Transfer-Encoding` value) and CVE-2026-66753 (CR/LF header injection) are open as issues [#287](https://github.com/tiny-http/tiny-http/issues/287) and [#288](https://github.com/tiny-http/tiny-http/issues/288) since 2026-07-29 with no maintainer reply; Debian lists them unfixed ([tracker](https://security-tracker.debian.org/tracker/CVE-2026-66752)). Real impact on a loopback tool is low, but `cargo deny` and OSV scanners flag it, and an unmaintained parser is the wrong thing to put in front of a delete button. Forks exist ([tiny_http_dh](https://crates.io/crates/tiny_http_dh) 0.12.2, 2026-09-20, 78 downloads, one maintainer): too young to trust. Earlier RUSTSEC-2020-0031 (smuggling) was patched in 0.8.0.
- Measured in the table above, all heavier than axum for no gain here: rouille 3.6.2 (last release 2023-04-24, 84 crates, 191 CPU-s), astra 0.4.0 ("blocking" API but 18 crates including tokio and hyper), poem 3.1.12 (88 crates, 305 CPU-s, 3.6 MiB), actix-web 4.15.0 (103 crates, 294 CPU-s), salvo 1.0.1 (112 crates, 368 CPU-s), rocket 0.5.1 (116 crates, 529 CPU-s, 3.1 MiB, no release since 2024-05). Resolve only: ntex (4.0 still beta, 111 crates), trillium (100 crates with its smol runtime crate).
- miniserve (actix-web, `Repos/miniserve/Cargo.toml:23`) also gates writes: `--upload-files`, `--mkdir` are opt-in (`Repos/miniserve/README.md:313`).

### Live updates and the sync-to-async boundary

- Chosen: SSE (`EventSource`), server to client only. MDN: HTTP/1.1 allows 6 connections per browser and domain, auto-reconnect is built in, and `EventSource` cannot set custom headers ([MDN](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)). The no-custom-header rule is why auth is a cookie, not a bearer header. WebSocket adds `tungstenite` crates and needs its own Origin check (cross-site WebSocket hijacking is one of the three root causes in the [Vite advisory](https://github.com/vitejs/vite/security/advisories/GHSA-vg6x-rcgg-rjx6)); nothing here needs client-to-server streaming. Polling fallback with no JS at all: `<meta http-equiv=refresh content=2>` while a scan runs.
- Flow (prototype, verified): engine thread to `std::sync::mpsc` to one plain bridge thread that folds events into shared state and calls `broadcast::Sender::send` (synchronous, never blocks) to SSE streams that re-read the state on every tick. Because the stream sends the state, a lagged receiver loses nothing. `tokio::sync::watch` is the simpler primitive for "state changed" ticks (no `Lagged` case). The engine crate has zero dependencies and never sees tokio; only the web crate knows both sides.
- Gotcha, reproduced: axum's graceful shutdown waits for open connections, and an SSE stream never ends. With the stream not tied to the stop signal the server was still running 5 s after SIGINT; with `take_until(stop)` it exited in 4 ms with a client attached. axum's own example needs a `TimeoutLayer` for the same reason (`Repos/axum/examples/graceful-shutdown/src/main.rs:40-41`).
- Second gotcha, hit while compiling: middleware must not hold `&Request` across an `.await` (`Body` is not `Sync`), otherwise the opaque error is "`FromFn<...>: Service` is not satisfied". Extract header strings in a block before awaiting.
- htmx: 4.0.0 released 2026-08-28 (0BSD, `htmx.min.js` 52,182 B, 16,885 B gzipped); in 4.x SSE moved into a separate extension (10 KiB) and the release is a breaking major. Not needed: the prototype's live page is 15 lines of `EventSource`. Revisit only if forms and partial swaps multiply.

## Option C: JSON API plus embedded SPA

- Closest real tool in our domain: skillshare's dashboard is a React 19, Vite, Tailwind 4, CodeMirror SPA: 424 files under `ui/src`, 498 packages in `ui/pnpm-lock.yaml`, build with `make ui-build` (pnpm) (`Repos/skillshare/Makefile:152-160`). The build output is gitignored, and the binary does not embed it: at first use it downloads `skillshare-ui-dist.tar.gz` from GitHub releases into `~/.cache` and verifies a checksum (`Repos/skillshare/internal/uidist/uidist.go:15-40`). Even with `go:embed` available they chose a runtime download, which says what an embedded SPA does to `go install` or `cargo install`.
- Counter-example: dufs ships a file manager with upload and delete as three vanilla files (1,006 lines JS, 305 CSS, 131 HTML), embedded with `include_str!` (`Repos/dufs/src/server.rs:54-57`), no `build.rs`, no Node.
- Embedding crates (resolve only): rust-embed 8.13.0 (MIT, 21 crates, reads from disk in debug unless `debug-embed`), include_dir 0.7.4 (MIT, 5 crates, last release 2024-06-17), memory-serve 2.4.0 (78 crates, brings tokio), axum-embed 0.1.0 (last release 2023-12-17, 43 crates). For three to ten assets `include_str!` costs zero crates.
- `cargo install` and CI: a build script that runs npm forces Node on every installer. Committing `dist/` avoids that but puts minified bundles in diffs and needs a CI check that they match the sources; packaging `dist/` through `include` in `Cargo.toml` works for crates.io only, not for `cargo install --git`. Not measured: Node install and build time.
- Verdict: no. Nothing on the screens needs client-side routing, and a Node toolchain plus hundreds of npm packages in the contributor path is the cost the requirements forbid.

## Options D and E: Rust WASM full-stack and desktop shells

Crates from `cargo tree` (Linux host, normal and build deps, resolve only):

| Option | Version | Crates | Needs besides cargo | Verdict |
|---|---|---|---|---|
| Leptos | 0.8.22 (MIT) | 173 | wasm32 target, Trunk (CSR) or cargo-leptos (SSR) ([book](https://book.leptos.dev/getting_started/)) | no |
| Dioxus web / desktop | 0.7.10 | 146 / 352 | `dx` CLI, wasm32 target; desktop needs webkit2gtk on Linux ([docs](https://dioxuslabs.com/learn/0.7/getting_started/)) | no |
| Yew | 0.23.0 | 87 | wasm32 target, a bundler | no |
| Sycamore | 0.9.4 | 34 | wasm32 target, a bundler, MSRV 1.94 | no |
| Tauri | 2.12.1 | 261 | libwebkit2gtk-4.1, GTK, tray libs on Linux ([prereqs](https://v2.tauri.app/start/prerequisites/)) | no |
| wry + tao | 0.57.0 / 0.37.1 | 139 | webkit2gtk on Linux | no |
| eframe/egui | 0.36.2 | 262 | GPU/windowing stack, MSRV 1.95 | no |
| iced | 0.14.0 | 237 | GPU/windowing stack | no |
| Slint | 1.18.1 | 343 | licence: GPL-3.0-only OR royalty-free OR proprietary | no (licence) |

Reasons: WASM full-stack trades the Node toolchain for a second toolchain (target plus a CLI installed globally) and compiles the UI twice; the screens are tables and diffs, which HTML does natively and accessibly. Desktop shells depend on system webviews or a GPU stack, cannot be reached over SSH port forwarding (the README says SSH is a use case), add 139 to 343 crates, and the immediate-mode toolkits give up native text selection, find-in-page and screen-reader semantics. They also make the web view a second app instead of a third front-end of the same engine.

## Cross-cutting findings

### Async isolation (checked)

`cargo tree -p cli -i tokio` fails with "did not match any packages" for the base build and for `--features web-tiny`, and lists axum, hyper and tokio for `--features web-axum`. Pitfall (cargo reference: features unify across all packages built together, [resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)): `cargo tree --workspace -i tokio` finds tokio when the web crate is a workspace member, so `cargo build --workspace` and `cargo test --workspace` compile it. Fix tested: `default-members` in the workspace lists everything except the web crate, and CI runs one explicit `--features web` job. Ratatui 0.30.2 resolves to 72 crates and none is tokio, so TUI builds are clean. The engine owns the thread model (rayon or `ignore` threads, plain channels) and the web crate is a consumer; the engine never needs `async`.

### Browser launching, port, shutdown

- Linux only (decided in the requirements), so launching is `$BROWSER` or `xdg-open`: spawn it with `std::process::Command`, detached, stderr discarded, about 8 lines and zero crates. Launcher crates for reference (resolve only): `open` 5.4.4 (MIT, 5 crates), `webbrowser` 1.2.4 (34 crates, it pulls `url` and the ICU stack; has `$BROWSER` support and a `hardened` http(s)-only feature, [docs](https://docs.rs/webbrowser/latest/webbrowser/)), `opener` 0.9.0 (33 crates, used by mdBook). Windows and macOS launchers: not needed. Print the URL every time; open only with `--open` or when `$DISPLAY` or `$WAYLAND_DISPLAY` is set; over SSH print the `ssh -L` hint; never fail because no opener exists.
- Port: bind `127.0.0.1:0`, read the assigned port, then print. Random beats a fixed port: skillshare's fixed default 19420 (`Repos/skillshare/cmd/skillshare/ui.go:76-77`) is a known target for drive-by requests and for port squatting. `--port` pins it.
- Ctrl-C (design; the prototype implements the first press): the `web` command owns SIGINT. First press stops accepting, closes SSE streams (watch channel), lets the in-flight skill swap finish (the engine's swap is atomic per skill, so apply is cancellable only between skills), exits 0; second press exits immediately. Measured exit with an SSE client attached: 4 ms (axum), 105 ms (tiny_http). `tokio::signal::ctrl_c` inside the runtime for axum, `ctrlc` 3.5.2 (6 crates) otherwise.
- Tab close: `sendBeacon` is not usable as the only signal: it fires on `visibilitychange` to hidden (tab switch too), is always POST and cannot set headers ([MDN](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/sendBeacon)). Use the SSE connection count: when it drops to zero after having been above zero, start a grace timer (10 s in the prototype, 60 s recommended so reloads and laptop sleep survive), then shut down. Verified: server exited by itself 3 s after the last stream closed. Also an initial grace if no tab ever connects, and an explicit Quit button (`fetch` with `keepalive`). Jupyter ships the same idea off by default: `shutdown_no_activity_timeout = 0` (`Repos/jupyter_server/jupyter_server/serverapp.py:1970`).

### UI approach, accessibility, coherence with the TUI

- Hand-written CSS (prototype: 0.9 KiB; a full UI maybe 60 lines), custom properties, `color-scheme: light dark`, no framework. Pico classless is 71,040 B minified (MIT, last release 2025-03-15), larger than the whole static report; Open Props (MIT) is a variable set, not a layout. System font stack, no fonts, no icons: status is a word plus a glyph (`ok`, `old`, `mod`, `--`) in the same vocabulary and colours as the TUI, so the two front-ends read alike.
- WCAG 2.2 ([quick reference](https://www.w3.org/WAI/WCAG22/quickref/)) drives four rules: status never by colour alone (1.4.1), real `<table>` with `th scope` (1.3.1), focus visible and everything keyboard operable (2.4.7, 2.1.1), progress and results in an `aria-live=polite` region with `role=status` (4.1.3); text contrast 4.5:1 in both themes (1.4.3), which the prototype palette is meant to meet but was not audited.
- Matrix reality: 60 x 82 cells with 8 % filled does not fit a screen. Default view lists only pairs that need attention (outdated, modified, missing mandatory), grouped by project, with the full matrix one toggle away; skills as rows and projects as chips is the better narrow-screen layout.

### Maintenance burden

- Stage 0 adds one module (about 80 to 300 lines), a golden-file test of the HTML for a fixed `Snapshot`, and one docs page.
- Stage 1 adds a crate and a CI job with `--features web`; handler tests run without sockets (`guard` is pure, 4 unit tests passed; axum routers can be driven with `tower::ServiceExt::oneshot`, see axum's `examples/testing`); one smoke test with `ureq` or `curl` against a real port; a security test script (the 14 checks below) that must stay green; a docs page on the threat model and `--allow-write`. Dependency churn: axum 0.8 to 0.9 migration, htmx not used so none there, no npm.
- Stage 2 (writes) adds the largest surface: plan/confirm state, backup/undo integration, an audit log, and a rule that every new route is classified read or write.

## Measurements

Release profile, prototype `cli` with feature sets (KiB = 1024 B). Clean-build time is two samples, "first / second" (CPU-s, then wall s in brackets); the second ran after the guard hardening and audit logging (binary +0.6 KiB for tiny_http, +4 KiB for axum) on a machine with load 30 to 34. Time columns are noisy (see Method).

| Variant | Stripped size | Delta | External crates | Clean build CPU-s (wall s) | Rebuild: edit bin / edit web crate (wall s) | Idle RSS (threads) |
|---|---|---|---|---|---|---|
| base: clap derive + trivial engine | 855 KiB | - | 17 | 49.8 / 59.5 (14.1 / 37.3) | 1.4 / - | 1.2 MiB (1) |
| + static report | 892 KiB | +37 KiB | 17 | 48.3 / 59.0 (10.1 / 24.6) | 1.1 / 1.4 | 1.2 MiB (1) |
| + web, tiny_http | 1,330 KiB | +475 KiB | 29 (+12) | 84.0 / 93.0 (13.2 / 24.1) | 1.4 / 1.7 | 3.4 MiB (7), 3.6 MiB with page and SSE |
| + web, axum minimal features | 1,804 KiB | +949 KiB | 56 (+39) | 158.1 / 184.6 (30.7 / 69.6) | 1.7 / 11.3 (9.4) | 1.4 MiB (1), 3.6 MiB with page and SSE |
| + web, axum default features, tokio full | 2,008 KiB | +1,153 KiB | 71 (+54) | 211.9 (55.4) | 4.7 / 18.0 | 1.6 MiB (1), 4.1 MiB with page and SSE |

`dist` profile (fat LTO, one codegen unit, panic abort): base 620 KiB, static +24 KiB, tiny_http +302 KiB, axum minimal +563 KiB, axum full +651 KiB. Clean-build CPU-s in that profile were 48, 38, 92, 158, 164 but with wall times of 37 to 157 s under load they are not usable beyond "the same order as release".

Reading: axum and tokio default features add 15 crates, 204 KiB and about 54 CPU-s on top of the minimal build (web delta +38 % crates, +21 % size), so always set `default-features = false` on axum and tokio and use the current-thread runtime (`new_current_thread`: 1 thread, no `rt-multi-thread`). RSS is irrelevant at this scale; the engine, not the server, dominates memory.

Templating benchmark: the same 60 x 82 matrix table, release profile, one binary per crate (numbers are for the whole binary including `core_sim`).

| Approach | Crates | Clean build CPU-s | Stripped size | Delta over std | Rebuild after edit (wall s) | Escaping |
|---|---|---|---|---|---|---|
| plain string building (std) | 0 | 3.1 | 354 KiB | - | 1.859 | manual `esc()` |
| maud 0.27.0 | 9 | 19.2 | 353 KiB | 0 KiB | 1.111 | automatic, compile time |
| askama 0.16.1 (default features) | 18 | 47.2 | 356 KiB | +2 KiB | 1.154 | automatic, compile time |
| minijinja 2.24.0 + serde derive | 9 | 89.9 | 1,830 KiB | +1,476 KiB | 1.190 | automatic for `.html` names, runtime |
| tera 2.4.0 + serde derive | 8 | 92.4 | 1,520 KiB | +1,166 KiB | 3.131 | automatic, runtime |

### Crate facts (crates.io API and `cargo tree`, 2026-10-07)

Downloads are the 90-day figure. Crates are unique external packages on the Linux host target (resolve only unless stated). Health is a one-phrase reading of release cadence and the repository.

| Crate | Version | Last release | Downloads | Crates | Health |
|---|---|---|---|---|---|
| axum | 0.8.9 | 2026-04-14 | 128 M | 38 (54 default) | tokio-rs, 66 commits and 27 authors in 90 days; 0.9 pending |
| hyper | 1.12.0 | 2026-10-06 | 224 M | 19 (with util) | release every 3 to 6 weeks, 48 commits in 90 days |
| tokio | 1.53.2 | 2026-10-03 | 248 M | n/a | very active |
| tower-http | 0.7.1 | 2026-08-31 | 167 M | +7 | active, 40 open issues |
| tiny_http | 0.12.0 | 2022-10-06 | 15 M | 5 | master 2023-05-16, 76 open issues, 2 open CVEs |
| rouille | 3.6.2 | 2023-04-24 | 4.4 M | 84 | quiet since 2023 |
| astra | 0.4.0 | 2024-11-07 | 1.7 k | 18 (tokio) | 1.7 k downloads, last release 2024-11 |
| actix-web | 4.15.0 | 2026-08-21 | 11 M | 103 (147) | active, heavy |
| poem | 3.1.12 | 2025-07-28 | 0.8 M | 86 | 14 months since last release |
| salvo | 1.0.1 | 2026-10-04 | 0.8 M | 110 (182) | 1.0.1 released three days ago |
| rocket | 0.5.1 | 2024-05-23 | 2.2 M | 116 | 17 months without release |
| ntex | 3.12.3 (4.0.0-beta.18) | 2026-10-07 | 0.1 M | 111 | 4.0 still beta |
| maud | 0.27.0 | 2025-02-02 | 2.5 M | 9 | stable, slow cadence |
| askama | 0.16.1 | 2026-09-04 | 13.5 M | 18 (2 without default features) | active; the rinja fork is marked deprecated on crates.io |
| minijinja | 2.24.0 | 2026-10-04 | 12.1 M | 4 | active; 3.0.0-alpha.3 out |
| tera | 2.4.0 | 2026-09-11 | 6.3 M | 3 | 2.x rewrite, active |
| rust-embed | 8.13.0 | 2026-10-07 | 17.1 M | 21 | active |
| include_dir | 0.7.4 | 2024-06-17 | 18.1 M | 5 | quiet |
| memory-serve | 2.4.0 | 2026-09-17 | 24 k | 78 (tokio) | small user base |
| axum-embed | 0.1.0 | 2023-12-17 | 0.1 M | 43 | stale |
| ctrlc | 3.5.2 | 2026-02-10 | 23 M | 6 | stable |
| signal-hook | 0.4.5 | 2026-10-04 | 59 M | 4 | active |
| getrandom | 0.4.3 | 2026-06-17 | 625 M | 3 | rust-random, standard |
| pulldown-cmark | 0.13.4 | 2026-05-20 | 53 M | 4 | active |
| comrak | 0.56.0 | 2026-10-06 | 2.6 M | 22 | active |
| similar | 3.2.0 | 2026-08-17 | 56 M | 1 | active |

## Security model

### Threat model

The tool can delete files, the browser is the attack path. Attackers: (1) any web page the user has open (drive-by requests to `127.0.0.1:PORT`, including port scanning), (2) DNS rebinding (attacker domain re-resolved to 127.0.0.1), (3) other pages served from `127.0.0.1` on a different port, (4) other local users on a shared machine (out of scope for the minimum, noted under residual risks), (5) hostile content inside a skill folder (XSS through `SKILL.md`, names, paths). Local processes of the same user can already delete the files directly, so they are not a boundary.

### What real tools do (read, not recalled)

| Tool | What it does | Source |
|---|---|---|
| Vite dev server | CVE-2025-24010: CORS `*`, WebSocket without Origin check, no Host check; any website could read source. Fix: `allowedHosts`, restricted CORS | [GHSA-vg6x-rcgg-rjx6](https://github.com/vitejs/vite/security/advisories/GHSA-vg6x-rcgg-rjx6) |
| MCP Inspector | CVE-2025-49596, CVSS 9.4: no auth between client and proxy, remote command execution from a browser; fixed in 0.14.1 with authentication | [GHSA-7f8r-222p-6f5g](https://github.com/modelcontextprotocol/inspector/security/advisories/GHSA-7f8r-222p-6f5g) |
| Jupyter Server | token from `os.urandom(24)` (`Repos/jupyter_server/jupyter_server/auth/identity.py:224`), exchanged for an HttpOnly cookie named per `host:port` (`identity.py:384`; cookies ignore ports), `check_host` allows loopback IPs and `localhost` only (`Repos/jupyter_server/jupyter_server/base/handlers.py:556`), `check_origin` and `_xsrf` for cookie-authenticated calls (`Repos/jupyter_server/jupyter_server/base/handlers.py:437`, `Repos/jupyter_server/jupyter_server/base/handlers.py:530`); its comment admits missing Origin is let through | source |
| Syncthing GUI | when bound to localhost it enforces that the `Host` header looks like localhost (opt-out `insecureSkipHostcheck`); default `127.0.0.1:8384`; API key header | [config docs](https://docs.syncthing.net/users/config.html), [REST docs](https://docs.syncthing.net/dev/rest.html) |
| skillshare dashboard (same domain) | no token; Origin, Fetch-Metadata and Host checks only on the MCP, hooks and plugin routes (`Repos/skillshare/internal/server/handler_restart_upgrade.go:163-190`); `DELETE /api/resources/{name}` is registered bare (`Repos/skillshare/internal/server/server.go:479`); fixed port 19420. I found no global middleware doing it (`middleware.go` is logging only) | source |
| mdBook serve | `localhost` default host, no Host or Origin check, read-only content | `Repos/mdBook/src/cmd/serve.rs` |
| dufs | read-only by default, writes behind `--allow-upload`, `--allow-delete`, `--allow-all`; optional account auth | `Repos/dufs/README.md:64-70` |
| miniserve | writes opt-in (`--upload-files`), optional `--auth`, but listens on `::` and `0.0.0.0` unless `--interfaces` is given | `Repos/miniserve/src/config.rs:211-216` |
| Tauri 2 | no listening port by default; capabilities and an isolation pattern gate IPC commands, CSP for the webview; a trust boundary instead of a network one | [security docs](https://v2.tauri.app/security/) |

OWASP's CSRF cheat sheet ([source](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)): reject non-safe methods when `Sec-Fetch-Site: cross-site`, fall back to Origin verification, treat "Missing headers and `Origin: null`" as no evidence of same-origin, custom headers force a preflight, `SameSite` is defence in depth only, never mutate on GET. Chrome's Local Network Access prompt (default from Chrome 142) gates public-site to loopback `fetch`, does not yet cover WebSockets, and leaves Host and Origin validation to the server ([Chrome](https://developer.chrome.com/blog/local-network-access)). MCP SDKs now ship a port-agnostic Host allow-list for `localhost`, `127.0.0.1`, `[::1]` ([docs](https://ts.sdk.modelcontextprotocol.io/v2/api/@modelcontextprotocol/node/middleware/hostHeaderValidation.html)).

### Minimum acceptable design

1. Bind `127.0.0.1` only, port 0, print the URL after binding. Never `0.0.0.0`; no `--host` flag in stage 1.
2. Host allow-list, exact match on `127.0.0.1:PORT`, `localhost:PORT`, `[::1]:PORT`; otherwise 421. Stops DNS rebinding, and is the only layer that does if auth were absent (Syncthing and Vite show why).
3. Launch token: 256 random bits from the OS (`getrandom` 0.4.3, 3 crates), printed in the URL `?token=`, single use. `GET /?token=` swaps it for a session cookie `sm_<port>=<random>; HttpOnly; SameSite=Strict; Path=/` and redirects to `/`, so the token is gone from the address bar (checked: URL is `/` after load, `document.cookie` is empty). Constant-time compare. Cookie name carries the port because cookies ignore ports (Jupyter does the same).
4. Fetch Metadata and Origin on every request after the cookie check: `Sec-Fetch-Site` must be `same-origin` or `none` when present, except for a top-level `GET` with `Sec-Fetch-Mode: navigate` (safe, response unreadable cross-origin); `Origin`, when present, must equal ours. The exception is needed: Firefox 142.0.1 labels a URL passed on the command line (what `xdg-open` does) `cross-site`, and the first version of the prototype, which rejected every cross-site request, answered its first page load with 403 (observed in the log); Chromium 139 sends `none`. This layer is load-bearing, not extra: measured in Chrome, a page on `http://127.0.0.1:8099` (same site as our `127.0.0.1:41909`, because SameSite ignores ports) sent the session cookie, and only this check returned 403 (log: `deny 403 (cross-site) POST /api/sync`).
5. Mutations: POST only, never GET; `Origin` present and equal (browsers always send it on POST, so absence means not a tab); a custom header (`HX-Request` or `X-Requested-With`, forces a preflight that no route answers); refused with 403 unless `--allow-write` was given.
6. No CORS headers anywhere, `Cross-Origin-Resource-Policy: same-origin`, `Cross-Origin-Opener-Policy: same-origin`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`, and CSP `default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'self'; frame-ancestors 'none'` (no inline script or style, so reflected markup cannot run).
7. The client never sends a path. Routes take ids the server issued from its own scan (`/skills/coding`, project index). Apply is two steps: `POST /api/plan` returns a server-computed plan with an id, and `POST /api/apply/<id>` runs exactly that plan once, expires after minutes, and refuses if the scan changed. Backup and `undo` apply as in the CLI.
8. Audit log on stderr for every deny and every executed mutation (used above as test evidence).
9. Auto-escaping templates; `SKILL.md` as text; no `PreEscaped` on disk-derived data.
10. Idle shutdown and an explicit Quit; session cookie without `Max-Age`.

tower-http is not needed for this: the header and admission layer is 12 lines of `from_fn` middleware over the pure `guard` module. Adding `tower-http` 0.7.1 costs 7 more crates.

### Verification (14 checks, both prototypes, plus real browsers)

curl, axum build then tiny_http build, all as expected: no cookie 401; wrong Host 421; wrong token 401; good token 303 to `/`; token reuse 401; GET with cookie 200; cross-site Sec-Fetch-Site 403; POST without header 403; POST with foreign Origin 403; POST in default mode 403 "read-only"; OPTIONS preflight returns no `access-control-allow-*`; security headers present; SSE streams events; server exits by itself after the idle grace; SIGINT with a client attached exits. Final build, headless Firefox 142.0.1 and Chromium 139: token exchange, redirect, CSS, JS and the SSE stream all allowed (request log shows `sec-fetch-site` `cross-site`/`none` only on the launch navigation, `same-origin` on every sub-request). Chrome (pane, before the navigation exception): same-origin POST 200 with header and 403 without; from `localhost:8099` (cross-site) preflight 401 and all five attempts failed (cookie not sent, SameSite=Strict), and the no-cors POST still reached the server (401 in the log), which is why the cookie and Fetch-Metadata layers matter more than CORS; from `127.0.0.1:8099` (same site) cookie sent, 403 by Fetch Metadata, no `AUDIT sync executed` line from any attack.

### Residual risks

- The token travels in the launcher's argument vector (`xdg-open URL`), visible in `ps` for a moment; single use limits the damage, but a shared multi-user machine can race it. Hardening if wanted: open a mode-0600 temp HTML file that redirects. Loopback TCP is not isolated per user.
- Browsers differ: Fetch Metadata is baseline since March 2023 and worked in Chromium 139 and Firefox 142.0.1 (headless, launch URL on the command line); other Linux browsers (Brave, WebKitGTK based ones) were not tested, and the launch navigation may be labelled `none` or `cross-site` depending on the browser.
- `localhost` versus `127.0.0.1` have separate cookie jars; users typing the other name get a 401, by design.
- Local Network Access and Private Network Access rules are moving targets, they are a bonus layer only.

## UI wireframes (ASCII)

Overview matrix (default: attention view):

```
+-------------------------------------------------------------------------------+
| <tool> web   vault ~/skills (clean, 9f2c1a)   root ~/code      [read-only]    |
| scan: 60 projects, 82 skills, 395 installs [##########] done 3.1 s [rescan]   |
+-------------------------------------------------------------------------------+
| Filter [ coding____ ]  Show (*) attention ( ) all ( ) modified ( ) missing    |
| Group [ all v ]                                   [ show full matrix ]        |
+--------------------+---------+---------+---------+---------+---------+--------+
| project            | coding  | doc-st  | astro   | vite    | secrets | ...    |
+--------------------+---------+---------+---------+---------+---------+--------+
| ~/code/site-a      | ok      | old     | ok      | --      | mod     |        |
| ~/code/api         | old     | ok      | --      | --      | --      |        |
+--------------------+---------+---------+---------+---------+---------+--------+
 ok up to date | old outdated (vault newer) | mod locally modified | -- absent
 [ Copy: <tool> sync --all ]   [ Sync 12 outdated ... ] (needs --allow-write)
```

Skill detail with diff:

```
< matrix   coding   group core/   vault 9f2c1a   14 projects: ok 9 old 3 mod 2
[ SKILL.md ] [ Files ] [ Diff vs vault ] [ Projects ]
--------------------------------------------------------------------------------
~/code/site-a/.agents/skills/coding     modified (local edit, vault unchanged)
--- vault/core/coding/SKILL.md
+++ site-a/.agents/skills/coding/SKILL.md
@@ -12,6 +12,7 @@
  Keep functions short.
- Max 300 lines per file.
+ Max 400 lines per file.            <- local edit
--------------------------------------------------------------------------------
 [ Copy: <tool> diff coding --project site-a ]  [ Overwrite from vault ... ]
```

Group browser:

```
Groups                       | web/seo   4 skills   installed in 9 projects
-----------------------------+------------------------------------------------
v core        (3)            |  core-web-vitals   7 projects   ok 6  old 1
    coding  doc-start  ...   |  schema-markup     3 projects   ok 3
v web         (5)            |  sitemap           2 projects   ok 2
  > seo       (4)            |  ...
    astro  vite  ...         |  [ Copy: <tool> add --group web/seo ]
> android     (2)            |
```

Apply confirmation (write mode only):

```
Confirm sync                                     plan 7f3a   valid for 4:52
This plan changes 12 skills in 5 projects: 9 files modified, 3 added, 1 removed.
  ~/code/site-a   doc-start   2 files modified    [diff]
  ~/code/api      coding      1 file modified     [diff]
  ...
Backup: ~/.local/state/<tool>/backups/2026-10-07T16-40   undo: <tool> undo
[x] 2 skills have local edits that this plan overwrites: site-a/secrets, api/vite
[ Cancel ]                                       [ Apply plan 7f3a ]
progress: [##########] 12 / 12 ok     (live through SSE, aria-live)
```

## Recommendation and staged path

| Stage | Scope | Gate | Cost |
|---|---|---|---|
| 0 | `<tool> report [--out f.html]`: static report in the `report` crate (renderer takes the engine's `Snapshot`) | ships with the first Rust release | prototype +37 KiB with std strings; maud +9 crates, +0 KiB; no feature flag |
| 1 | `<tool> web`: read-only live view, SSE progress, GET-form filters, plan preview, no mutations route compiled in | TUI done; real demand beyond the report; threat-model test script green | feature `web`, axum minimal, +949 KiB, +39 crates |
| 2 | `--allow-write`: plan/confirm/apply, audit log, undo | drift guard, backup and `undo` exist; stage 1 used for a month | no new crates |
| never | SPA, WASM, desktop shell, WebSocket, remote bind | | |

Workspace shape: `engine` (sync, no async), `report` (pure renderers: `Snapshot` to HTML, diff to HTML), `web` (axum, `guard`, SSE bridge; depends on `report`), `cli` (clap, `tui`, optional `web`). `cli/Cargo.toml`: `web = ["dep:web"]`, off by default, on in release artifacts (cargo-dist `features`), `default-members` excludes `web`, CI has a `--features web` job and the verification script. Flip the default to on only if stage 1 earns it. A build without the feature must print a one-line error for `<tool> web` ("built without the web feature") and exit non-zero (hard errors, no fallback).

## Good but licence-blocked

Project licence is Hippocratic License 3.0 (non-OSI). Checked SPDX strings with `cargo metadata` over every tree above (distinct strings: MIT, Apache-2.0, MIT OR Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unlicense, Unicode-3.0, Unicode-DFS-2016, CC0-1.0, BSL-1.0 (Boost, not Business Source), CDLA-Permissive-2.0, MPL-2.0) and with `gh api` for JS, CSS, font and icon projects.

- Slint 1.18.1: `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` (13 crates in the tree). Blocked: GPL, and the royalty-free terms are a custom licence needing review. Otherwise a strong desktop toolkit.
- `self_cell` 1.3.0 (via eframe, iced): `Apache-2.0 OR GPL-2.0-only`: take the Apache arm, no block.
- MPL-2.0, allowed but marked: `option-ext` 0.2.0 (tauri, wry, dioxus-desktop trees), `cssparser`, `cssparser-macros`, `selectors`, `dtoa-short` (tauri tree). Also mdBook and Syncthing are MPL-2.0, used here as references only, not linked.
- System libraries: Tauri and wry link webkit2gtk (LGPL) dynamically on Linux from the OS package; not bundled, but static LGPL linking would count.
- Fonts and icons (none embedded in the recommended design, system font stack and text glyphs): Inter and JetBrains Mono are `OFL-1.1` (embedding allowed, reserved font name and licence-text obligations, not on the permissive list); Font Awesome Free (GitHub reports NOASSERTION; `LICENSE.txt` splits it) has icons under `CC-BY-4.0` (attribution), font files under OFL-1.1; Nerd Fonts is OFL-1.1 plus mixed. `epaint_default_fonts` (egui) is `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`.
- Embedded web assets we would plausibly add, all verified: htmx `0BSD` (GitHub reports NOASSERTION; the LICENSE file is Zero-Clause BSD, no notice required), Pico CSS `MIT`, Open Props `MIT`, Datastar `MIT`, Preact `MIT`, Alpine `MIT`, highlight.js `BSD-3-Clause`, Lucide `ISC`, Feather `MIT`, Heroicons `MIT`, Bootstrap Icons `MIT`, Material Design Icons `Apache-2.0`. Obligation for MIT, ISC, BSD and Apache: keep the copyright and licence text with the binary distribution, so a generated third-party notice (also for the Rust crates, which the CLI stream covers with cargo-about or cargo-deny) must include any vendored JS or CSS, and the web view should serve it at `/licenses`. In the recommended stage 0 and 1 no third-party JS or CSS ships at all.
- GPL, AGPL, LGPL, SSPL, BUSL, source-available or non-commercial dependencies in the recommended path (axum, hyper, tokio, getrandom, open, ctrlc, maud or askama, similar): none found.

## Rejected and why

- tiny_http (open CVEs, unmaintained), rouille, astra, poem, salvo, actix-web, rocket, ntex, trillium: weight or health, numbers above.
- SPA with embedded build (Svelte, Vue, Solid, Preact, React): Node in the contributor path, runtime download or committed bundles, no screen needs it.
- WASM full-stack (Leptos, Dioxus, Yew, Sycamore) and desktop shells (Tauri, wry/tao, egui/eframe, iced, Slint, Dioxus desktop): crates, extra CLIs, system libraries, no SSH, Slint licence.
- WebSocket: no client-to-server stream, adds Origin attack surface.
- memory-serve, axum-embed, rust-embed for stage 1: for under ten assets `include_str!` is free; memory-serve even adds tokio.
- Launcher crates (`open`, `webbrowser`, `opener`): not needed on Linux, `xdg-open` through `std::process::Command` is enough.
- htmx 4.0.0: not needed yet, major release one month old.
- Pico CSS: 71 KB of minified CSS for a page that needs 60 lines.

## Risks and open points

- The numbers: rerun clean-build timings on an idle machine before quoting seconds; sizes and crate counts are exact.
- The prototype data is synthetic; real SKILL.md text and diffs can double the page. The attention view and `<details>` keep it bounded.
- axum 0.9 is on the horizon; the migration is a day, the `guard` and `report` crates do not change.
- Other Linux browsers and interactive (non-headless) Firefox and Chrome were not driven through a real `xdg-open` handover; headless command-line launches were.
- Decide whether `web` is on by default; the default only changes who pays 100 CPU-s per clean build.
- Hippocratic License 3.0 is non-OSI: confirm that permissive dependencies combined with HL3 code are acceptable for the cargo-deny `allow` list (owned by the CLI stream).

Suggested changes to the requirements doc: replace "must not force an async runtime" by "async runtime only inside the optional `web` crate, `default-members` excludes it"; add "web view is read-only unless `--allow-write`; loopback only, no remote bind flag"; add a requirement that the engine exposes a `Snapshot` plus typed events (project found, plan ready, target applied, error, done) through std channels with no async types; add that renderers (`Snapshot` to HTML) live in a crate shared by the static report and the server; browser launching is `$BROWSER` or `xdg-open` through std (Linux only), no launcher crate.

## Clones

Made by this stream under `Repos/`: `tiny-http` (tiny-http/tiny-http, master 2023-05-16), `axum` (tokio-rs/axum, 2026-10-07), `jupyter_server` (jupyter-server/jupyter_server, 2026-09-15), `mdBook` (rust-lang/mdBook, 2026-10-05), `dufs` (sigoden/dufs, 2026-06-29), `miniserve` (svenstaro/miniserve, 2026-10-01). Reused from another stream: `skillshare` (runkids/skillshare). Prototypes, scripts and raw CSV live in the session scratchpad, not in the repo.

## Verdicts

| Crate or option | Verdict | Reason | Version checked | Licence |
|---|---|---|---|---|
| Option A: static HTML report | yes | stage 0, no server, no crate, +37 KiB, 41 KiB output | n/a | ours |
| Option B: loopback server, server-rendered | yes (stage 1, optional) | read-only live view behind feature `web`; decide after the TUI | n/a | ours |
| Option C: SPA with embedded build | no | Node in contributor path, 498 npm packages in the nearest tool | n/a | n/a |
| Options D and E (WASM, desktop shells) | no | 34 to 352 crates, extra CLIs, system libraries, no SSH | n/a | n/a |
| axum | yes | minimal features: +39 crates, +719 KiB hello; maintained; mdBook precedent | 0.8.9 | MIT |
| hyper (direct) | maybe | half of axum's cost, but hand-written routing; fallback | 1.12.0 | MIT |
| tokio (web crate only, current-thread, minimal features) | yes | isolated by feature, never in CLI or TUI builds | 1.53.2 | MIT |
| tokio-stream, futures-util | yes | SSE glue in the web crate (or `tokio::sync::watch` alone) | 0.1.19, 0.3.34 | MIT, MIT OR Apache-2.0 |
| tower-http | no | not needed, 12 lines of `from_fn`; +7 crates | 0.7.1 | MIT |
| tiny_http | no | two open CVEs since 2026-07-29, unmaintained since 2023 | 0.12.0 | MIT OR Apache-2.0 |
| tiny_http_dh, tiny_http_fork | no | forks too young (78 and 319 downloads) | 0.12.2, 0.12.17 | see crates.io |
| rouille | no | 84 crates, last release 2023 | 3.6.2 | MIT/Apache-2.0 |
| astra | no | "blocking" API but pulls tokio and hyper | 0.4.0 | MIT |
| actix-web | no | 103 crates, 294 CPU-s hello | 4.15.0 | MIT OR Apache-2.0 |
| poem | no | 88 crates, 3.6 MiB hello | 3.1.12 | MIT OR Apache-2.0 |
| salvo | no | 112 crates, 368 CPU-s hello | 1.0.1 | Apache-2.0 |
| rocket | no | 116 crates, 529 CPU-s hello, no release since 2024-05 | 0.5.1 | MIT OR Apache-2.0 |
| ntex, trillium | no | 111 and 100 crates, ntex 4.0 beta | 3.12.3, 1.4.0 | MIT OR Apache-2.0 |
| maud | yes | auto-escaping, compile-time checked, 9 crates, +0 KiB, +16 CPU-s | 0.27.0 | MIT OR Apache-2.0 |
| askama | maybe | second choice: real template files, 18 crates, +44 CPU-s | 0.16.1 | MIT OR Apache-2.0 |
| minijinja | no | +1.4 MiB binary, +87 CPU-s, errors only at run time, needs serde | 2.24.0 | Apache-2.0 |
| tera | no | +1.1 MiB binary, +89 CPU-s, errors only at run time, needs serde | 2.4.0 | MIT |
| handlebars, sailfish, markup, ructe, hypertext | no | 31, 24, 6, 7 (build script), 9 crates; no gain over maud or minijinja | 6.4.4, 0.11.8, 0.16.0, 0.18.2, 0.12.1 | MIT, MIT, MIT/Apache-2.0, MIT OR Apache-2.0, MIT |
| rust-embed | no (maybe later) | 21 crates; `include_str!` suffices for under ten assets | 8.13.0 | MIT |
| include_dir | maybe (later) | 5 crates, if the asset tree grows | 0.7.4 | MIT |
| memory-serve, axum-embed | no | 78 crates with tokio; stale since 2023 | 2.4.0, 0.1.0 | Apache-2.0 OR MIT, MIT |
| open, webbrowser, opener | no | Linux only: `xdg-open` through std; 5, 34, 33 crates | 5.4.4, 1.2.4, 0.9.0 | MIT, MIT OR Apache-2.0, MIT OR Apache-2.0 |
| ctrlc, signal-hook | maybe | only if the CLI stream wants one SIGINT handler; web crate uses `tokio::signal` | 3.5.2, 0.4.5 | MIT/Apache-2.0, MIT OR Apache-2.0 |
| getrandom | yes | 256-bit launch token, 3 crates | 0.4.3 | MIT OR Apache-2.0 |
| similar | yes | diff rendering (engine stream owns the choice), 1 crate | 3.2.0 | Apache-2.0 |
| pulldown-cmark | maybe (later) | `SKILL.md` rendering if wanted; convert raw-HTML events to text | 0.13.4 | MIT |
| comrak | no | 22 crates; pulldown-cmark is enough | 0.56.0 | BSD-2-Clause |
| tungstenite, tokio-tungstenite | no | WebSocket not needed, adds Origin surface | 0.30.0 | MIT OR Apache-2.0, MIT |
| htmx | no (not yet) | 4.0.0 is a one-month-old breaking major; 15 lines of JS suffice | 4.0.0 | 0BSD |
| Pico CSS, Open Props | no | 71 KB minified for a 60-line need; Open Props is variables only | 2.1.1, not checked | MIT, MIT |
| Datastar, Alpine, Preact | no | not needed | 1.0.4, 3.17.4, 11.0.0 | MIT |
| Leptos, Dioxus, Yew, Sycamore | no | second toolchain, 34 to 352 crates | 0.8.22, 0.7.10, 0.23.0, 0.9.4 | MIT, MIT OR Apache-2.0, MIT OR Apache-2.0, MIT |
| Tauri, wry + tao | no | 139 to 261 crates, webkit2gtk, no SSH | 2.12.1, 0.57.0 + 0.37.1 | Apache-2.0 OR MIT |
| egui/eframe, iced | no | 237 to 262 crates, GPU stack | 0.36.2, 0.14.0 | MIT OR Apache-2.0, MIT |
| Slint | no | GPL-3.0-only OR royalty-free OR proprietary | 1.18.1 | GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0 |
