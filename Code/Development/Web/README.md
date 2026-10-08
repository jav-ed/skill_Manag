# Web check

Two scripts for the web interface (`Ui/`, served by `skillmirror web`).

## `build_Ui.sh`: build the interface and put it where Rust embeds it

```bash
Code/Development/Web/build_Ui.sh           # npm ci, type check, build, copy to Crates/Web/assets/ui/
Code/Development/Web/build_Ui.sh --check   # build to a temporary folder, fail when the committed files differ
```

The built files are committed, so `cargo build` and `cargo install` need no node. Run the script after every change in `Ui/` and commit `Crates/Web/assets/ui/` with it; `--check` is the guard that this was done (the build is reproducible: file names carry a hash of their content). Needs node 22.12 or newer. CI does not run it (no node job yet); run it before a push that touches `Ui/`.

## `check_Web.sh`: drive the interface in a browser

Builds a throwaway world (a vault with a group, four projects, one named like markup), starts `skillmirror web --allow-write` on it and drives the pages in Chromium through Playwright: the first-visit link and its cookie, the overview and its counters, a project that opens and sends you to sync, the filter, a sync plan with its diff and its apply, a push, the skills page, the history and an undo, doctor and settings, the theme button and its memory, a direct visit of an inner address, the refusals a page script can provoke, any request that leaves the server, and console errors (a Content-Security-Policy violation is one). It prints one `ok` or `FAIL` line per check and exits 1 on a failure. Screenshots are written when a folder is given.

```bash
Code/Development/Web/check_Web.sh                                  # target/debug/skillmirror
Code/Development/Web/check_Web.sh path/to/skillmirror /tmp/shots   # also writes overview.png, plan.png, ...
```

Needs: bash, git, node with Playwright (`PLAYWRIGHT_NODE` names its folder, default `/opt/node-tools/node_modules/playwright`), and a Chromium (`PLAYWRIGHT_CHROMIUM` names the binary when Playwright does not find its own). It touches nothing outside a temporary folder: not your vault, not your projects, not your config.

| file | job |
|---|---|
| `build_Ui.sh` | builds `Ui/` into `Crates/Web/assets/ui/`, or checks that it is up to date |
| `check_Web.sh` | makes the world, starts the server, reads the link |
| `drive_Web.js` | the browser checks |
