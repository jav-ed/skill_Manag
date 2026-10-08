# Setup: Build resources

Building the Rust workspace is the heaviest thing this repository asks of a machine. The running tool is small and quick (see the timings in the [handoff](../../Live_Working/Rust_Rewrite_Handoff/current_State.md)); the compiler is what eats memory, disk and time. Read this before you build on a small machine, a shared server or a container with a fixed disk allowance.

## What it costs

Measured on 2026-10-08, 4 cores, 16 GB RAM, one container, clean build of everything including the tests (`cargo test --workspace --no-run`) unless the row says otherwise, a separate target folder per row. **Peak memory** is the sum of every build process's proportional memory (PSS from `/proc/<pid>/smaps_rollup`, sampled every 0.1 s, which counts shared library pages once, not once per compiler process); **largest process** is the biggest single one. Read the first as "how much RAM must be free for this not to swap".

| Settings | Disk | Time | Peak memory, all processes | Largest process |
|---|---|---|---|---|
| Cargo defaults (full debug information, incremental cache) | 2.5 GB | 51 s | 1.74 GB | 603 MB |
| **This repository's default** (`debug = false`, `incremental = false`) | **0.58 GB** | **35 s** | **1.21 GB** | **351 MB** |
| Same, one compile job at a time (`CARGO_BUILD_JOBS=1`) | 0.58 GB | 123 s | 0.51 GB | 411 MB |
| `cargo build --release -p skillmirror` (what `cargo install` runs) | 0.36 GB | 72 s | 0.91 GB | 457 MB |
| Same, `CARGO_BUILD_JOBS=1` | 0.36 GB | 255 s | 0.50 GB | 427 MB |
| `cargo build --profile dist -p skillmirror` (fat LTO, one codegen unit; what you ship) | 0.33 GB | 113 s | 1.02 GB | 694 MB |

What the table says: turning debug information and the incremental cache off cuts the disk by three quarters (2.5 to 0.58 GB) and the peak memory by about a third, and makes the build faster. It does not make the build light. The compiler still needs more than a gigabyte of RAM free at its peak with all four cores busy, and **the lever on memory is the number of parallel jobs**: `CARGO_BUILD_JOBS=1` brought the peak to about 500 MB in the two rows measured (506 and 503 MB) and costs time (3.5 times as long for the tests, 3.5 times for a release build). The `dist` profile has the biggest single process (694 MB), because link-time optimisation holds the whole program at once; `CARGO_PROFILE_DIST_LTO=thin` should lower it (not measured).

The earlier version of this page (written before the local web server existed, measuring system-wide memory instead) said 0.47 GB, 27 s and about 500 MB for the repository default. The same row measured with the same method as the rest of this table is 0.58 GB, 35 s and 1.21 GB peak (866 MB by system-wide memory drop, the old method): the web server and the growth of the test suite cost real memory. If you have an old number in your head, replace it.

The whole local gate (`Code/Development/Gate/check_Gate.sh`: clippy and format on two toolchains, deny, every test) from a clean folder with the settings here took 63 s and left 0.7 GB in `target/` (measured before the web server; with a warm `target/` it takes 48 s today). The same work with Cargo's defaults had left 13 GB: a `target/` folder that is used for a few days of ordinary development (build, test and lint over many days) grows to about that, 12 GB of it the debug profile (6.1 GB compiled dependencies, 5.3 GB incremental cache). Nothing in it is wrong; it is what Cargo's defaults produce.

After a one-line edit the rebuild takes 7 s with the default settings here and 2 s with the incremental cache on (measured before the web server, not repeated). The cache buys 5 seconds and costs a second copy of the build in memory and gigabytes of disk. That is why it is off.

The local web server (`skillmirror web`, cargo feature `web`, on by default) brings `axum`, `hyper` and `tokio`. Measured on the real tool, `cargo build --release -p skillmirror` with and without `--no-default-features`: 72 s against 59 s (13 s more), a 355 MB build folder against 289 MB (66 MB more), 457 MB against 367 MB in the largest compiler process (90 MB more), an 11.1 MB binary against 8.7 MB (2.4 MB more; the `dist` binary is 6.0 MB), and the same peak memory (912 against 903 MB). `cargo build --no-default-features -p skillmirror` leaves it out, and `just check-features` keeps that build working.

Changing the web interface (`Ui/`, only then) needs node 22.12 or newer and a `node_modules` folder of about 370 MB (261 packages, all dev-time; the built interface is 224 KB and is committed). `cargo build` and `cargo install` never touch it.

To measure a row yourself without `/usr/bin/time` (which minimal containers lack), sample the process tree: read `Pss:` from `/proc/<pid>/smaps_rollup` for the build's pid and all its descendants every 100 ms and keep the maximum of the sum and of the single largest. Do not sum `VmRSS`: shared pages of the compiler's own libraries are counted once per process.

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

Installing the tool for use is `cargo install --path Crates/Cli --locked` (compiles in release mode: 72 s and about 0.9 GB of peak memory on four cores, 255 s and 0.5 GB with `CARGO_BUILD_JOBS=1`; then the build folder is thrown away). A prebuilt binary needs no compiler at all: `cargo build --profile dist` once (113 s), copy `target/dist/skillmirror` (6.0 MB) wherever you want it.
