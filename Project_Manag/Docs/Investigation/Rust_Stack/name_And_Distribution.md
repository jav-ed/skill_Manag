# Rust stack: Name and distribution

Recommendation: rename the tool to `skillmirror` (crate, binary, GitHub repo `jav-ed/skillmirror`, config directory `~/.config/skillmirror/`, environment variables `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT`), with the optional installer-made alias `skmir`. Of 25 candidates it is the best balance: it says what the tool does (the README calls sync "a mirror, not an overlay"), it is free on crates.io, AUR, Homebrew, npm and PyPI, it has no same-purpose GitHub project, and unlike `skill-manag` it does not look like a typo of a taken crate. The skill namespace is saturated (`skillctl`, `skill-manager`, `skm`, `skillhub`, `skillsmith` are taken and several in-purpose tools exist). Build the product as one virtual Cargo workspace under `Crates/` (`Core`, `Cli`, `Tui`, `Web`, `Testkit`, `Xtask`): measured on equivalent 10.4k-line prototypes it makes a rebuild after a TUI or CLI edit 6 to 9 times cheaper, enforces "core never depends on the front ends" at compile time (a single package does not), and keeps the async runtime out of the CLI and TUI build (tokio appears only through the `Web` crate). Keep the frozen Go tree in place beside the new root `Cargo.toml` until cutover, tag the oracle commit, and delete Go in one commit at cutover. Release with cargo-dist (GitHub Releases, Linux gnu and musl, checksums, attestations), cargo-release for the version bump and tag, git-cliff for the changelog, a source PKGBUILD in the repo from day one and `-bin` after the first release; crates.io, release-plz, a shell installer and aarch64 come later or never. The Hippocratic licence blocks no tool we use, but it blocks Homebrew core, Debian main and (very likely) Fedora, and it forces `license-file = "LICENSE"` instead of an SPDX field.

Scope note: the user decided the tool is Linux only (see [rewrite_Requirements.md](rewrite_Requirements.md): Platforms). Homebrew, winget, scoop, PowerShell and MSI installers, and macOS or Windows cross builds are out of scope, Linux only. Config handling and tests are shared with [cli_Stack.md](cli_Stack.md) (config paths, deny.toml, parity harness, 300-line gate), which this file does not repeat. Ecosystem context is in [prior_Art.md](prior_Art.md).

Evidence conventions: every check was run on 2026-10-07 from the repo machine (Manjaro, rustc 1.98.1, go 1.27.0). Prototypes and raw results are in the session scratchpad `.../scratchpad/dist/` (`results.jsonl`, `gen/`, `check.sh`), not in the repo. Tools marked "not installed, not measured" were judged from docs and source only.

## Name

### How it was checked

`curl` against the crates.io API (User-Agent `skill-manager-research`, one request per second), AUR RPC v5 (`info` for `n`, `n-bin`, `n-git`), the Homebrew formula and cask JSON (8,645 formulae), repology (`/api/v1/project/<n>`, covers Arch, Debian, Fedora, Nix), npm and PyPI registries, `gh api search/repositories` (name matches sorted by stars), `pacman -F usr/bin/<n>` and `which`, Debian contents search for `/usr/bin/<n>`, and `cargo search` for near names. No candidate contains a trademark.

Findings that shape the choice:
- crates.io already holds a dozen Rust tools for the same job (`capsync`, `akm`, `dalo`, `kasetto`, `aps`, `knack`, `spm-cli`, `terraphim-skills`, `skill-harness`, `agents-skills`, `skillctl`, `skill-manager`; `cargo search "skill sync"` and `"agent skills"`). Generic names are gone.
- GitHub has tens of thousands of name matches for "skill"; a collision is only a problem when the other project does the same job (listed in the table).
- crates.io treats `-` and `_` as the same, so `skill_manag` and `skill-manag` are one name (both free). `skillmirror` and `skill-mirror` are two different names (both free); pick the single-word form.

### Candidates

Legend: f = free, T = taken. GitHub column counts repos named exactly that (hyphens ignored, among the top 30 name matches) and names the same-job ones.

| Name | crates.io | AUR | brew | npm / PyPI | GitHub | Verdict |
|---|---|---|---|---|---|---|
| skillmirror | f | f | f | f / f | 27 exact, all unrelated (students, interview apps, top 2 stars) | recommended |
| skillgraft | f | f | f | f / f | 0 repos | strong, needs a one-line gloss |
| skillsync | f | f | f | T / f | 17; AlfonsSkills/SkillSync 27 stars, same job | generic, weaker |
| skill-manag | f | f | f | f / f | 0 exact | reads like a typo of taken `skill-manager` |
| skillrelay | f | f | f | f / f | 2; liangxiao1714/skillrelay 0 stars, same job | acceptable fallback |
| skillfan | f | f | f | f / f | 1, unrelated | "fan" is unclear |
| skillferry | f | f | f | f / T | 1; GreenLv/skillferry 0 stars, same job | metaphor unusual |
| skillcast | f | f | f | f / T | 13; super-rick/skillcast 1 star, same job | meaning blurry |
| sksync | f | f | f | f / f | 2; takemo101/sksync 2 stars, same job | cryptic |
| skillshelf | f | f | f | T / f | 26; Wang-Cankun/skillshelf 3 stars, same job | suggests a registry |
| skilltap | f | f | f | T / f | 5; nklisch/skilltap 6 stars, same job | no |
| skillweave | f | f | f | T / f | 17; LangeVC/skillweave 4 stars | suggests orchestration |
| skillkeeper | f | f | f | T / f | 8; lorem-dev/skillkeeper 6 stars, same job | no |
| skillvault | f | f | f | T / T | 25; huangluke89757/SkillVault 29 stars, same job | no |
| skillport | f | f | f | f / T | 22; gotalab/skillport 414 stars, same job | no |
| skillwarden | f | f | f | T / f | 2; both are security scanners | wrong meaning |
| vaultsync | f | f | f | f / f | 28; psimaker/vaultsync 148 stars (Obsidian) | "vault" means Obsidian or HashiCorp |
| driftless | f | f | f | T / T | 22; dbkaplun/driftless 85 stars (JS timers) | brand-only, not descriptive |
| skx | f | f | f | T / T | 0 exact | cryptic |
| skill-manager | T 0.9.0, same job | f | f | T / f | 6; buzhangsan/skill-manager 321 stars, same job | no |
| skillctl | T 0.2.0, same job | T (r3b1s/skillctl, same job) | f | T / T | 29; dvlshah/skillctl 5 stars, same job | no |
| skm | T 0.1.0, same job | T `skm`, `skm-bin` (SSH keys) | T (SSH keys) | T / T | TimothyYe/skm 1,087 stars; reorx/skm 78 stars, same job | no |
| skillhub | T 0.2.0 | f | f | T / T | iflytek/skillhub 5,154 stars | no |
| skillsmith | T 0.0.0 | f | f | T / T | 28; ChristopherKahler 158 stars, smith-horn 77 stars | no |
| skillet | T 0.6.3 (expression language) | f | f | not checked | many; crate `skillet-mcp` is in the same domain | no |

