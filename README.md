# skillmirror

> Work in progress. The Rust rewrite is on the branch `rust-rewrite-handoff`; the Go tool it replaces (`skill_Manag`) is still on `main` until the cutover.

A command line tool and a full-screen interface that keep your agent skills in sync across all your projects: one vault, zero drift.

Built by [javedab.com](https://javedab.com). Get in touch if you want help with your tooling.

---

## The problem

Agent skills live in `.agents/skills/<name>/` inside each project. When you maintain several projects, the same skill files end up copied everywhere. The moment you improve a skill in one place, every other project is out of date.

Symlinks would solve this, but they break over SSH and are not tracked properly in git.

## The solution

`skillmirror` knows two things: your **vault** (one git folder where you write and maintain your skills) and your **root** (the folder that contains all your projects).

It walks every project under the root, finds every `.agents/skills/<name>/` folder, and for any skill that also exists in the vault, replaces the project copy with the vault version. The copy is a mirror, not an overlay: the folder is swapped as a whole, so no stale file of an older version survives. Files are copied, never linked, so projects stay git-tracked and work over SSH.

**Key rule: `sync` only updates skills a project already has.** It never installs a skill into a project that has not opted in. Each project controls its own skill set by what it has in `.agents/skills/`. Installing something new is a separate, explicit act: `add`, `init` or `push`.

```
vault/
  coding/        <- your master copy
  doc-start/
  web/
    astro/       <- a group: a folder of skills, only in the vault

projects/
  project-A/
    .agents/skills/
      coding/    <- exists, so it is updated from the vault
                    doc-start is not here, so it is NOT touched
  project-B/
    .agents/skills/
      astro/     <- exists, so it is updated from the vault
```

## Install

Needs Rust 1.88 or newer and `git`. Linux only.

```bash
git clone git@github.com:jav-ed/skill_Manag.git
cd skill_Manag
git checkout rust-rewrite-handoff
just install            # or: cargo install --path Crates/Cli --locked
```

The binary lands in `~/.cargo/bin/skillmirror`. Check it with `skillmirror doctor`. The compile takes about a minute and up to about 600 MB of memory; the build folder is removed afterwards (see [Development](#development) for the cost of working on the code).

First run: `skillmirror` with no arguments opens the interface, and its Setup entry asks for the vault and the root and writes the configuration. Starting from nothing, `skillmirror vault init ~/skills --root ~/projects` makes the vault, `skillmirror new NAME` or `skillmirror adopt NAME --from PROJECT` fills it, and you commit in the vault when a skill is ready. Coming from the Go tool: `skillmirror migrate` copies its vault pointer.

## Commands

Every command accepts `--vault <DIR>` and `--root <DIR>`. Commands that write ask first, unless `--yes`; they show a plan with `--dry-run`. Commands that print results take `--json` for scripts.

| Command | What it does |
|---|---|
| `skillmirror` | Opens the interface when stdin and stdout are terminals |
| `sync [SKILL...] [--group P] [--profile N] [--project DIR] [--dry-run] [--check] [--yes] [--json] [--all]` | Updates the skills each project already has; never adds one. Names, `--group`, `--profile` and `--project` limit it to some skills or one project. `--check` writes nothing and exits 1 when something differs |
| `push [SKILL...] [--project DIR]` | Installs the vault's mandatory skills into every project that has a skills folder, or only the named ones, or only one project |
| `add [SKILL...] [--group PATH] [--profile NAME] [--project DIR]` | Installs skills, whole vault folders or profiles into one existing project |
| `init DIR [SKILL...] [--group] [--profile] [--git] [--no-mandatory]` | Makes a new project folder with the mandatory skills plus a selection |
| `delete NAME [--project DIR]` | Removes one skill from every project, or from one |
| `list` | Every installed skill folder, with the ones the vault lacks marked |
| `skills [--group PATH]` | The vault as a tree of groups |
| `info SKILL [--json]` | One skill from every side: group, description, the files that are copied and the ones git does not track, the profiles that name it, and what it is in each project |
| `status [SKILL...] [--group P] [--project DIR] [--all]` | How every project stands against the vault: outdated, mandatory missing, not in the vault. Exit 1 when something differs |
| `diff [SKILL] [--project DIR] [--stat]` | The lines a sync would bring in and take away, as unified diffs |
| `report [-o FILE] [--open]` | One self-contained HTML page: skills against projects, the vault tree, diffs, a filter, dark mode |
| `new NAME [--group PATH] [--description TEXT] [--dry-run]` | Creates a skill in the vault from a template (`SKILL.md` with a header), staged in git, not committed |
| `adopt NAME --from PROJECT [--group PATH] [--dry-run]` | Copies a skill folder that a project already has into the vault, staged, not committed; the project's folder is then in sync |
| `vault init DIR [--root DIR] [--use] [--dry-run]` | Makes a new vault (a git repository with a `config.yaml`) and, when you have no default vault yet, makes it the default |
| `config show [--json]`, `config path`, `config root DIR` | Shows the vault, the scan root (with where each came from) and the settings; prints the path of `config.yaml`; sets `root:` there |
| `mandatory list`, `mandatory add SKILL...`, `mandatory remove SKILL...` | Shows or edits the skills that `push` installs everywhere, in `config.yaml` (comments and other keys stay) |
| `bridge [--dry-run]` | Links other agent folders such as `.claude/skills` to `.agents/skills` (see `targets` below) |
| `doctor` | Checks the machine, the configuration, the vault and every `SKILL.md` header; writes nothing |
| `undo [RUN]` and `history` | Brings back what a run replaced or removed; undoing is a run too, so a second `undo` redoes it |
| `migrate`, `completions SHELL` | Copies the old tool's vault pointer; prints a shell completion script |

Exit codes: `0` done or nothing differs, `1` drift found (`--check`, `status`, `diff`, a dry run of `bridge`), `2` the command line cannot be carried out as given, `3` a hard error (bad configuration, missing vault, unreadable root), `4` the command ran but some targets failed. A failure in one project never stops the others and never leaves a half written skill folder.

## The interface

`skillmirror` with no arguments opens a full-screen interface with full mouse and keyboard navigation. `?` shows the keys of the current screen, `q` or `alt+left` goes back, `ctrl+c` quits.

| Entry | What it does |
|---|---|
| Sync, Push | Pick skills, see what would be written (`v` shows the changes as a diff), confirm, run |
| List | Every installed skill; `/` filters, space selects, `s` syncs and `d` deletes the selection in place |
| Skills | The vault's skills with a detail card for the one under the cursor: description, files, profiles, and what it is in each project. Read-only; `/` filters |
| Delete | Nothing is pre-selected; you pick, then confirm |
| Add | Pick the project folder, then vault skills to install into it |
| Init | Pick a parent folder, name the new project, tick skills (the mandatory ones are ticked), `g` for a git repository |
| History | The backup runs, newest first; pick one to undo it |
| Setup | Pick the vault and the root, tick the mandatory skills, save |

Every page that works out a plan shows it before anything is written. If the scan could not read something, the heading says so and `i` lists it.

## Configuration

The configuration is split in two so the vault is self-contained and portable:

```
~/.config/skillmirror/vault   <- one line: the path to your vault
<vault>/config.yaml           <- everything else
```

```yaml
# <vault>/config.yaml
root: /path/to/your/projects
mandatory: [coding, doc-start]      # what `push` installs; optional
exclude_paths:                      # skipped with everything below them
  - /path/to/project/testdata
exclude_dirs: [testdata]            # skipped wherever the name occurs; optional
targets: [claude]                   # other agent folders to link; optional
profiles:                           # named selections for add and init; optional
  websites:
    description: Everything for a website
    groups: [web]
    skills: [seo-checklist]
  site:
    extends: [websites]
    exclude: [astro]
```

Precedence, highest first: `--vault` and `--root`, the environment variables `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT`, then the files above. The old `SKILL_MANAG_*` variables are refused with a message that names their replacement. Unknown keys in `config.yaml` are an error, not ignored.

Built-in skips apply on top: `.git`, `node_modules`, `vendor`, `dist`, `build`, `out`, `target`, `.next`, `.nuxt`, `.venv`, `__pycache__`, `.tox`, `.pytest_cache`, `.cache`, `.turbo`, `.parcel-cache`.

### Groups and profiles

A **group** is a folder in the vault that holds skills. Groups only exist in the vault: the installed layout stays flat (`.agents/skills/<name>/`), so moving a skill between groups changes no project. `skillmirror skills` shows the tree; `add --group web/seo` installs everything below a folder. A **profile** names a selection of groups and skills, can extend other profiles and can exclude names.

### What travels with a skill

The vault must be a git repository. The files git tracks in a skill folder are what is copied, with their permission bits; untracked files never are, so a half finished edit does not leak out. A tracked symlink or submodule inside a skill, or a skill whose `SKILL.md` is not tracked, makes that skill fail with a hint instead of copying something wrong.

### Targets

`targets: [claude]` asks for `<project>/.claude/skills` to be a relative link to `../.agents/skills`, so Claude Code sees the same skills. `skillmirror bridge` makes the links; `add` and `init` make them for their project. A link is made only where nothing is: a real folder, a link that leads elsewhere and a linked `.claude` are reported and left alone. `status` and `doctor` tell you about a missing link.

## Safety

- A skill folder is replaced by building the new copy beside it and swapping the two folders in one step (`renameat2` exchange), so a crash never leaves a half written skill. Filesystems that cannot do this (NFS, CIFS, FUSE, FAT) are named by `doctor`.
- Before a run replaces or removes a folder, the old copy goes to `~/.local/state/skillmirror/backups/<run>/`; the newest 30 runs are kept. `history` lists them and `undo` brings one back.
- If a project folder changed between the plan and the write, that skill fails instead of being overwritten.
- Links are never followed and never written through. Only the commands that grow the vault write into it (`new`, `adopt`, `vault init`, and the configuration file the Setup screen saves): they create files that did not exist, stage them with `git add`, and never overwrite, delete or commit anything.
- Everything the scan could not read, and every leftover of an interrupted run, is reported, never hidden.

## Development

```bash
just check      # format, clippy, tests, file size (300 code lines), core stays synchronous
just deny       # advisories, licences, bans
just parity     # replay 131 scenarios of the old Go tool against this one (needs the Go oracle)
```

**Building Rust is heavy.** A clean build of the workspace with Cargo's defaults uses 1.9 GB of disk and around 800 MB of extra memory, and a `target/` folder that has been used for a few days reaches 10 GB or more (debug information and the incremental cache). This repository turns both off in the root `Cargo.toml` (0.47 GB, about 500 MB, 27 s on four cores). On a small machine add `CARGO_BUILD_JOBS=2`; `cargo clean` frees everything. Numbers, what to switch back on for a debugger, and the rest: [`build_Resources.md`](Project_Manag/Docs/Setup/build_Resources.md).

Crates: `Crates/Core` (the engine, no async), `Crates/Cli`, `Crates/Tui`, `Crates/Web` (the HTML report), `Crates/Testkit`. The architecture, the behaviour contract (one numbered row per rule) and the decisions behind them are under `Project_Manag/Docs/`; start at [`doc_Start.md`](Project_Manag/Docs/doc_Start.md).

## License

[![Hippocratic License HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR](https://img.shields.io/static/v1?label=Hippocratic%20License&message=HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR&labelColor=5e2751&color=bc8c3d)](https://firstdonoharm.dev/version/3/0/bds-cl-eco-extr-media-mil-sv-xuar.html)

See [LICENSE](./LICENSE) for the full terms.
