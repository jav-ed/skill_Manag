# Gate

Two scripts for proving a change before it is pushed.

## `check_Gate.sh`: everything, in one command

```bash
Code/Development/Gate/check_Gate.sh
```

Runs, in this order and stopping at the first failure: `cargo fmt --check`, clippy with `-D warnings`, the same two on a second toolchain (default `1.99.0`, which is what CI runs; clippy lints differ between versions; set `GATE_SECOND_TOOLCHAIN`, and the script says when it skipped them), `just loc-gate` (300 code lines per file), `just check-features` (the CLI builds without the web server), `just check-deps` (the engine has no `tokio`), `just deny` (advisories, bans, licences, sources) and every test. It ends with `GATE OK` and the test count. About one minute on four cores once the dependencies are built. Needs `tokei`, `jq` and `cargo-deny` on the PATH.

`just check` is the shorter form without `deny` and the second toolchain. CI does not yet run `check-features` or the `Ui/` checks (see `Project_Manag/Live_Working/open_Issues.md`), so this script is where those two are held today.

## `check_Mutation.sh`: does a test notice when the guard is gone?

```bash
Code/Development/Gate/check_Mutation.sh Crates/Web/src/server/guard.rs 's/"sec-fetch-site"/"sec-fetch-sitx"/' -p skillmirror-web --features server --locked
```

Applies one `sed` expression to one source file, runs `cargo test` with the arguments you give, prints the failing tests (or `ok`), and puts the file back whatever happens. **Name the feature the file is built with**: the web server code is compiled only with `--features server` on `skillmirror-web`, and without it the mutated line is never compiled, so every test stays green and the check proves nothing (a 13-test run instead of 72 is the sign). A mutation that leaves every test green is a survivor: the guard is not proven. Write the test that fails, run the mutation again, then drop the mutation. Used on the web server's guards (Host, Origin, Sec-Fetch-Site, the JSON header, the single-use plan), the backup swap-back and the undo run id; each survivor found that way got a test.

## What neither does

Neither starts a terminal or a browser. For those see `Code/Development/Smoke/` (the built binary end to end), `Code/Development/Web/` (Chromium) and `Code/Development/Parity/` (the Go tool's behaviour).