Binaries: none of the 25 names exists as a command on this machine or in the Arch file database (`which`, `pacman -F usr/bin/<n>`). Among short aliases tried, `sk` (skim) and `sm` (INN) do exist in Arch.

### Ranking (top 5)

| Rank | Name | Why | Against |
|---|---|---|---|
| 1 | skillmirror | literally what the README says ("a mirror, not an overlay"); free on all six registries; no same-job GitHub project; single lowercase word like the user's other Rust tool `refac` | 11 letters; "mirror" could suggest copying everything, but the opt-in rule is documented |
| 2 | skillgraft | graft = copy a master cutting onto many plants, matches vault to projects; the only name with zero GitHub repos | needs explanation; not self-describing |
| 3 | skillsync | most obvious word | same-job GitHub project, npm taken, undersells push, delete, doctor |
| 4 | skill-manag | smallest change from today's name | one letter from the taken crate `skill-manager`; the reason for the rename (looks odd) stays |
| 5 | skillrelay | free everywhere including npm and PyPI | a 0-star same-job repo; "relay" is vague |

Brand-style alternative that is not descriptive: `driftless` (matches the README tagline "zero drift"). Not recommended because the name says nothing about skills and two ecosystems already hold it.

### Recommendation

| Item | Value | Evidence |
|---|---|---|
| Crate and binary | `skillmirror` | crates.io 404, AUR, brew, repology, npm, PyPI free (check.sh run 2026-10-07) |
| Optional alias | `skmir` | free on crates.io, AUR, brew, repology, npm, PyPI; `pacman -F`, Debian contents and GitHub (0 exact repos) empty. Rejected aliases: `skm` taken, `skmr` (MinweiShen/skmr is a same-job tool), `sm` (INN binary), `smir` (rustc term) |
| GitHub repo | `jav-ed/skillmirror` | rename the existing repo; GitHub redirects issues, stars, `git clone`, `fetch`, `push`; it does not redirect Pages or Actions (docs.github.com, renaming a repository); the repo has 0 stars and 0 forks and no Pages or Actions |
| Internal crates (only if ever published) | `skillmirror-core`, `-tui`, `-web`, `-testkit` | all four are free on crates.io (404) |
| Config directory | `skillmirror` | see next section |

Do not ship the alias as a second `[[bin]]`: cargo-dist `[dist.bin-aliases]` creates it in installers (symlink for the shell installer), and the PKGBUILD can `ln -s`. A shell alias costs nothing until then.

Naming of non-code things: the user's improved convention (uppercase first letter for folders, `doc_Start.md` for non-code files) stays for repo folders and docs; ecosystem-facing names (crate, binary, repo, config directory, environment variables) stay lowercase and conventional, as for `refac` (`/home/jav/.../01_Refac/Refac_Cli_Code/Cargo.toml`: package `refac`, GitHub repo `ai_refac`). The folder `Project_Manag` is the doc-start convention and does not change. Keep the checkout folder `02_Skill_Manager`: it names the role, not the brand, and it keys per-project agent memory (the Claude memory path in this session contains the folder name) and is written into `Project_Manag/Docs/Setup/internal_Repo_Paths.md`. If it is renamed anyway, copy the memory folder and update that file in the same step. Put every product-specific on-disk or environment name behind one constant module (`Crates/Core/src/brand.rs`), so a future rename touches one file. Today the old name is spread over 25 files (17 Go files including `go.mod`, 7 docs, and `.gitignore`; `grep -rIl`).

## Config and environment migration

Old state (verified on this machine and in `cmd/root.go:61-84`, `README.md`):

| Item | Old | New |
|---|---|---|
| Vault pointer | `~/.config/skill_Manag/vault` (one line; present here) | `$XDG_CONFIG_HOME/skillmirror/vault` (`~/.config/skillmirror/vault`) |
| Vault config | `<vault>/config.yaml` (`root`, `mandatory`, `exclude_dirs`, `exclude_paths`) | unchanged name and keys |
| Environment | `SKILL_MANAG_VAULT`, `SKILL_MANAG_ROOT` (viper `AutomaticEnv` also reads any other `SKILL_MANAG_*`) | `SKILLMIRROR_VAULT`, `SKILLMIRROR_ROOT` |
| State and backups | none | `$XDG_STATE_HOME/skillmirror/` (undo backups, doctor reports) |
| Cache | none | `$XDG_CACHE_HOME/skillmirror/` (project registry) |
| Binary | `~/go/bin/skill_Manag` (installed here, 10.7 MB) | `skillmirror` |
| Dead code | `internal/config.go` documents `~/.config/skill_Manag/config.yaml`, never called (`grep LoadConfig`: one definition, no caller) | nothing to migrate |

Paths crate: with Linux only, `etcetera 0.11.0` (released 2025-10-28, 2 deps, XDG on Linux) or ten hand-written lines; `directories 6.0.0` and `dirs 7.0.0` pull `dirs-sys` and `option-ext` (MPL-2.0) (crates.io dependency API, 2026-10-07). cli_Stack makes the final call.

