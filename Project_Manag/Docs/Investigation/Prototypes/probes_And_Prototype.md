# Prototypes: probes and the TUI prototype

*Two archives hold the programs behind the terminal-library decision. Extract them outside this repository (they carry their own empty `[workspace]` table, so Cargo treats them as separate projects, but a build output inside the repo would be picked up by tooling). Versions were current on 2026-10-07; re-check before relying on a result after a dependency bump.*

```bash
mkdir -p /tmp/skillmirror-proto && cd /tmp/skillmirror-proto
tar xzf <repo>/Project_Manag/Docs/Investigation/Prototypes/tui_Probe.tar.gz
tar xzf <repo>/Project_Manag/Docs/Investigation/Prototypes/tui_Proto.tar.gz
```

## `tui_Probe.tar.gz`: three single-file probes

| Probe | What it does | Result on 2026-10-07 |
|---|---|---|
| `evcount` (crossterm 0.29) | Raw mode, counts mouse-move, key and other events, writes the counts to the file named by the `OUT` variable, quits on `q` | Fed 100 mouse-motion reports (1.2 KB) in one write by a pseudo-terminal harness it keeps up; 200 events (2.3 KB) stall and the later `q` is never read. Upstream issue crossterm #1057 was open |
| `evcount_termina` (termina 0.4) | Same counting through `termina`'s `PlatformTerminal` | 100, 200, 500, 3,000 and 20,000 events all arrive with the last position correct |
| `theme_probe` (terminal-colorsaurus 1.0) | Asks the terminal for its background colour with a timeout from `TIMEOUT_MS` | `Ok(Light)` or `Ok(Dark)` in about 0.2 ms from an answering terminal, an explicit error from one that does not answer |

The harness that wrote the event bursts was a few lines of `portable-pty`; the same technique is in `Crates/Cli/tests/common/pty.rs`. To reproduce: build a probe, spawn it in a pseudo-terminal, write N SGR mouse-motion reports (`ESC [ < 35 ; col ; row M`) in a single `write`, then send `q` and read the `OUT` file.

Why it matters: the Rust TUI uses `ratatui-termina`, and the flood test in the first prototype guards that choice. If crossterm fixes the stall, swapping back is a 90-line adapter; the app only sees its own `Input` type (`Crates/Tui/src/input.rs`).

## `tui_Proto.tar.gz`: the first TUI prototype

1,347 source lines in 19 files plus tests: menu, list screen with a fuzzy filter, a directory picker, a help overlay, an OSC 8 link, a spinner fed by a worker thread, a hit map for mouse targets and a key binding table. It was ported into `Crates/Tui` (see `Docs/Decisions/rust_Rewrite.md`, section 4), so it is evidence and a reference, not a base for new work.

```bash
cd /tmp/skillmirror-proto/tui_Proto
cargo test --locked                                   # default backend: crossterm
cargo test --locked --no-default-features --features termina --test pty motion_flood
```

Expected: with the default crossterm backend nine tests pass and `motion_flood_is_coalesced` FAILS with "BACKEND STALL: writer still blocked after 3 s; the child stopped reading a 2000-event burst"; that failure is the reproduction of the bug. With `--features termina` the same test passes (checked 2026-10-08). The other tests (insta snapshots in `tests/snapshots/`, mouse hit tests, picker, key forms including Alt+Left in two encodings, UTF-8 paste in the filter, cpu ticks at idle) pass on both backends.

Where the numbers are used: [TUI stack report](../Rust_Stack/tui_Stack.md), sections "Prototype results" and the crossterm findings.
