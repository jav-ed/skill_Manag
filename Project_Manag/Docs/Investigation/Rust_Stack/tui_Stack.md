# Rust stack: TUI

Decision: build the TUI on ratatui 0.30.2 with the termina backend (`ratatui-termina` 0.1.0 over `termina`), not the default crossterm backend, because crossterm 0.29 (and its main branch as of 2026-10-06) stops delivering input after a burst of more than about 1 KB, and I reproduced that in a 40-line crossterm-only program and in the full prototype (500 mouse-motion events stall it, 20,000 events pass on termina). Keep the app independent of the backend with an own `Input` type (about 105 lines), so crossterm stays a 90-line drop-in if termina disappoints. Use no framework (tui-realm, rat-salsa and cursive are rejected): one `App` struct in the Elm style, a `Screen` enum as router, `std::thread` plus one `std::sync::mpsc` channel (no tokio), a 51-line `HitMap` of rectangles recorded while drawing for hover, click and wheel, and per-frame virtual lists with ratatui's own `Scrollbar` for the 400+ row lists. Take only small crates on top: `tui-input` (state only, backend features off), `nucleo-matcher` (fuzzy filter, MPL-2.0), `tui-tree-widget` (group tree), `terminal-colorsaurus` (light or dark), and for tests `insta`, `portable-pty` and `vt100`. Write the rest ourselves (spinner, key-binding table with help, directory picker, OSC 8 link, overlays), because every existing widget I read either lacks mouse support, throws away the scroll state, or costs more than it saves. All of this was checked by a working prototype (1,347 lines, 25 tests, same screens rendered byte-identical on both backends), not only by reading docs.

## Method and caveats

- Date 2026-10-07, Manjaro, rustc 1.98.1, 12 cores shared with five other research agents, so wall times below are noisy. Versions, dates, downloads and licences come from the crates.io API and `gh api`. Dependency counts are unique `name version` pairs from `cargo tree -e normal,build` beyond ratatui's default tree (72 packages, all permissive), probe crate excluded.
- Prototype: `.../scratchpad/tui/proto` inside the session scratchpad (not in the repo, so copy it if it is wanted): `src/hit.rs`, `input.rs`, `binding.rs`, `app.rs`, `picker.rs`, `backend/{crossterm_be,termina_be}.rs`, `ui/*`, plus `tests/{snapshot,mouse,picker,draw_cost,pty}.rs`. Side probes are in `.../scratchpad/tui/probe` (`evcount`, `evcount_termina`, `theme_probe`). Cargo feature `crossterm` or `termina` selects the backend. Update 2026-10-08: the prototype and the probes are now archived in [Prototypes](../Prototypes/linker_Prototypes.md).
- Platform: Linux only (see [rewrite_Requirements.md](rewrite_Requirements.md): Platforms). Windows key-event kinds, ConPTY and macOS terminals are not needed and not examined. Release events can still arrive on Linux when a terminal speaks the Kitty keyboard protocol (we do not enable it), so one line in `Binding::matches` drops them.
- Not verified, stated once here: real terminal emulators (kitty, foot, alacritty, tmux, ssh) were only emulated through `vt100`; the termina panic-hook path was written but a panic restore was not exercised; `tuirealm`, `rat-salsa`, `crokey`, `frizbee`, `tachyonfx`, `hyperrat` and `ratatui-interact` were judged from source and metadata, not run.

## Options considered

### Core: ratatui and its split

- ratatui 0.30.2, released 2026-06-19, MIT, 57.6M downloads (20.7M in the last 90 days), MSRV 1.88. Releases: 0.30.0 on 2025-12-26, 0.30.1 on 2026-06-05, 0.30.2 on 2026-06-19. 205 commits since 2026-04-07, 22.9k stars, 202 open issues (`gh api`).
- Since 0.30 the workspace is split ([ARCHITECTURE.md](../../../../Repos/ratatui/ARCHITECTURE.md)): `ratatui-core` 0.1.2 (traits, buffer, layout, text), `ratatui-widgets` 0.3.2, backends `ratatui-crossterm` 0.1.2, `ratatui-termion`, `ratatui-termwiz`, and `ratatui-termina` 0.1.0 (listed at `Repos/ratatui/ARCHITECTURE.md:68`). Apps should depend on `ratatui`; widget authors on `ratatui-core`. yazi already builds on `ratatui-core` plus `ratatui-widgets` with its own terminal layer (`Repos/yazi/yazi-tui/src/raterm.rs:11`) and patches `ratatui-core` for a wide-cell diff bug (`Repos/yazi/Cargo.toml:115`); ratatui 0.30.2 fixed the same class of bug ("uncovered cells", `Repos/ratatui/CHANGELOG.md` 0.30.2 section).
- Pitfall found: with `default-features = false`, linking fails (`_critical_section_1_0_acquire` undefined) unless the `std` feature is on. Use `features = ["std", "underline-color", "layout-cache", "macros", "all-widgets"]` when choosing the backend yourself.
- Cost: `ratatui` default tree is 72 packages. Hello world, release with lto and opt-level s: 33.7 s clean, 472 KB.
- Verdict: yes.

### Backend: crossterm versus termina (the key finding)