`<vault>/config.yaml` keeps its name: it contains no product name, lives in the user's vault repository (renaming rewrites vault history on every machine), and its keys are unchanged. Add no `version:` key now; unknown keys are already hard errors, and the first incompatible change introduces `version` with a migration. Open point: `root` and `exclude_paths` are absolute machine paths inside a git-tracked vault file (the real one holds `root: /home/jav/.../1_Code`); they belong in a per-machine file, but moving them is a separate decision.

Detection (core module `config::legacy`, runs before config loading for every command except `--help`, `--version`, `completions`, `man` and `migrate`; every failure below is a hard error with exit code 3, "runtime hard error" in cli_Stack's exit-code table):

| State | Behaviour |
|---|---|
| any `SKILL_MANAG_*` set | hard error naming each variable and its replacement; always an error, because silently ignoring an explicit override could sync the wrong vault; the tool cannot edit the parent shell, so it prints the exact replacement `export` lines and says "run `skillmirror migrate` for the checks" |
| old pointer exists, new pointer missing | hard error: `legacy configuration found at ~/.config/skill_Manag/vault; run: skillmirror migrate` |
| both pointers exist | no error and no reading of the old one; `doctor` warns that the legacy directory is still there and `migrate --retire` removes the warning |
| neither exists | ordinary "not configured" hard error pointing at the setup command |

`skillmirror migrate`: (1) read the old pointer; (2) check that the vault directory exists, is a git repository, and that `<vault>/config.yaml` passes the strict new parser (unknown or malformed keys stop it with line numbers; it never rewrites the file); (3) print the plan and ask once (`--yes`, `--dry-run` supported); (4) write the new pointer atomically (temporary file in the same directory, rename) and read it back; (5) leave the old directory alone, so the Go binary keeps working during the transition; (6) print the manual remainder: replacement `export` lines, and the path of any `skill_Manag` found by `which -a` (printed, never deleted). `migrate --retire` renames `~/.config/skill_Manag` to `~/.config/skill_Manag.migrated` (never deletes; refuses if that target exists). Idempotent: a second run prints "nothing to migrate" and exits 0. It never touches projects: the Go tool wrote no marker files into them, so project data needs no migration. Run it once per machine (the user also has a remote host, `mail.zetunweb.com`, per the remote-helper skill). Keep the detection until every machine has migrated (suggest two minor releases); after removal an old variable yields the normal "not configured" error, which is still a hard failure.

Symlinked agent folders (decision 2: `.agents/skills` canonical, `.claude/skills` and others are directory symlinks to it): the scan must never follow directory symlinks, otherwise each project is found twice; add a symlinked `.claude/skills` to the Testkit project builder and to the parity fixtures.

## Workspace layout

### Options

| Option | What it is |
|---|---|
| (a) | one package `skillmirror`, `src/lib.rs` plus `src/main.rs`, modules `core`, `cli`, `tui`, `web`, cargo features `tui`, `web` |
| (b) | virtual workspace `Crates/*`: `Core`, `Cli` (the only binary), `Tui`, `Web`, plus `Testkit` and `Xtask`; shared `[workspace.dependencies]`, `[workspace.lints]`, `default-members = ["Crates/Cli"]` |
| (c) | hybrid (ripgrep): root package with `src/` plus `crates/*` libraries (`Repos/ripgrep/Cargo.toml:1-60`: root `[package]` with `[workspace] members = ["crates/..."]`) |

Reference projects: virtual workspaces are the norm for multi-crate tools: release-plz (`Repos/release-plz/Cargo.toml`: `members = ["crates/*"]`, `[workspace.lints]`), uv (75 crates under `crates/`, virtual root, cargo-dist configured at the root), jj (virtual, `cli/` `lib/` `core/`). ripgrep is the hybrid.

### Measurements

Two prototypes generated from the same templates (`gen/gen.py`): 40 core modules with 8 serde-derive structs each plus blake3 and `ignore`, 12 clap modules, 14 ratatui modules, 10 axum modules; 10,384 and 10,382 lines of Rust. Debug profile unless stated, `CARGO_TARGET_DIR` per scenario, CPU seconds (user plus system) because the machine was shared with the other research agents (load average 17 to 38 on 12 cores) and wall times were 2 to 6 times inflated. Identical repeats vary by about 20 percent (cold CLI-only build: 90, 118, 116 CPU-s in (a); 116, 105, 114 in (b)), so differences below 25 percent are noise. Three runs are listed where taken.

| Scenario | (a) single package | (b) workspace |
|---|---|---|
| cold build, CLI only | 90 / 118 / 116 | 116 / 105 / 114 |
| cold build, default (CLI plus TUI) | 179 | 179 |
| cold build, everything (plus web) | 214 | 237 |
| cold release build, default | 361 | 371 |
| cold `cargo check`, default | 133 | 128 |
| cold test build of core only | 84 / 91 / 91 | 101 / 120 / 118 |
| no-op rebuild | 0.3 | 0.3 |
| rebuild after editing a core function | 7.0 / 7.7 / 7.3 | 5.1 / 6.7 / 7.3 |
| rebuild after editing a TUI file | 7.4 / 7.8 / 7.5 | 1.3 / 1.1 / 1.1 |
| rebuild after editing a CLI file | 7.2 / 7.6 / 7.1 | 0.9 / 0.9 / 0.8 |
| `cargo check` after a core edit | 2.3 / 2.2 / 3.6 | 5.0 / 5.2 / 4.9 |
| `cargo check` after a TUI edit | 3.9 / 4.6 / 4.7 | 0.7 / 0.7 / 0.7 |

Reading: cold costs are equal (the dependency compile dominates). In (a) every edit costs about 7 s because the library is one compilation unit; in (b) leaf-crate edits are 6 to 9 times cheaper in `build` and about 6 times in `check`. The workspace loses on `cargo check` after a core edit (every dependent crate is re-checked, 5 s against 2.3 s) and was 25 percent slower on the core-only test build (unexplained, probably noise). Rule of thumb: most edits happen in the CLI, TUI and later web code, so (b) wins on the common case. Caveat: synthetic code, one machine; the absolute numbers matter less than the 7 s versus 1 s shape.

Dependency direction and runtime isolation (experiments in `exp.sh`):
- (a): `core/m0.rs` calling `crate::tui::total()` compiles cleanly with default features; it only fails when someone builds `--no-default-features --features cli` (error E0433). The violation survives a normal build and needs `cargo-hack` over feature sets in CI to be found.
- (b): the same call in `Crates/Core` fails always: `error[E0433]: cannot find module or crate skillmirror_tui in this scope`.
- `cargo tree -e normal -i tokio` fails ("did not match any packages") for the default build in both layouts and finds tokio only through `axum` in `skillmirror-web` with `--features web`. In (b) `cargo tree --workspace` does contain tokio, so plain workspace commands compile it; `default-members = ["Crates/Cli"]` keeps plain `cargo build` and `cargo test` off it. Unique packages: core alone 37, CLI only 51, default 124, with web 167 (a: 50, 121, 163).

`cargo install` ergonomics (all run in the scratchpad):
- `cargo install --path .` on a virtual root fails: "found a virtual manifest ... instead of a package manifest". Use `cargo install --path Crates/Cli --locked` (works with the uppercase folder; checked) wrapped in `just install`.
- `cargo install --git <url>` works without a package name when only one package has a binary; I installed the prototype that way from a `file://` git repo (release build, CLI only, 102 s wall, binary ran).
- Publishing to crates.io needs every path dependency published first: `cargo publish -p skillmirror` fails with "no matching package named `skillmirror-core` found"; `cargo publish --workspace --dry-run` orders core, tui, web, cli itself (cargo 1.98). So (b) means four crate names and four versions on crates.io, (a) one. Because cli_Stack sets `publish = false`, this costs nothing now; a later crates.io release is the only point that favours (a) or (c).
- Features of the binary crate (`--features web`) work with `cargo install` and `dist` alike.

Verdict: (b), virtual workspace. (c) only if `cargo install --path .` at the root is worth an asymmetric tree; `just install` removes that need. (a) rejected: no boundary enforcement, 7 s rebuilds, feature combinations to police.

### Concrete tree

Folders at the repo root follow the improved convention (uppercase first letter); names dictated by Cargo and tools stay as they are. Cargo accepts uppercase member paths: `members = ["Crates/*"]` with `Crates/Cli` and package name `skillmirror` resolved correctly (`cargo metadata`, `cargo install --path Crates/Cli`, checked).

```
02_Skill_Manager/                      (repo root, folder name unchanged)
  Cargo.toml                           virtual: members Crates/*, default-members Crates/Cli,
                                       [workspace.package] edition 2024, rust-version, license-file = "LICENSE", publish = false,
                                       [workspace.dependencies], [workspace.lints]
  Cargo.lock  rust-toolchain.toml  rustfmt.toml  deny.toml  cliff.toml  dist-workspace.toml  justfile  hk.pkl
  .cargo/config.toml                   alias xtask = "run --package xtask --"
  .github/workflows/                   ci.yml (fmt, clippy, test, deny, loc gate), release.yml (generated by dist)
  Crates/
    Core/    package skillmirror-core. Sync only, no terminal, no async. No dependency on any sibling.
      src/ lib.rs  brand.rs  error.rs
        config/   paths.rs  pointer.rs  vault_config.rs  env.rs  legacy.rs  migrate.rs
        vault/    discover.rs  frontmatter.rs  groups.rs  tracked_files.rs
        scan/     walk.rs  prune.rs  registry.rs
        plan/     change.rs  diff.rs  hash.rs  lock.rs
        apply/    stage.rs  swap.rs  backup.rs  modes.rs
        ops/      sync.rs  push.rs  list.rs  delete.rs  add.rs  init.rs  status.rs  doctor.rs
        events/   event.rs  sink.rs
      tests/      public-API integration tests
    Tui/     package skillmirror-tui. ratatui and crossterm, depends on Core only. No tokio.
      src/ lib.rs  app.rs  router.rs  screens/{menu,sync,list,delete,push,setup,init,status,diff,groups}.rs
           widgets/  input/{mouse,keys}.rs
    Web/     package skillmirror-web. The only crate that may pull an async runtime. Depends on Core only.
      src/ lib.rs  server/  api/  security/  assets/ (embedded)
    Cli/     package skillmirror, binary skillmirror. Depends on Core, Tui (default feature), Web (optional feature).
      src/ main.rs (under 30 lines)  args/  commands/{sync,push,list,delete,add,init,status,doctor,migrate}.rs
           output/{human,json,progress}.rs  report.rs  exit.rs
      tests/ cli/  parity/             end-to-end and parity tests live here
    Testkit/ package skillmirror-testkit, publish = false. Builders for vault and project trees in tempdirs. Dev-dependency only.
    Xtask/   package xtask, publish = false, not a default member. Man pages (clap_mangen), schema output.
  Packaging/Aur/                       PKGBUILD for skillmirror-git now, skillmirror-bin after the first release
  Code/Development/                    repo tooling (existing Scratch/clean.sh) plus Quality/loc_gate.sh
  Project_Manag/  Repos/  Scratch/     unchanged (Repos and Scratch gitignored)
  cmd/  internal/  styles/  main.go  go.mod  go.sum     frozen Go oracle, deleted at cutover
```

Where things live:
- Dependency direction: Cargo manifests are the rule: Core depends on nothing of ours; Tui, Web, Testkit depend on Core; Cli depends on all three. Enforce with a `just check-deps` recipe (`cargo tree -p skillmirror-core -e normal -i tokio` must fail), and in `deny.toml` `[bans]` with `wrappers` if a stricter rule is wanted.
- Tests: unit tests in sibling `tests.rs` files (the 300-line gate counts code lines, see cli_Stack); integration tests per crate in `Crates/<X>/tests/`; end-to-end CLI and parity tests in `Crates/Cli/tests/` because cargo sets `CARGO_BIN_EXE_skillmirror` only for the package that owns the binary (cargo reference, environment variables), so a separate Parity crate cannot use it. cli_Stack puts the harness in `tests/parity/`; this is that folder under `Crates/Cli/`.
- Fixtures: generate trees in tempdirs (Testkit). Do not commit a folder literally named `.agents/skills` inside the repo: the real tool scans `1_Code`, so such a fixture gets synced into; the existing vault config proves it (its `exclude_paths` lists `.../02_Skill_Manager/internal/testdata`). Static samples keep a neutral name (`agents_skills`) and are renamed on copy.
- Automation: Just owns repository tasks (default-tools skill; `justfile` exists). `Xtask` exists only because man-page generation needs `clap_mangen` outside the shipped binary (cli_Stack); every other task is a Just recipe. cargo-dist's own repo is Just based (`Repos/cargo-dist/Justfile`), rust-analyzer uses xtask.
- 300 lines and one responsibility: a file is a verb or noun of the table above; `mod.rs` is a facade of `pub use` lines; use the 2018 style `foo.rs` plus `foo/` only when a module has children. cli_Stack measured that mature crates exceed 300 lines often (clap_builder 13 of 57 files), so this layout plans small modules from the start.

## Go to Rust transition

Verified by experiment: with `Cargo.toml`, `Crates/` and a `target/` folder beside `go.mod`, `go list ./...` still lists exactly the five Go packages and `go build` produced the oracle in 2.5 s; `cargo metadata` sees the four crates. `git archive HEAD cmd internal styles main.go go.mod go.sum | tar -x -C <dir>` followed by `go build` also works (2.1 s, 11.9 MB binary), so the oracle can be built from any tag with no worktree. The Go module is named `skill_Manag` (no host path), so no outside project can import it and no Go release needs support. The repo has no tags and 13 commits; `.git` is 7.1 MB and contains one 10.4 MB binary blob (`skill_manag`, committed in `98103af`); leave history alone, a rewrite gains little at this size.

Recommendation (one approach):
1. Rust at the repo root next to Go, Go untouched. No move of `cmd/`, `internal/`, `styles/`, `main.go`, `go.mod`, `go.sum`. Moving frozen files into a subfolder makes the oracle differ from `main` and costs a rename commit for no gain.
2. Freeze by tag, not by promise. cli_Stack found that a real sync is TUI-only in Go (`cmd/tui/sync.go:258`) and proposes a 40-line `cmd/oracle/main.go`. Put that file on a side branch from `c7310f9`, tag it `go-oracle`, and build the oracle from the tag (`git archive go-oracle ... | tar -x`, then `go build`) in `just parity-oracle`. The Go tree on `rust-rewrite` stays byte-identical to `main`, and the oracle stays buildable after Go is deleted.
3. Guard in CI and `hk`: `git diff --exit-code main -- cmd internal styles main.go go.mod go.sum` must be empty until cutover.
4. Parity run: `just parity` builds the oracle, builds `skillmirror --release`, runs the nextest profile `parity` in `Crates/Cli` (cli_Stack, "Differential parity test"). It fails hard if the oracle path is unset, no skip. The harness variable should use the new prefix (`SKILLMIRROR_GO_ORACLE`); the only place `SKILL_MANAG_` may remain is the legacy detector and its test. Parity runs in CI as a separate job, non-blocking until the Rust CLI covers dry-run, delete and sync apply, then blocking.
5. Cutover: parity green and `known_divergences.toml` reviewed; one commit "remove Go implementation (frozen at tag go-oracle)" deleting the six Go paths and the `internal/testdata` exclude from the vault config; merge `rust-rewrite` into `main` with `--no-ff` (keeps the 13 old commits and `git log -- cmd/` working); rename the GitHub repo and run `git remote set-url origin`; update `README.md`, `doc_Start.md`, `Descr/sync_Concept.md`, `Setup/internal_Repo_Paths.md` (every `skill_Manag` string; `grep` finds the old name in 25 files today); then on each machine `cargo install --path Crates/Cli --locked`, `skillmirror migrate`, `skillmirror migrate --retire`, remove `~/go/bin/skill_Manag`.
6. First Rust release is `0.1.0`; there is no earlier version number to continue (no tags).

## Distribution and release

Linux only. Targets: `x86_64-unknown-linux-musl` (static, runs on the glibc of any server the tool is copied to, which matters because the design goal is SSH use) and `x86_64-unknown-linux-gnu`; add `aarch64` linux gnu and musl only when a machine needs it. Unknown here: the architecture of the remote host. Out of scope, Linux only: Homebrew formula or tap, winget, scoop, PowerShell and MSI installers, macOS and Windows builds and their cross toolchains (`cargo-xwin`, Apple targets).

### Tools

| Tool | Version, date | Licence (crates.io) | Verdict |
|---|---|---|---|
| cargo-dist (`dist`) | 0.33.0 on GitHub 2026-09-10 (crates.io has 0.32.0 from 2026-05-22); pre-1.0, a release about every 3 months | MIT OR Apache-2.0 | yes: tag push builds archives, `sha256` files, a manifest and attestations, writes `release.yml`; works with a virtual workspace (uv uses it that way); set `precise-builds = true` so only `skillmirror` is built, not Web and Testkit (config.md:653-675); cross builds for aarch64 use cargo-zigbuild automatically (`book/src/ci/customizing.md:118-124`) |
| cargo-release | 1.1.6, 2026-09-16 | MIT OR Apache-2.0 | yes: local `cargo release minor` is a dry run until `--execute`; set `shared-version = true`, `tag-prefix = ""` (default for members is `{{crate_name}}-`, which would not match dist's `v1.2.3` tag) |
| git-cliff | 2.14.2, 2026-09-18 | MIT OR Apache-2.0 | yes: commits here are prose (`edc03df now alhamdulillah showing ...`), so set `filter_unconventional = false` with a catch-all parser (git-cliff.org, configuration/git) |
| release-plz | 0.3.169, 2026-09-19 | MIT OR Apache-2.0 | later or never: assumes Conventional Commits and makes a patch bump for anything else (`website/docs/changelog/format.md:21`); needs a personal token for tag-triggered workflows (`github/token.md`); `git_only = true` supports unpublished crates; revisit only if commits become conventional |
| cargo-binstall | 1.25.2, 2026-10-06 | GPL-3.0-only (a tool, see Licence consequences) | later: reads crates.io metadata and finds dist archives by URL pattern (cargo-dist `installers/other.md:6`); `--git` exists (`crates/bin/src/args.rs:121-127`); worth a metadata block only after a crates.io release |
| cargo-zigbuild | 0.23.4, 2026-09-02 | MIT | not installed (no zig), not measured; dist uses it for aarch64 |
| cross | crates.io 0.2.5 from 2023-02-04, GitHub pushed 2026-09-24 | MIT OR Apache-2.0 | no: needs Docker, stale on crates.io, dist makes it unnecessary |
| cargo-auditable, cargo-about, cargo-cyclonedx | 0.7.7, 0.9.2, 0.5.9 | MIT OR Apache-2.0, MIT OR Apache-2.0, Apache-2.0 | auditable: yes (one dist flag); about: yes before the first binary release (third-party notices); cyclonedx: later |
| taiki-e/upload-rust-binary-action | v1.30.2, 2026-04-17 | Apache-2.0 | fallback if dist's generated workflow becomes a burden (release-plz docs use it); has `checksum` and `tar` inputs |

### Channels

| Channel | Verdict | Notes |
|---|---|---|
| `just install` (`cargo install --path Crates/Cli --locked`) | now | the user's real channel |
| GitHub Releases via cargo-dist, musl and gnu tarballs, sha256, attestations | now | `github-attestations = true`: GitHub says attestations alone give SLSA v1.0 Build Level 2, Level 3 with reusable workflows, verified with `gh attestation verify` (docs.github.com, 2026-10-07); the repo is public and private repos are supported too. Do not add minisign or cosign now |
| `cargo install --git https://github.com/jav-ed/skillmirror` | now, free | works without a name (tested) |
| AUR source package `skillmirror-git` | now in `Packaging/Aur/`, publish later | pattern from the official Arch ripgrep PKGBUILD (`gitlab.archlinux.org/.../ripgrep/-/raw/main/PKGBUILD`): `cargo fetch --locked --target host-tuple`, `cargo build --release --frozen`, install binary, generate completions and man page with the built binary, install licence under `/usr/share/licenses/$pkgname`; `makepkg -si` works from the repo with no AUR account |
| AUR `skillmirror-bin` | after the first release | takes the musl tarball and dist's sha256; aliases by `ln -s` |
| Shell installer from dist (`curl ... \| sh`) | maybe | binary only, cannot detect musl (cargo-dist `installers/shell.md`: limitation 3); only for hosts without cargo or an AUR helper; skip until needed |
| cargo-binstall metadata | later | after crates.io |
| crates.io | optional, later | needs four crate names and `cargo publish --workspace`; publish real content, not a placeholder (crates.io policies page, name squatting rule, `svelte/src/routes/policies/+page.svelte:68`) |
| Nix flake | never for now | nixpkgs marks HL3 unfree; no Nix user |
| Docker, Flatpak, Snap, AppImage | never | a CLI that edits host files |
| Homebrew, winget, scoop, PowerShell, MSI | out of scope, Linux only | |

Completions and man pages: dist installers copy only the binary (`installers/shell.md`, "throw out all files except for the binary"), so they cannot install them. Ship `skillmirror completions <shell>` in the binary (cli_Stack), generate man pages through `Xtask`, let the PKGBUILD run both at package time, and put the man page into the release tarball with dist `include`. Third-party notices: generate with `cargo about` and `include` them; dist includes only `LICENSE*` and README automatically (`book/src/artifacts/archives.md:16`).

### Decide before the first release

1. Name (this file) and the `migrate` command shipped in the same release as the rename.
2. Licence declaration and third-party notices (next section).
3. Version policy: SemVer, workspace-wide single version, `0.x` means CLI flags and config may change in a minor release; tags `vX.Y.Z`; first release `0.1.0`.
4. Commit style: keep prose commits and a catch-all git-cliff config, or adopt Conventional Commit prefixes (only then does release-plz make sense).
5. Publish to crates.io: yes or no (changes crate naming and the `publish = false` setting).
6. Targets: musl only, or musl plus gnu; aarch64 needed or not.
7. Musl allocator: musl's allocator is known to be slower under multi-threaded allocation; measure the walk and plan phases on a musl build before making it the primary artifact (not measured here; `blake3` also needs the `pure` feature or a C toolchain for the musl target).

## Licence consequences

The user's LICENSE is the modular Hippocratic License 3.0 variant `HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR` (`LICENSE:4`). It is not OSI approved (SPDX list 3.29.0 of 2026-09-16: `isOsiApproved: false`) and has field-of-use and group restrictions.

### 1. crates.io and `cargo publish`

- SPDX has `Hippocratic-2.1` and `Hippocratic-3.0-core` (core text only, `seeAlso` firstdonoharm.dev/version/3/0/core.txt); there is no identifier for the user's module combination.
- crates.io validates the `license` field with the `spdx` crate pinned `=0.13.6` (`Cargo.toml:159`, `src/licenses.rs`), whose list is v3.29.0. I ran its exact parse mode: accepted `Hippocratic-3.0-core` and `LicenseRef-HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR`; rejected `Hippocratic-3.0` and `HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR` ("unknown term"). Production may lag the repository; not testable without publishing.
- Declaring `Hippocratic-3.0-core` would misstate the user's licence (core without the modules). Use `license-file = "LICENSE"` only: crates.io then stores the licence as "non-standard" (`src/controllers/krate/publish.rs:352-356`) and requires one of the two fields (`:324-325`). Crates such as `fractionfree`, `mof`, `crepes` do this today; `stampsmith` publishes `Hippocratic-2.1` as SPDX (crates.io API, 2026-10-07). The crates.io policies page has no open-source licence rule.
- Cargo side, tested: `license-file = "LICENSE"` in `[workspace.package]` with `license-file.workspace = true` in members resolves to `../../LICENSE`, and `cargo package` copies it into each crate; setting `license` and `license-file` together packages without a warning (cargo does not validate SPDX). Also needed for publishing: `description`, `repository`.

### 2. Channels that reject or restrict it

| Channel | Result | Source |
|---|---|---|
| Homebrew core | rejected: formulae must be "open source under a licence compatible with the Debian Free Software Guidelines"; none of 8,645 formulae lacks a licence, none is Hippocratic. A tap has no such rule but is out of scope | docs.brew.sh/Acceptable-Formulae; formulae.brew.sh JSON, 2026-10-07 |
| Debian main | not acceptable by inference: DFSG point 6, "must not restrict anyone from making use of the program in a specific field of endeavor", and point 5 on groups; OSI rejected Hippocratic on its equivalent points 5 and 6. I found no Debian ruling naming HL3, so this is a reading of the guidelines, not a legal opinion | debian.org/social_contract; OSI position only via secondary articles (mondaq, packtpub search results); SPDX JSON |
| Fedora | very likely not allowed (field-of-use restrictions are refused by Fedora legal in the threads found), not verified for HL3: docs.fedoraproject.org is blocked by Anubis for my fetcher (the licence-approval page loaded but does not mention field-of-use or Hippocratic) and I could not reach the Fedora licence data files | web search summary of lists.fedoraproject.org threads; unverified |
| AUR | accepted: any licence is allowed; AUR holds `google-chrome` (`custom:chrome`) and `visual-studio-code-bin` (`custom: commercial`). Custom licences use `LicenseRef-<name>` or `custom:<name>` and the text goes to `/usr/share/licenses/<pkg>` | AUR RPC 2026-10-07; Arch wiki "License" (search summary, direct fetch blocked) |
| cargo-binstall | no licence check; its support doc has no licence field; needs crates.io or `--git` | `SUPPORT.md` (0 matches for "licen"), `args.rs:121-127` |
| Nix | nixpkgs has `hl3` with `free = false; redistributable = true`, so users need `allowUnfree`; not planned anyway | `lib/licenses/licenses.nix:931-936` |
| GitHub | licence detected as "Other", SPDX `NOASSERTION` for `jav-ed/skill_Manag`; the picker lists 13 licences, none Hippocratic. Cosmetic; keep the README badge | `gh api repos/jav-ed/skill_Manag`, `gh api licenses` |
| cargo-dist installers | no licence gate: it reads `license` only to fill Homebrew and npm metadata and parses it with `spdx`; with `license-file` only the field stays empty (`cargo-dist/src/tasks.rs:1326`, `homebrew.rs:196`); archives auto-include `LICENSE*` | `Repos/cargo-dist` |

### 3. cargo-deny and the project's own licence

- Own crates: `publish = false` plus `[licenses.private] ignore = true` (cli_Stack deny sketch; cargo-deny docs, `private`). If crates are ever published: `[[licenses.clarify]]` with `expression = "LicenseRef-HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR"` and the hash of `LICENSE`, and the same `LicenseRef-...` string in `allow`; custom identifiers must start with `LicenseRef-` (cargo-deny `docs/src/checks/licenses/cfg.md:13`). cargo-deny 0.20.2 is not installed, not run.
- Dependencies: `cargo metadata` over my 217-package prototype graph gives the atoms MIT, Apache-2.0 (with LLVM-exception), Unlicense, Zlib, BSD-3-Clause, BSL-1.0, CC0-1.0, MIT-0, MPL-2.0, Unicode-3.0, Unicode-DFS-2016 and WTFPL. WTFPL comes from `terminfo 0.9.0`, which sits in the lock but has no path in `cargo tree --target all`, so it needs an allow entry only if cargo-deny reports it.
- Why permissive-only: HL3 adds use restrictions, which GPL-family terms forbid on top of their own, so the requirements doc is right to block GPL, AGPL and LGPL (this is a reading, not legal advice). MIT and Apache-2.0 code may be included under HL3, but binary releases must carry their notices: generate them with `cargo about` (Apache-2.0 OR MIT) and ship them.
- Repositories whose code cannot be copied (reading for ideas is fine): `beautyfree/skiller` (Sustainable Use License 1.0, `Repos/skiller/LICENSE`, non-transferable and non-sublicensable), and any `cargo-binstall` library crate (GPL-3.0). MIT or Apache-2.0 and so usable with notices: `openskills`, `ruler`, `rulesync`, `skillport`, `skillshare`, `vercel-labs/skills`, `agentskills`, `gitui`, `television`, `yazi`, `tiny-http`, `axum`, `ratatui` (`gh api repos/<r> .license`, 2026-10-07; GitHub reports `openskills` as NOASSERTION, but its `LICENSE` file and `package.json` say Apache-2.0).

### 4. Licences of the tools themselves

Build tools run as separate programs and do not pass their licence to our binary, so only what they copy into our output matters.

| Tool | Licence | Matters? |
|---|---|---|
| cargo-dist, release-plz, git-cliff, cargo-release, cargo-deny, cross | MIT OR Apache-2.0 (GitHub shows Apache-2.0 for the dual files) | no; generated `installer.sh` carries the header "Licensed under the MIT license" (`cargo-dist/templates/installer/installer.sh.j2:6-7`); keep it in the release |
| cargo-zigbuild, zig, mise, hk | MIT | no |
| just | CC0-1.0 | no |
| cargo-binstall | GPL-3.0-only | no as a tool the user runs; never use its library crates as dependencies |
| `swatinem/rust-cache` (GitHub Action) | LGPL-3.0 | no: not linked; dist emits it only with `cache-builds` (default off) |
| `actions/*`, `dtolnay/rust-toolchain`, `softprops/action-gh-release` | MIT | no |
| `taiki-e/*` actions, cargo-auditable, cargo-semver-checks, cargo-nextest | Apache-2.0 or dual | no |

Conclusion: nothing in the layout, release or tooling options is blocked or tainted by the licence choice; the cost is channel reach (no Homebrew core, Debian or likely Fedora) and the `license-file` declaration.

## Rejected and why

- `skill-manag`, `skillsync`, `skillport`, `skillvault`: reasons in the candidate table (typo-like, same-job competitors).
- Single package with features (a): no compile-time boundary, 7 s rebuilds for any edit (measured).
- Hybrid ripgrep layout (c): asymmetric root; only gains `cargo install --path .`.
- Moving Go into a subfolder: breaks "frozen and identical to main" for no benefit.
- release-plz now: conventional-commit assumption does not match the commit history.
- cross, Docker builds: dist's zigbuild path covers aarch64, Linux only.
- Shipping the alias as a second binary: doubles link work and size; installers can symlink.
- Renaming `02_Skill_Manager` or `Project_Manag`: path-keyed agent memory and the doc-start convention.
- `Hippocratic-3.0-core` as the SPDX field: accepted by crates.io's parser but misdescribes the modular licence.

## Risks and open points

- Name: GitHub has many same-word projects; no trademark found, but I did not run a trademark database search. The Sitra "skills mirror" exercise (a Finnish HR tool) is an unrelated phrase.
- Measurements: shared machine, synthetic code, one repeat for most cold runs; the unexplained 25 percent on the core test build in (b).
- crates.io acceptance of the `LicenseRef` form is untested (cannot publish); `license-file` only is the safe route.
- cargo-dist is pre-1.0 and generates workflow files that must be regenerated on upgrade (`dist generate`, `allow-dirty` setting in uv's config); crates.io has 0.32.0 while GitHub has 0.33.0.
- Fedora statement unverified; Debian and Fedora are not targets, so this does not block anything.
- `root` in the vault `config.yaml` is machine specific and committed; separate decision.
- The remote host's architecture and glibc are unknown; they decide whether musl is mandatory.
- `panic = "abort"` and other profile choices belong to cli_Stack; the release profile `dist` is created by `dist init`.

## Related files

- [Requirements](rewrite_Requirements.md): the constraints this file is judged against (Linux only, licence, 300-line rule, naming).
- [CLI and quality stack](cli_Stack.md): config paths, `deny.toml` sketch, exit-code table, parity harness, the 300-line gate, Just and hk wiring.
- [Prior art](prior_Art.md): existing skill managers; the in-purpose tools named in the Name section.

## Clones

Made by me under `Repos/` (gitignored; the lead records them in `repos_List.md`): `Repos/cargo-dist` (https://github.com/axodotdev/cargo-dist, commit d1055f4, 2026-09-04), `Repos/release-plz` (https://github.com/release-plz/release-plz, 7a93d23, 2026-10-07), `Repos/ripgrep` (https://github.com/BurntSushi/ripgrep, 15.2.0 layout). Read through `gh api` without cloning: uv, jj, rust-analyzer, crates.io, cargo-binstall, cargo-deny, spdx, nixpkgs. Reused existing clones only to read licences: `Repos/skiller`, `Repos/openskills`.

## Verdicts

| Option | Verdict | Reason | Date checked |
|---|---|---|---|
| Name `skillmirror` | yes | free on six registries, no same-job GitHub project | 2026-10-07 |
| Name `skillgraft` | maybe | most unique, least descriptive | 2026-10-07 |
| Name `skillsync` | maybe | obvious but same-job project and npm taken | 2026-10-07 |
| Name `skill-manag` | no | looks like a typo of a taken crate | 2026-10-07 |
| Alias `skmir` | maybe | free everywhere; installer symlink only | 2026-10-07 |
| Virtual workspace under `Crates/` | yes | 6 to 9 times cheaper leaf rebuilds, compile-time boundaries | 2026-10-07 |
| Single package with features | no | no boundary, 7 s rebuilds | 2026-10-07 |
| Hybrid root package | maybe | only if root `cargo install --path .` matters | 2026-10-07 |
| `etcetera` for paths | maybe | 2 deps, XDG; cli_Stack decides | 2026-10-07 |
| `migrate` plus legacy detection | yes | hard error, one command, idempotent | 2026-10-07 |
| Keep `<vault>/config.yaml` name | yes | no product name in it | 2026-10-07 |
| Go beside Rust at root, tagged oracle | yes | verified coexistence, oracle builds in 2 s | 2026-10-07 |
| cargo-dist 0.33.0 | yes | Linux tarballs, checksums, attestations | 2026-10-07 |
| cargo-release 1.1.6 | yes | local, manual bump and tag | 2026-10-07 |
| git-cliff 2.14.2 | yes | handles prose commits with `filter_unconventional = false` | 2026-10-07 |
| release-plz 0.3.169 | no | assumes conventional commits | 2026-10-07 |
| cargo-binstall support | maybe | only after crates.io release | 2026-10-07 |
| cross | no | Docker, stale crates.io release | 2026-10-07 |
| cargo-zigbuild | maybe | used by dist for aarch64; not installed | 2026-10-07 |
| AUR source and `-bin` | yes | pattern verified on official PKGBUILDs; AUR accepts any licence | 2026-10-07 |
| crates.io publish | maybe | works with `license-file`; four crate names | 2026-10-07 |
| Shell installer | maybe | binary only, no musl detection | 2026-10-07 |
| Nix flake | no | HL3 unfree in nixpkgs, no user | 2026-10-07 |
| Homebrew, winget, scoop, PowerShell, MSI, macOS, Windows | no | out of scope, Linux only | 2026-10-07 |
| `license-file = "LICENSE"` | yes | the accurate declaration for the modular HL3 | 2026-10-07 |
