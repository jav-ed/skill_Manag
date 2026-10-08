# Setup: Build resources

Building the Rust workspace is the heaviest thing this repository asks of a machine. The running tool is small and quick (see the timings in the [handoff](../../Live_Working/Rust_Rewrite_Handoff/current_State.md)); the compiler is what eats memory, disk and time. Read this before you build on a small machine, a shared server or a container with a fixed disk allowance.

## What it costs

Measured on 2026-10-08, 4 cores, 16 GB RAM, clean build of everything including the tests (`cargo test --workspace --no-run`), a separate target folder per row.

| Settings | Disk | Time | Extra system memory at peak | Largest single process |
|---|---|---|---|---|
| Cargo defaults (full debug information, incremental cache) | 1.9 GB | 38 s | about 810 MB | 490 MB |
| `debug = "line-tables-only"`, no debug information for dependencies | 1.2 GB | 34 s | not measured | 395 MB |
| **This repository's default** (`debug = false`, `incremental = false`) | **0.47 GB** | **27 s** | **about 500 MB** | **375 MB** |
| Same, one compile job at a time (`CARGO_BUILD_JOBS=1`) | 0.47 GB | about 100 s | a fraction of the above | 375 MB |

Other builds, same machine: `cargo build --release` 55 s and 287 MB on disk (largest process 335 MB); `cargo build --profile dist` (fat LTO, one codegen unit, what you ship) 89 s, about 720 MB of memory at peak (largest process 565 MB) and 267 MB on disk. The `dist` profile is the one that needs the most memory per process, because link-time optimisation holds the whole program at once.

The whole local gate (`Scratch/gate.sh`: clippy and format on two toolchains, deny, every test) from a clean folder with the settings here took 63 s and left 0.7 GB in `target/`. The same work with Cargo's defaults had left 13 GB.

After a one-line edit the rebuild takes 7 s with the default settings here and 2 s with the incremental cache on. The cache buys 5 seconds and costs a second copy of the build in memory (900 MB at peak) and gigabytes of disk. That is why it is off.

The number that surprised us: a `target/` folder that had been used for a few days of ordinary development had grown to **13 GB**, 12 GB of it the debug profile (6.1 GB compiled dependencies, 5.3 GB incremental cache). Nothing in it was wrong; it is what Cargo's defaults produce when you build, test and lint over many days.

## What this repository does about it

The root `Cargo.toml` turns off the two things that cost the most and help the least:

```toml
[profile.dev]
debug = false          # no DWARF debug information
incremental = false    # no incremental compilation cache
```

`test` inherits from `dev`, so `cargo test` and `cargo nextest` get the same. Panics and `RUST_BACKTRACE=1` still name the functions, only the source line numbers are missing. CI sets `CARGO_INCREMENTAL=0` for the same reason.

## When you need more

- **Stepping through code in a debugger.** Ask for it for one shell, not for the repository: `CARGO_PROFILE_DEV_DEBUG=true cargo build`. Or use `cargo build --profile profiling` (release speed plus line tables).
- **Fast edit-and-test loops.** `CARGO_INCREMENTAL=1 cargo test` for the session you need it in. Run `cargo clean` afterwards.
- **Less memory at once.** `CARGO_BUILD_JOBS=2` (or `1`) limits the number of compiler processes. It is the single biggest lever on peak memory and it costs time, not correctness. For a `dist` build also try `CARGO_PROFILE_DIST_LTO=thin`.
- **Less disk.** `cargo clean` removes everything; `cargo clean --profile dev` removes only the debug profile; deleting `target/debug/incremental` is always safe. Several clones can share one folder with `CARGO_TARGET_DIR=~/.cache/skillmirror-target`.
- **rust-analyzer** builds into its own folder by default and doubles the disk. Setting `rust-analyzer.cargo.targetDir` to `true` makes it use `target/rust-analyzer`; pointing it at the folder Cargo uses makes the two block each other instead. Pick one.
- **`just check`** runs formatting, clippy and all tests, so it compiles the workspace twice over (a type-check pass for clippy and a full build for the tests). The first run after a `cargo clean` is the expensive one.

## What does not need any of this

Installing the tool for use is `cargo install --path Crates/Cli --locked` (compiles in release mode, about a minute, then the build folder is thrown away). A prebuilt binary needs no compiler at all: `cargo build --profile dist` once, copy `target/dist/skillmirror` (4.8 MB) wherever you want it.