- crossterm 0.29.0, MIT, last release 2025-04-05 (18 months), 204.5M downloads, 4.2k stars, 29 commits since April, 256 open issues. It is ratatui's default and `EnableMouseCapture` already turns on any-motion tracking (`?1003h`, `crossterm-0.29.0/src/event.rs:321`), so hover works.
- Bug, reproduced. The Unix reader (`crossterm-0.29.0/src/event/source/unix/mio.rs:22` and `:118`) reads 1,024 bytes, returns the first parsed event, and leaves the rest of the tty buffer unread while mio is edge-triggered, so no new wake-up arrives until more input does. Upstream knows: [#1057](https://github.com/crossterm-rs/crossterm/pull/1057) (open since 2026-05-12, "drain tty fd to EAGAIN"), [#1126](https://github.com/crossterm-rs/crossterm/issues/1126) and [#1128](https://github.com/crossterm-rs/crossterm/pull/1128), none merged. `diff` of `mio.rs` against main (2026-10-06) shows no logic change.
- My reproduction (`probe/evcount`, 25 lines, plus a `portable-pty` harness writing SGR mouse-motion reports in one write): 100 events (1.2 KB) pass, 200 events (2.3 KB) stall, and the later `q` key is never read either. Writing in chunks of 100 to 1,024 bytes with a 2 to 5 ms pause passes. In the full prototype on crossterm: 100 events pass; 500, 3,000 and 20,000 stall (the writer blocks because the child stopped reading).
- Why it matters here: the tool is meant to work over SSH, where TCP batches many motion events per segment, and users paste paths into the setup wizard. The symptom is a hover or click that is applied late or only after the next input, not a crash.
- termina 0.4.0 (2026-08-31), `MIT OR MPL-2.0`, helix-editor org, 93 stars, 3 open issues; Helix itself depends on it (`termina = "0.3"` in Helix's `Cargo.toml`). Same harness: 100, 200, 500, 3,000 and 20,000 events all delivered with the final position correct (`probe/evcount_termina`). In the prototype through `ratatui-termina` (which pins termina 0.3.3): 20,000 events produced 166 draws and about 0.2 s CPU (dev profile).
- `ratatui-termina` 0.1.0 (2026-06-19, MIT, 929 lines in `src/lib.rs`) renders through a caller-owned `termina::Terminal`. It does not enter raw mode or the alternate screen and installs no panic hook (its README says so), so the app owns about 110 lines of setup and teardown (`proto/src/backend/termina_be.rs`): raw mode, `ClearAndEnableAlternateScreen`, mouse modes 1000, 1002, 1003, 1015, 1006, an `EventReader` on the input thread, and a panic hook that writes the reset sequences.
- Equivalence: the same input script through both backends produced identical vt100 screens and identical bold, underline and colour attributes for every cell (`tests/pty.rs::dump_screen_for_backend_comparison`). Key decoding passes on both for `alt+left` as `ESC [1;3D` and as `ESC ESC [D`, a lone `Esc`, arrows in CSI and SS3 form, UTF-8 and emoji in the filter, `?` and ctrl+u.
- Cost: crossterm variant 75 packages, 49 s clean release build, 834 KB; termina variant 65 packages, 36 s, 806 KB (noisy timings).
- yazi (42.7k stars) left crossterm for its own `yazi-term` layer (`Repos/yazi/yazi-tui/src/raterm.rs:11`), another sign that crossterm is a weak spot.
- Risks of termina: young adapter (0.1.0), `ratatui-termina` still pins termina 0.3 while 0.4 exists, small user base (93 stars), no helpers for setup. Mitigation: the own `Input` layer and the flood test (below) as a CI regression test, so a switch back to crossterm after #1057 merges is a 90-line change that the test validates.
- Verdicts: ratatui-termina yes, termina yes (via it), crossterm maybe (fallback adapter, same feature switch). `termion` and `termwiz` backends: not examined (no reason to leave the default family).

### Frameworks on top of ratatui

- **tui-realm** (`tuirealm` 4.1.0, 2026-05-02, MIT, 244k downloads, 1,001 stars, 23 commits since April, +11 packages incl. regex). React-and-Elm style with `Application::tick`, component ids, subscriptions (`Repos/tui-realm/CLAUDE.md:67`). Mouse hit-testing is left to the app: the standard component library has zero files that mention mouse (`grep -rli mouse crates/tuirealm-stdlib/src` returns nothing). It adds generics (`Id`, `Msg`, `UserEvent`) and mount/unmount machinery that a 9-screen tool does not need. No.
- **rat-salsa and rat-widget** (4.0.3 and 3.2.1, 2026-03-08, MIT or Apache-2.0, 31k downloads, 64 stars, one maintainer, 0 commits since April, +37 packages incl. chrono, ropey, regex-automata). The only family where mouse and focus are first class: `item_at`, `row_at`, `MouseFlags` (`Repos/rat-salsa/rat-event/src/util.rs:16` and `:139`), a mouse-aware `file_dialog.rs` (1,722 lines). But widgets, event loop and state model come as one idiom, and a solo maintainer with no recent commits is a bus-factor risk for the core of our UI. No.
- **cursive** 0.21.1 (2024-08-03, MIT, 2.07M downloads, 4.9k stars, +37 packages with its own crossterm 0.28). Widgets do hit-test themselves (`views/select_view.rs:763`, `checked_sub(offset)`), but `MouseEvent` has only Press, Release, Hold and wheel variants (`Repos/cursive/cursive-core/src/event.rs:485`): no hover. Retained mode with callbacks and a separate widget world also drops ratatui's `TestBackend`. No.
- **ratatui-interact** 0.5.3 (2026-04-02, MIT, 71k downloads, 39 stars, 30,233 lines, +6): a kitchen sink whose `ClickRegionRegistry` is a `Vec<ClickRegion>` with a first-match `find`, so overlays resolve wrongly unless registered in reverse. It confirms the pattern; take the pattern, not the crate. No.

### Text input

- **tui-input** 0.15.5 (2026-09-26, MIT, 2.18M downloads, 205 stars, 15 commits since April, +1 package). `Input` is backend-agnostic (`handle(InputRequest)`, `value`, `visual_cursor`, `visual_scroll(width)`); its crossterm helper filters to Press and Repeat. Use `default-features = false` and map keys to `InputRequest` ourselves (25 lines, `proto/src/input.rs::to_request`); this also removes the Go bug where the filter accepted only single-byte keys (`len(k.String()) == 1`, `cmd/tui/list.go:170`). Yes.
- **ratatui-textarea** 0.9.3 (2026-10-05, MIT, fork of tui-textarea under the ratatui org, 757k downloads, 97 stars; gitui uses 0.8, `Repos/gitui/Cargo.toml:52`). `textarea.rs` alone is 2,706 lines (undo, search, selection). Overkill for one-line fields. No now, maybe if multi-line editing ever appears. `tui-textarea` 0.7.0 (2024-10-22) is superseded.

### Lists, tree, scrolling, progress

- **Built-in** `List`/`ListState` expose `offset()` (`Repos/ratatui/ratatui-widgets/src/list/state.rs:95`) but no item-at-position API, and the visible window is recomputed inside `render` (`list/rendering.rs:129`). So row hit-testing needs either `offset + row/height` arithmetic (the Go bug class) or rects recorded per row. We draw visible rows ourselves and record rects. `Scrollbar`, `Gauge` and `LineGauge` are built in and used. No paginator or checkbox widget exists; both are a few lines.
- **tui-tree-widget** 0.24.1 (2026-08-09, MIT, 1.76M downloads, 129 stars, 1,084 lines, +1 package, depends only on `ratatui-core`, `ratatui-widgets`, `unicode-width`). It records `last_area` and `last_rendered_identifiers` while rendering (`Repos/tui-tree-widget/src/lib.rs:183`) and offers `rendered_at(Position)` and `click_at(Position)` (`src/tree_state.rs:274` and `:291`): the same stored-rect technique as our HitMap, inside the widget. Yes, for the group tree screen (not prototyped: not verified in the app).
- **tui-scrollview** 0.6.8 (2026-09-24, MIT or Apache-2.0, 487k downloads, +1): `ScrollView::new(size)` allocates a `Buffer` of the full content size every frame (`src/scroll_view.rs:86` and `:90`). Fine for small panes, wrong for diffs of thousands of lines, where slicing lines `[offset..offset+h]` is trivial. No.
- **throbber-widgets-tui** 0.11.1 (2026-06-19, Zlib, 968k downloads, 133 lines, +1): a spinner is a frame table indexed by a tick counter (6 lines in `proto/src/ui/list.rs`). No.
- **tui-checkbox**, **tui-popup** (+7 packages with darling), **tui-prompts**, **tui-menu**: not worth a dependency over `Clear` + `Block` + `Paragraph`. No.

### Filesystem picker

- **ratatui-explorer** 0.3.0 (2026-03-06, MIT, 166k downloads, 94 stars, 0 commits since April, +4): builds `ListState::default()` on every render (`Repos/ratatui-explorer/src/widget.rs:22`), so the scroll offset is lost and there is no way to learn row positions; no mouse; whole-list render. No.
- **tui-file-dialog** 0.1.0 (2023-07-16, 1.6k downloads, tui-rs era): abandoned. No. **fpicker**, **ratatree**, **rat-widget FileDialog**: not adoptable without their frameworks or too young (not verified beyond metadata).
- **Own picker**: `proto/src/picker.rs`, 169 lines: directories only, sorted case-insensitively, hidden toggle (`.`), up row, breadcrumb, "select this folder" button, click once to select and again to open, read errors shown in the footer and never swallowed (tested with a mode 000 directory), tests with mouse and keys. This is the main gap versus bubbles `filepicker` (529 lines in `Repos/bubbles/filepicker/filepicker.go`, no mouse handling in it either). Cosmetic issue found: ratatui draws a full scrollbar thumb when everything fits, so hide the scrollbar when `len <= viewport`.

### Fuzzy matching

- **nucleo-matcher** 0.3.1 (2024-02-20, MPL-2.0, 4.67M downloads, +2 packages: memchr). The helix repo is active (1.5k stars, last push 2026-06-24) but the crate has had no release since February 2024 and one commit since April. Only 8 `unsafe` occurrences in `src`. Gives fzf-style atoms (`'exact`, `^prefix`, `!neg`), smart case, and match indices for highlighting (`Pattern::indices`, `proto/src/filter.rs`, 33 lines). Filtering 400 names is instant. Yes. MPL-2.0 is file-level copyleft and allowed; it is the only MPL crate in the set besides the MIT-or-MPL termina. The full `nucleo` (0.5.0, +8 packages with rayon) is for million-row lists. No.
- **frizbee** 0.13.0 (2026-08-13, MIT, 2.9M downloads, 590 stars, 178 commits since April): SIMD, typo tolerant, but 910 `unsafe` occurrences in `src`, MSRV 1.89, built for huge lists. Maybe only if matching ever becomes a bottleneck (it is not a bottleneck at 400 rows).
- **fuzzy-matcher** 0.3.7 (2020-10-04, MIT, 34M downloads) and **sublime_fuzzy** 0.7.0 (2020-12-19, Apache-2.0 text in `LICENSE` but crates.io shows "non-standard"): unmaintained. No.

### Key bindings and help

- Own `Binding { keys, key, desc }` table (35 lines) drives both dispatch and the `?` overlay, which is what bubbles `key.Binding` plus `help` do and prevents drift between the two (`proto/src/binding.rs`). The Go code repeats the same four-field struct per screen (`cmd/tui/keys.go`).
- **crokey** 1.5.0 (2026-07-24, MIT, 6.1M downloads, 45 stars, +7 packages incl. serde and a proc macro): parses `"ctrl-alt-left"` strings, `key!(alt-left)` patterns, display names. Useful only if key bindings become user-configurable in `config.yaml`. Maybe, later. Its API is crossterm-typed, so it would need an adapter next to our own `Input`.

### Hyperlinks (OSC 8), theme, unicode width, effects

- **OSC 8**: ratatui has no link widget. Its example still uses a 2-character-chunk hack for issue #902 (`Repos/ratatui/examples/apps/hyperlink/src/main.rs:55`). Since 0.30.1 `CellDiffOption::ForcedWidth` (`ratatui-core/src/buffer/cell.rs:13`) lets a whole link live in the first cell, with tests at `buffer.rs:1214`. My `Link` widget is 26 lines (`proto/src/ui/link.rs`); PTY test confirms `ESC ] 8 ; ; url ESC \ text ESC ] 8 ; ; ESC \` reaches the terminal. Caveats: `TestBackend` snapshots then contain the raw escape text, so put OSC 8 behind a runtime flag that tests switch off; and the style of a squeezed link sits only on its first cell (the hover test checks that cell). `hyperrat` 0.1.3 (2026-06-28, 11.8k downloads, from the gitv repo): not verified, not needed. Click-to-open uses `std::process::Command::new("xdg-open")` (the Go code does the same, `cmd/tui/browser.go`); the `open` crate costs +4 packages, `opener` +28 (ICU). No crate.
- **Light or dark**: `terminal-colorsaurus` 1.0.3 (2025-12-28, MIT or Apache-2.0, 4.1M downloads, +4 packages). Tested in a PTY against a fake terminal: an answering terminal gives `Ok(Light)` in 0.2 ms; a terminal that only answers the DA1 probe gives `UnsupportedTerminal` in 0.24 ms; a silent terminal gives `Timeout` after the configured duration (default 1 s; 150 ms works). Call it once before the input thread starts, with a short timeout, only when stdout is a TTY and `NO_COLOR` is unset, and expose `theme = auto | dark | light` in config so the failure case is an explicit, documented choice and not a silent fallback. Yes. `terminal-light` 1.9.1 (MIT, +5) is similar but not tested: no. `termbg` 0.6.2 (+17 packages, its own crossterm 0.28 and mockall): no. `dark-light` asks the desktop portal, not the terminal: wrong tool, no.
- **Unicode width**: ratatui measures with `unicode-width` 0.2 and `unicode-segmentation`; `unicode-truncate` is already in its tree. Snapshots with `日本語-skill` and `emoji-🚀-skill` rows keep the project column aligned (`proto/tests/snapshots/snapshot__list_screen_80x24.snap`), because columns are layout cells and not `%-22s` padding by rune count as in Go (`cmd/tui/list.go:331`). The Go filter backspace also slices bytes (`cmd/tui/list.go:162`), which breaks on multi-byte characters. No extra crate.
- **tachyonfx** 0.25.2 (2026-09-06, MIT, 450k downloads, 1.3k stars, +8 packages incl. bon): shader-style effects (fade, sweep). They need a 30 to 60 fps redraw loop while active, which fights the zero-idle-CPU design (measured below), and cannot appear in deterministic snapshots without time injection. Decoration for this tool. No (not built; judged from README and deps).

### Testing

- **insta** 1.49.0 (2026-10-03, Apache-2.0, 108M downloads, +7 packages): `assert_snapshot!(terminal.backend())` on `TestBackend` works as is (its `Display` also marks cells hidden by wide glyphs). Five screen snapshots plus the picker pass. Without `cargo-insta` installed, accept snapshots with `INSTA_UPDATE=always`. Yes.
- **portable-pty** 0.9.0 (2025-02-11, MIT, 19M downloads, +10 packages incl. old `nix` 0.28 and `thiserror` 1) with **vt100** 0.16.2 (2025-07-12, MIT, +4, 0 commits since April, stable): spawn the real binary on a PTY, feed `vt100::Parser`, send raw bytes (SGR mouse reports `ESC [ < 35 ; x ; y M`, wheel code 65, `ESC [1;3D`), resize with `master.resize`, read `/proc/<pid>/stat` for idle CPU, read the raw stream for escape sequences. About 90 lines of harness. Yes for both.
- **expectrl** 0.9.0 (2026-05-11, MIT, +12 packages incl. regex and `nix` 0.26): expect-style text matching with no screen model. No (the CLI stream may still want it for plain CLI tests). **tui-term** 0.3.4 (renders a vt100 screen as a widget, for embedding terminals): no. **termlens**, **ratatui-testlib**, **testty** and the many `ratatui-*` 0.0.0 name reservations on crates.io: not verified, too young or placeholders.

## Design answers

### (a) Application pattern

Ratatui documents three patterns (Elm, Component, Flux) and recommends none ([application patterns page](https://ratatui.rs/concepts/application-patterns/), fetched 2026-10-07); its [event handling page](https://ratatui.rs/concepts/event-handling/) lists centralised matching, message passing and distributed loops, and the Elm page itself says the view only knows the area at draw time and suggests storing sizes between frames. The templates split the same way: `event-driven` is std thread plus `mpsc` with `Event::{Tick, Crossterm, App}` (`Repos/ratatui-templates/event-driven-generated/src/event.rs:56`); `component` is tokio (`features = ["full"]`, `component-generated/Cargo.toml:37`), clap, config, json5, tracing, an `Action` channel and a `Component` trait with `register_action_handler`, `handle_events`, `update`, `draw` (`components.rs:27`). That template is far more than we need.

Recommended: Elm shape with explicit effects, which is what the prototype does. One `App` owns shared state (`rows`, `scan`, `apply`, `view`, `hits`, `hover`). `App::handle(Event)` is the update function and records `Effect`s (open URL, start scan, start apply) that the main loop performs, so every behaviour is testable without a terminal. `ui::draw(frame, &mut App)` is the view and takes `&mut` only to refill the hit map and write back the viewport height. A `Screen` enum is the router; per screen a module with `on_key`, `on_mouse`, `draw` as plain functions (no `Box<dyn Component>`). Use component style only for reusable stateful widgets (`Picker`, a text field, the tree). File size: the prototype's `app.rs` already has 319 lines with three screens, so the real app needs `app/{mod,keys,mouse}.rs` and `screens/<name>/{state,keys,mouse,view}.rs` from day one to respect the 300-line rule.

### (b) Mouse hit-testing

Pattern: clear a `HitMap` at the start of `draw`, push `(Rect, Target)` for every clickable thing as it is drawn (header back arrow, header link, menu items, each visible row, scrollbar track, picker rows, overlay and dismiss areas), and resolve a mouse event by scanning the vector in reverse, so the topmost thing wins (`proto/src/hit.rs`, 51 lines). Hover is a `Moved` event (any-motion tracking), click is `Down(Left)`, wheel is `ScrollUp` and `ScrollDown`. Evidence this is what working apps do: tui-tree-widget records rendered identifiers and offers `click_at` (`src/tree_state.rs:291`); rat-salsa keeps `Vec<Rect>` per widget state and `item_at` (`rat-event/src/util.rs:16`); television stores a `Layout` of rects and routes scroll through a pure function `get_action_for_mouse_event(event, &layout, mode)` with unit tests (`Repos/television/television/mouse.rs:8`; it handles only wheel, not hover or click); yazi recomputes layout from the terminal size on each mouse event by calling its Lua `Root.new(area)` (`yazi-actor/src/app/mouse.rs:26`); gitui has no mouse. The Go code's `msg.Y - itemsStart` with `itemsStart := 6` or `7` (`cmd/tui/list.go:113`) and `const itemsStart = 10` (`setup.go:80`) breaks whenever a line is added or wrapped.

Tested (7 unit tests, headless): a click on row 2 toggles row 2 at 80x24, 60x12 and 120x40; after ten wheel steps a click on row 32 hits row 32; hover moves the cursor and underlines the header link; an overlay swallows clicks and the dismiss area closes it; the scrollbar track jumps; after a resize the next draw rebuilds the map. Known limit: between a state change and the next draw the map is stale (picker test showed this), so always redraw before the next event, which the loop does.

### (c) Event loop and threading

- One input thread calls `poll(50 ms)` then `read()` and forwards typed `Input` values into one `std::sync::mpsc` channel; scan and apply workers hold clones of the same `Sender` and send `EngineEvent`s. The main thread blocks on `recv()` (no timeout) when nothing animates and on `recv_timeout(80 ms)` only while a spinner or progress bar runs, so idle CPU is zero: at most 1 clock tick over 2 s in the PTY test. After the first event it drains the channel with `try_recv` and draws once, which coalesces mouse-motion floods and resize bursts: 20,000 events became 166 draws. The input thread is joinable (flag plus short poll), so after the TUI ends it cannot steal keystrokes from the shell.
- Same design in gitui: std thread with `poll` and `crossbeam_channel::Select` over six receivers (`Repos/gitui/src/main.rs:286`, input thread `src/input.rs`), no tokio. television and yazi use tokio and the component template does too, but nothing in this tool is async: the engine is CPU and disk bound, so threads fit. `crossterm::EventStream` needs `futures` and an executor; skip it. If the web view brings tokio, the TUI is unaffected. If the engine stream settles on `crossbeam-channel`, forward with one 10-line thread or accept a `Sender<Event>` in the engine's observer; do not add crossbeam to the TUI only for this.
- Background progress never blocks drawing because workers only send events. The Go code did the work inside `Update` (`cmd/tui/sync.go:112` and `list.go:236`), which freezes the UI.

### (d) Large lists, resize, key kinds

- 400 rows (and 20,000) cost the same per frame, because only `viewport` rows are built: 200x60 full-screen draw plus diff takes 2.1 ms in release; in a plain dev build 24 ms, which drops to 5.2 ms with `[profile.dev.package.ratatui*] opt-level = 3` (gitui does the same for ratatui, `Repos/gitui/Cargo.toml:91`). Use ratatui's `Scrollbar` plus wheel, `PageUp/PageDown`, `Home/End` and track click; drop the paginator for long lists (bubbles dots can be drawn in 10 lines for the 3-item menu if the look matters).
- Resize needs no handler: `Terminal::draw` re-measures; the PTY test resized 100x30 to 60x20 and the screen re-rendered. Coalesce bursts via the drain loop (television flushes resize events by hand, `television/event.rs`).
- Key kinds: ignore `Release` in `Binding::matches` and in `to_request`; accept `Press` and `Repeat`. Windows specifics: not needed, Linux only.

### (e) Bubbles feature map

| Bubbles/Bubbletea | Rust | Own code |
|---|---|---|
| `tea.Program` per screen, `tea.Quit` to go back | one `App` plus `Screen` enum | router, about 30 lines |
| `spinner` | frame table and tick counter | 6 lines |
| `paginator` (dots) | virtual window plus `Scrollbar` | none (dots optional, 10 lines) |
| `progress` | `LineGauge` or `Gauge` | none; real events, no easing needed |
| `help` and `key.Binding` | own `Binding` table and overlay | 35 lines plus 33 lines overlay |
| `filepicker` | own `Picker` | 169 lines (gap) |
| `textinput` | `tui-input` state plus key mapping | 25 lines |
| `list` (menu with description) | `Paragraph` rows plus hit rects | 22 lines |
| `list` filter | `nucleo-matcher` plus `tui-input` | 33 lines, adds match highlighting |
| lipgloss styles | `Style`/`Stylize` plus a `Theme` struct | theme plus `NO_COLOR` switch |
| mouse `MouseAllMotion` | `?1003h` via backend | `HitMap`, 51 lines |
| clickable header link | `Link` widget (OSC 8) plus `xdg-open` | 26 lines |

## Prototype results

| Check | crossterm backend | termina backend |
|---|---|---|
| Snapshot tests (5 screens) and picker (2) | pass | pass |
| Headless mouse and key tests (7) | pass | pass |
| PTY: full session (menu, scan spinner, filter, click, hover, wheel, resize, idle CPU, quit) | pass | pass |
| PTY: OSC 8, mouse modes, alt screen restore | pass | pass |
| PTY: 6 key-decoding tests | pass | pass |
| PTY: motion flood 100 / 500 / 3,000 / 20,000 events | pass / stall / stall / stall | pass at all sizes (20,000: 166 draws) |
| Same screen and attributes | identical | identical |
| Clean release build, binary, packages | 49 s, 834 KB, 75 | 36 s, 806 KB, 65 |

Prototype size: 1,347 lines in `src` (19 files), 534 lines in `tests` before the key tests. Release profile used `lto = true`, `opt-level = "s"`, `strip = true`.

## Recommendation

```toml
[dependencies]
ratatui         = { version = "0.30", default-features = false, features = ["std", "underline-color", "layout-cache", "macros", "all-widgets"] }
ratatui-termina = "0.1"                       # brings termina (MIT OR MPL-2.0: use under MIT)
tui-input       = { version = "0.15", default-features = false }
tui-tree-widget = "0.24"                      # group tree screen
nucleo-matcher  = "0.3"                       # MPL-2.0, file-level copyleft
terminal-colorsaurus = "1.0"

[dev-dependencies]
insta = "1.49"
portable-pty = "0.9"
vt100 = "0.16"

[profile.dev.package.ratatui]         # keep the dev build usable, as gitui does
opt-level = 3
[profile.dev.package.ratatui-core]
opt-level = 3
[profile.dev.package.ratatui-widgets]
opt-level = 3
```

Own code (sizes from the prototype): `input.rs` (neutral key and mouse types, 105), `backend/termina_be.rs` (113), `hit.rs` (51), `binding.rs` (35), `picker.rs` (169), `ui/link.rs` (26), filter (33), spinner, overlays. Keep the flood test and a stalled-input check in CI so a crossterm switch is verifiable. Do not ship both backends.

## Rejected and why

tui-realm, rat-salsa and rat-widget, cursive, ratatui-interact (frameworks and kitchen sinks, see above); ratatui-explorer and tui-file-dialog (no scroll state or abandoned); tui-scrollview (full content buffer per frame); throbber-widgets-tui, tui-checkbox, tui-popup, tui-prompts (cheaper to write); ratatui-textarea (multi-line editor for one-line fields); tachyonfx (decoration, redraw loop); nucleo (rayon for 400 rows), frizbee (910 unsafe sites), fuzzy-matcher and sublime_fuzzy (unmaintained since 2020); termbg, terminal-light, dark-light (worse fit than colorsaurus); expectrl and tui-term (no screen model or wrong job); `open`, `opener` (a `xdg-open` call is enough); tokio and `crossterm::EventStream` (nothing async in the TUI).

## Good but licence-blocked

None found in the TUI space. Checked: the 160 most downloaded crates returned by ten crates.io searches (tui, ratatui, terminal ui, crossterm, fuzzy matcher, tui widget, terminal input, pty test, terminal hyperlink, terminal color theme detect) plus every crate named in this file. Non-permissive hits in that sweep were all search noise and none is a TUI candidate: `evalexpr` AGPL-3.0-only, `gifski` AGPL-3.0-or-later, `syd` GPL-3.0-only, `malachite-*` LGPL-3.0-only, `downloader` LGPL-3.0-or-later, `temporalio-sdk` non-standard. Licences seen on the candidates: MIT, Apache-2.0, MIT OR Apache-2.0, Zlib (throbber-widgets-tui), BSD-2-Clause OR Apache-2.0 (a portable-pty dependency), Unlicense OR MIT (memchr), Unicode-3.0, Apache-2.0 OR BSL-1.0 (ryu: BSL here is the permissive Boost licence, not BUSL).

Marked MPL-2.0 (allowed): `nucleo-matcher` 0.3.1 (in the recommended set), `nucleo` 0.5.0 (rejected for other reasons), `termina` 0.3.3 and 0.4.0 (`MIT OR MPL-2.0`, take MIT), `pulldown-cmark-mdcat` (`MPL-2.0 AND Apache-2.0`, not used). `sublime_fuzzy` shows "non-standard" on crates.io, its `LICENSE` is the Apache-2.0 text (rejected for age, not licence). `rat-salsa` shows no licence on GitHub but `MIT/Apache-2.0` in Cargo metadata.

Transitive check of the recommended set (`cargo tree -e normal,build -f '{p}|{l}'` on the prototype, both backends): 65 packages with termina, 75 with crossterm, no GPL, LGPL, AGPL, SSPL, BUSL, Elastic or non-commercial licence, and no unknown licence. The only copyleft items are nucleo-matcher (MPL-2.0) and termina (dual, MIT chosen). Dev-dependency trees (insta, portable-pty, vt100) showed only MIT, Apache-2.0, BSD-2-Clause OR Apache-2.0 and Unlicense OR MIT in the delta counts; they were not run through the full audit script (not verified in full).

## Risks and open points

1. Backend maturity: `ratatui-termina` 0.1.0 and termina 0.3/0.4 are young. Mitigations above. Real-terminal coverage (kitty, foot, alacritty, tmux, ssh) is not verified, only vt100 emulation. A manual smoke test on the user's terminal is still required before relying on hover.
2. crossterm may fix #1057 later; recheck at implementation time (`gh pr view 1057 -R crossterm-rs/crossterm`).
3. nucleo-matcher has had no release since 2024-02-20, which misses the "release in twelve months" rule in the requirements; the code is small, stable and the repo is active. Fallback is a 60-line own subsequence matcher.
4. OSC 8 depends on `CellDiffOption::ForcedWidth`, new in ratatui 0.30.1; keep it behind a flag and out of snapshots.
5. Group tree screen (tui-tree-widget) and the setup wizard flow were not built into the prototype; the picker and list were.
6. ratatui's `Scrollbar` draws a full thumb when content fits; hide it in that case.
7. Hover over SSH sends many motion events; the drain loop copes, but latency is the terminal's.

## Changes suggested for the requirements doc

- Add: input must survive bursts (paste, mouse floods) and a PTY regression test must cover it; the TUI must use virtual lists with a scrollbar (the paginator is dropped); light or dark `theme = auto|dark|light` plus `NO_COLOR`; optional OSC 8 link with a runtime flag; hit-testing by recorded rectangles, never row arithmetic.
- The requirement "rendering must be testable without a terminal" is met by `TestBackend` plus `insta` plus a pure `App::handle`; say so, and add PTY end-to-end tests with `portable-pty` and `vt100`.
- Engine events: the TUI wants a `Sender<Event>` (or a `Fn(EngineEvent) + Send`) observer, not an async stream.
- File size: plan a module-per-screen layout up front (the 300-line rule is hit at three screens).

## Clones

All under `Repos/`, shallow:

- `ratatui` (https://github.com/ratatui/ratatui): workspace layout, `CellDiffOption`, List internals, hyperlink example, changelog.
- `ratatui-templates` (https://github.com/ratatui/templates): simple, event-driven and component templates (cloned as `templates`, renamed).
- `tui-widgets` (https://github.com/ratatui/tui-widgets): scrollview, popup, prompts source (only metadata used).
- `ratatui-textarea` (https://github.com/ratatui/ratatui-textarea): size and scope check.
- `tachyonfx` (https://github.com/ratatui/tachyonfx): effects crate source.
- `tui-realm` (https://github.com/veeso/tui-realm): framework architecture and the no-mouse finding.
- `rat-salsa` (https://github.com/thscharler/rat-salsa): mouse utilities and file dialog.
- `tui-tree-widget` (https://github.com/EdJoPaTo/tui-rs-tree-widget, cloned as `tui-tree-widget`): `rendered_at` and `click_at`.
- `ratatui-explorer` (https://github.com/tatounee/ratatui-explorer): the per-frame `ListState::default()` finding.
- `cursive` (https://github.com/gyscos/cursive): no hover events, widget-side hit-testing.
- `yazi` (https://github.com/sxyazi/yazi): own terminal layer, per-event layout, ratatui-core patch.
- `gitui` (https://github.com/gitui-org/gitui): std threads plus `Select`, key-release filter, dev profile.
- `television` (https://github.com/alexpasmantier/television): pure mouse function over stored layout, tokio loop.
- Used but cloned by the lead: `bubbles` (feature map for filepicker and mouse).

## Verdict table

| Crate | Verdict | Reason | Version checked | Licence |
|---|---|---|---|---|
| ratatui (+ ratatui-core, ratatui-widgets) | yes | renderer, widgets, `TestBackend`, 0.30 split | 0.30.2 (0.1.2, 0.3.2) | MIT |
| ratatui-termina | yes | reader works under bursts, identical rendering, smaller tree | 0.1.0 | MIT |
| termina | yes | the backend's terminal and event reader (via ratatui-termina) | 0.3.3 used, 0.4.0 latest | MIT OR MPL-2.0 |
| crossterm | maybe | stalls on input bursts (#1057 open), keep a 90-line fallback adapter | 0.29.0 | MIT |
| tui-realm (tuirealm) | no | mouse left to the app, extra generics, +11 packages | 4.1.0 | MIT |
| rat-salsa, rat-widget | no | one idiom, solo maintainer, +37 packages | 4.0.3, 3.2.1 | MIT/Apache-2.0 |
| cursive | no | no hover events, own widget world, 14 months old | 0.21.1 | MIT |
| ratatui-interact | no | 30k-line kitchen sink; pattern adopted, crate not | 0.5.3 | MIT |
| tui-input | yes | single-line state, backend-neutral, +1 package | 0.15.5 | MIT |
| ratatui-textarea | no | 2,706-line multi-line editor for one-line fields | 0.9.3 | MIT |
| tui-textarea | no | superseded | 0.7.0 | MIT |
| tui-tree-widget | yes | group tree with built-in `click_at`, +1 package | 0.24.1 | MIT |
| tui-scrollview | no | full-content buffer per frame | 0.6.8 | MIT OR Apache-2.0 |
| throbber-widgets-tui | no | 6 lines to write | 0.11.1 | Zlib |
| ratatui-explorer | no | resets scroll state every frame, no mouse | 0.3.0 | MIT |
| tui-file-dialog | no | abandoned since 2023 | 0.1.0 | MIT |
| tui-popup, tui-prompts, tui-checkbox, tui-menu | no | cheaper to write than to depend on | 0.7.7, 0.6.8, 0.4.6, 0.3.1 | MIT OR Apache-2.0 / MIT |
| nucleo-matcher | yes | fzf-style fuzzy with indices, 8 unsafe sites, +2 packages | 0.3.1 | MPL-2.0 |
| nucleo | no | rayon and async for million-row lists | 0.5.0 | MPL-2.0 |
| frizbee | maybe | fast SIMD, but 910 unsafe sites; only if matching becomes slow | 0.13.0 | MIT |
| fuzzy-matcher, sublime_fuzzy | no | unmaintained since 2020 | 0.3.7, 0.7.0 | MIT, Apache-2.0 (file) |
| crokey | maybe | only if keys become configurable, +7 packages | 1.5.0 | MIT |
| terminal-colorsaurus | yes | light or dark in 0.2 ms, explicit timeout and unsupported errors | 1.0.3 | MIT OR Apache-2.0 |
| terminal-light, termbg, dark-light | no | untested, +17 packages, or wrong tool | 1.9.1, 0.6.2, 3.0.0 | MIT, MIT OR Apache-2.0, MIT/Apache-2.0 |
| tachyonfx | no | decoration, forces redraw loop, +8 packages | 0.25.2 | MIT |
| insta | yes | `TestBackend` snapshots | 1.49.0 | Apache-2.0 |
| portable-pty | yes | PTY end-to-end harness (about 90 lines) | 0.9.0 | MIT |
| vt100 | yes | screen model for PTY assertions | 0.16.2 | MIT |
| expectrl, tui-term | no | no screen model, or wrong job | 0.9.0, 0.3.4 | MIT |
| open, opener | no | `xdg-open` via std is enough | 5.4.4, 0.9.0 | MIT, MIT OR Apache-2.0 |
| tokio, crossbeam-channel | no | std threads and `mpsc` suffice in the TUI | 1.53.2, 0.5.17 | MIT, MIT OR Apache-2.0 |
| unicode-width, unicode-segmentation, unicode-truncate | yes | transitive via ratatui, correct column widths | 0.2.2, 1.13.3, 2.0.1 | MIT OR Apache-2.0 |
