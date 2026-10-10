# Smoke

`check_Smoke.sh` runs a built `skillmirror` binary end to end in a throwaway world, the way a new user would: make a vault, write a skill, install it into a project, change it in the vault, see the drift (`status`, `diff`), `sync`, `undo` and redo, `adopt` a skill a project made by hand, `info`, `add` to a second project, the text of the AGENTS.md blocks (seeded into `project-files/AGENTS.md` and never overwritten, a root `AGENTS.md` of the vault ignored) and the AGENTS.md of a project (written by `init` with its closing note, checked, `status` saying the blocks are behind, updated with a diff while your own text beside the block survives, undone, left out with `--no-agents-md`, refused when the vault text cannot be used, and named by `doctor`), and then start `skillmirror web --allow-write` and talk to it over HTTP with `curl`: the one-time link, the cookie, the refusals (no cookie, a wrong Host header, a change without the JSON header), a plan, its apply, the job, and a plan that cannot run twice.

```bash
cargo build --locked && Code/Development/Smoke/check_Smoke.sh                       # target/debug/skillmirror
cargo install --path Crates/Cli --locked --root /tmp/inst && Code/Development/Smoke/check_Smoke.sh /tmp/inst/bin/skillmirror
```

It prints one `PASS` or `FAIL` line per check and ends with `SMOKE OK`, or `SMOKE FAILED: n check(s)` and exits with `n`. Pointed at a binary that does nothing (`/bin/true`) it fails with 37 checks, so it cannot pass by accident.

Needs bash, git, curl and python3 (only to read JSON fields). It uses a temporary `HOME`, vault and scan root and removes them on exit: not your vault, not your projects, not your configuration.

What it is for: the one thing the Rust tests cannot say is whether the artifact a user gets, built from a clean checkout and installed, works. Run it against `cargo install --path Crates/Cli --locked` from a fresh clone before a release. It does not replace `Code/Development/Web/check_Web.sh` (the browser side).
