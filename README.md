# skillmirror

> This is the Rust version of the tool. The Go tool it replaced (`skill_Manag`) is in the history at commit `c7310f9`.

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
  coding/        <- a skill: a folder with a SKILL.md, your master copy
  doc-start/
  Web_Skills/    <- a group: a folder without SKILL.md that holds skills, only in the vault
    astro/
    seo/         <- groups nest as deep as you like
      schema/

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

Needs Rust 1.89 or newer and `git`. Linux only.

```bash
git clone git@github.com:jav-ed/skill_Manag.git
cd skill_Manag
just install            # or: cargo install --path Crates/Cli --locked
just install-web        # the same with the web view built in (see below); or add --features web
```

The binary lands in `~/.cargo/bin/skillmirror`. Check it with `skillmirror doctor`. Tab completion is **not** on after the install, see the next section. The compile takes about a minute and a quarter and, with all cores busy, peaks near 0.9 GB of memory (about 0.5 GB and four minutes with `CARGO_BUILD_JOBS=1`); the build folder is removed afterwards (see [Development](#development) for the cost of working on the code).

### Tab completion

The install does not turn tab completion on: your shell has to be told once, with one line in its startup file. Until then `skillmirror sy` plus Tab does nothing.

| Shell | Add this line to | After |
|---|---|---|
| zsh | `~/.zshrc` | the line that starts oh-my-zsh, or the one that runs `compinit` (it must come first) |
| bash | `~/.bashrc` | anywhere |
| fish | `~/.config/fish/config.fish` | anywhere |

```bash
source <(skillmirror completions zsh)       # zsh
source <(skillmirror completions bash)      # bash
skillmirror completions fish | source       # fish
```

Open a new terminal and try `skillmirror sy` plus Tab. The line runs the installed binary each time a shell starts (a few milliseconds), so the completion always matches the version you have; there is no file to refresh after an upgrade. Commands, options and options with a fixed set of values are completed. Skill names, groups and projects are not completed yet. `elvish` and `powershell` are supported by `skillmirror completions SHELL` too.

First run: `skillmirror` with no arguments opens the interface, and its Setup entry asks for the vault and the root and writes the configuration. Starting from nothing, `skillmirror vault init ~/skills --root ~/projects` makes the vault, `skillmirror new NAME` or `skillmirror adopt NAME --from PROJECT` fills it, and you commit in the vault when a skill is ready. Coming from the Go tool: `skillmirror migrate` copies its vault pointer.

## Commands

Every command accepts `--vault <DIR>` and `--root <DIR>`. Commands that write ask first, unless `--yes`; they show a plan with `--dry-run`. Commands that print results take `--json` for scripts.

| Command | What it does |
|---|---|
| `skillmirror` | Opens the interface when stdin and stdout are terminals |
| `sync [SKILL...] [--group P] [--profile N] [--project DIR] [--dry-run] [--check] [--yes] [--json] [--all]` | Updates the skills each project already has; never adds one. Names, `--group`, `--profile` and `--project` limit it to some skills or one project. `--check` writes nothing and exits 1 when something differs |
| `push [SKILL...] [--project DIR]` | Installs the vault's mandatory skills into every project that has a skills folder, or only the named ones, or only one project |
| `add [SKILL...] [--group PATH] [--profile NAME] [--project DIR]` | Installs skills, whole vault folders or profiles into one existing project |
| `init DIR [SKILL...] [--group] [--profile] [--no-git] [--no-scaffold] [--no-mandatory] [--no-agents-md]` | Makes a new project: a git repository on `main`, the mandatory skills plus a selection, an `AGENTS.md` (see [AGENTS.md](#agentsmd)) and the doc-start scaffold (see [A new project](#a-new-project)) |
| `agents status [--project DIR] [--all] [--json]` | How the `AGENTS.md` of every project stands against the text: current, outdated, edited by hand, missing, without a block. Writes nothing; exit 1 when any is not current |
| `agents sync [--project DIR] [--force] [--dry-run] [--diff] [--yes]` | Rewrites the autogenerated block of the `AGENTS.md` files where the text changed; the rest of each file is left alone |
| `agents add [--project DIR] [--dry-run] [--yes]` | Makes `AGENTS.md` in a project that has none, or puts the block in front of a file that has none |
| `agents seed [--dry-run] [--json]` | Writes the built-in text to `<vault>/project-files/AGENTS.md` so you can edit it; stops if the file is there |
| `delete NAME [--project DIR]` | Removes one skill from every project, or from one |
| `list` | Every installed skill folder, with the ones the vault lacks marked |
| `skills [--group PATH]` | The vault as a tree of groups |
| `info SKILL [--json]` | One skill from every side: group, description, the files that are copied and the ones git does not track, the profiles that name it, and what it is in each project |
| `status [SKILL...] [--group P] [--project DIR] [--all]` | How every project stands against the vault: outdated, mandatory missing, not in the vault. Exit 1 when something differs |
| `diff [SKILL] [--project DIR] [--stat]` | The lines a sync would bring in and take away, as unified diffs |
| `report [-o FILE] [--open]` | One self-contained HTML page: skills against projects, the vault tree, diffs, a filter, dark mode |
| `web [--allow-write] [--port N] [--open] [--idle-timeout MINUTES]` | The same views live in your browser, served on 127.0.0.1 only behind a link that works once. With `--allow-write` the pages can also run sync, push and undo, each after showing its plan with the diffs |
| `new NAME [--group PATH] [--description TEXT] [--dry-run]` | Creates a skill in the vault from a template (`SKILL.md` with a header), staged in git, not committed |
| `adopt NAME --from PROJECT [--group PATH] [--dry-run]` | Copies a skill folder that a project already has into the vault, staged, not committed; the project's folder is then in sync |
| `changes [--project DIR] [--all] [--json]` | Which skills of one project differ from the vault, read-only. The project is the one around the current folder (up to the top of its git repository), else the only one below it; `--project` names it. Each skill is `edited` here, `behind` (an older version the vault had, found in the vault's git history: sync brings it up), `new` (not in the vault: adopt), `unusable` or `same`. Exit 1 when some skill is edited |
| `promote SKILL... \| --all [--project DIR] [--sync] [--dry-run] [-y]` | Replaces the vault's copy of skills you changed in a project with the project's, after showing what differs. The old copy goes to the backup store (`undo` brings it back), the new one is staged in git and not committed. Refused: a skill that is not edited, and a vault skill with unstaged changes. `--sync` then updates every other project that has the skills, from the vault, with the usual sync question |
| `vault init DIR [--root DIR] [--use] [--dry-run]` | Makes a new vault (a git repository with a `config.yaml`) and, when you have no default vault yet, makes it the default |
| `config show [--json]`, `config path`, `config root DIR`, `config web on\|off` | Shows the vault, the scan root (with where each came from) and the settings; prints the path of `config.yaml`; sets `root:` there; turns the web view on or off (`web:` there) |
| `mandatory list`, `mandatory add SKILL...`, `mandatory remove SKILL...` | Shows or edits the skills that `push` installs everywhere, in `config.yaml` (comments and other keys stay) |
| `bridge [--dry-run]` | Links other agent folders such as `.claude/skills` to `.agents/skills` (see `targets` below) |
| `doctor` | Checks the machine, the configuration, the vault, every `SKILL.md` header and the AGENTS.md text; writes nothing |
| `undo [RUN]` and `history` | Brings back what a run replaced or removed; undoing is a run too, so a second `undo` redoes it |
| `migrate`, `completions SHELL` | Copies the old tool's vault pointer; prints a shell completion script (see [Tab completion](#tab-completion)) |

Exit codes: `0` done or nothing differs, `1` drift found (`--check`, `status`, `agents status`, `diff`, a dry run of `bridge`), `2` the command line cannot be carried out as given, `3` a hard error (bad configuration, missing vault, unreadable root), `4` the command ran but some targets failed. A failure in one project never stops the others and never leaves a half written skill folder.

## A new project

`skillmirror init DIR` makes the folder and then, in this order:

1. **A git repository on the branch `main`**, with no remote. When `DIR` is inside a repository already, none is made, so there is never a repository inside a repository. `--no-git` skips it.
2. **The skills** (the mandatory ones plus your selection) and the `AGENTS.md`.
3. **The scaffold**, made by the project's own copy of the doc-start skill: skillmirror runs its `Code/bootstrap.sh`, which makes `Project_Manag/Docs`, `Scratch/` with its folders, `Repos/`, the `.gitignore` lines for both and the `just scratch-clean` recipe. The skill stays the one place that says what a new project looks like; change it there and every new project follows. `--no-scaffold` skips it.

The scaffold needs the doc-start skill among the installed skills and its scripts tracked in the vault. Without them `init` stops with an error before it makes anything; make the skill mandatory (`skillmirror mandatory add doc-start`) or pass `--no-scaffold`. If the script itself fails, the skills stay, the exit code is `4` and the message names the command to run again. `undo` takes back the skills and the `AGENTS.md` of the run; the folder, the repository and the scaffold stay.

## AGENTS.md

Every new project gets an `AGENTS.md` next to its skills, so an agent that opens the project reads your rules first. The file has two parts. The first is a block that skillmirror owns: it is marked as autogenerated and ends with a note that says, in plain words, to change only what comes below it. Everything below the block is the project's own, because every project is different; skillmirror never touches it:

```
<!-- skillmirror:autogenerated begin checksum=0123456789abcdef; do not edit inside this block, ... -->
# Coding related
1. If you start fresh, read the doc start skills (`doc-start`). ...

---

*Managed by skillmirror. Everything above this line can be updated automatically, so please change only what comes below it.*
<!-- skillmirror:autogenerated end -->

## Notes for this project
(yours, edit freely)
```

The text of the block lives in your vault, in a folder for files that go into every project: `<vault>/project-files/AGENTS.md`. The folder holds no `SKILL.md`, so `sync` and `push` never take it for a skill, and a file called `AGENTS.md` at the root of the vault is the vault's own and is never read. Without a file there, five built-in rules for coding agents are used. `agents seed` writes the built-in text into the vault so you can edit it (it never overwrites a file that is there). To change the rules everywhere, edit that file, commit it in the vault and run:

```bash
skillmirror agents seed       # once: write the built-in text to <vault>/project-files/AGENTS.md, to edit it
skillmirror agents status     # which projects are current, outdated, missing, edited by hand
skillmirror agents sync       # rewrite the block where it is out of date (plan first, asks, keeps a backup)
skillmirror agents add        # in an existing project: make the file, or put the block in front of one that has none
```

`status` adds a line when blocks are out of date or were edited by hand, and `doctor` checks that the text can be used. The checksum in the begin line is how an out of date block (the text changed) is told from one somebody edited by hand. A block edited by hand is not replaced: `agents sync` stops for that project and `--force` replaces it (the old file is kept, so `undo` brings it back). A file is written like a skill is: built beside the old one, swapped in at once, an edit made while the plan waited survives, never through a link, line ends and a byte order mark stay. The interface has an Agents entry for the same job, and `m` on its Init page leaves the file out.

The built-in text names the skills `doc-start`, `coding` and `file-tree-optimization`, so `init` and `agents add` stop with an error when the project would not have them (make them mandatory with `skillmirror mandatory add ...`, or use `init --no-agents-md`). A text of your own in the vault is not checked. The web view does not show `AGENTS.md` yet.

## The interface

`skillmirror` with no arguments opens a full-screen interface with full mouse and keyboard navigation. `?` opens a help for the current screen: its keys in topics (move, choose, find, the pane, run, general), each with what it is good for. `q` or `alt+left` goes back, `ctrl+c` quits.

| Entry | What it does |
|---|---|
| Override | Opens on the project around the folder you started in (or lists the ones below it to pick from) and shows its skills against the vault: edited, behind, new. Nothing is read before the entry is opened. Space ticks an edited skill, `a` all of them, `s` also updates the other projects that have them; Enter asks, then replaces the vault's copy. The old copy is kept for History |
| Sync, Push | Pick skills, see what would be written (`v` shows the changes as a diff), confirm, run. A pane beside the list (under it on a terminal narrower than 100 columns) follows the cursor and names the projects of the skill: `tab` switches it between the projects that would change, the others, and all of them; `p` (or a click on `↔`) writes them short (`parent/project`) or long (the path below your scan root); `f` filters them; `ctrl+d` and `ctrl+u` scroll it |
| List | Every installed skill; `/` filters, space selects, `s` syncs and `d` deletes the selection in place |
| Skills | The vault's skills with a detail card for the one under the cursor: description, files, profiles, and what it is in each project. Read-only; `/` filters |
| Delete | Nothing is pre-selected; you pick, then confirm. The pane names every project the skill would be removed from |
| Agents | Every project with the state of its `AGENTS.md`; the ones whose block is out of date are ticked. Pick, read the changes with `v`, confirm; only the block is written |
| Add | Pick the project folder, then vault skills to install into it |
| Init | Pick a parent folder, name the new project, tick skills (the mandatory ones are ticked). `g` leaves the git repository out, `m` the `AGENTS.md`, `d` the scaffold |
| History | The backup runs, newest first; pick one to undo it |
| Setup | Pick the vault and the root, tick the mandatory skills, save |

Every page that works out a plan shows it before anything is written. If the scan could not read something, the heading says so and `i` lists it.

## The web view

The web view is **off by default**, and two switches keep it off:

- **The build.** A plain `just install` leaves out its server (`axum`, `tokio`). Build it in with `just install-web`, or `cargo install --path Crates/Cli --locked --features web`.
- **The config.** `web: false` in `<vault>/config.yaml` (what you get without the key) makes `skillmirror web` refuse to start. `skillmirror config web on` sets `web: true`, `skillmirror config web off` sets it back.

`skillmirror web` starts only when both are on. When one is missing, it stops with a usage error (exit 2) that names what to do, and nothing is listening.

`skillmirror web` prints a link and serves on `127.0.0.1` (a free port, or `--port`) until Ctrl-C or until nobody has asked for anything for two hours (`--idle-timeout`). The pages: an overview of every project, Sync and Push (pick skills and a project, see the plan with its diffs, apply), History (undo a run), Doctor, Settings, and the full report. Without `--allow-write` it only looks: no page can change a file. With it, a change is a plan first, then a go-ahead for exactly that plan; folders edited in between are left alone, replaced folders are saved for History, and one change runs at a time.

It is built to be safe on a computer where other web pages run in the same browser: the link works once and is traded for a cookie that scripts cannot read, requests that do not come from the page itself are refused, and nothing the pages show can become markup. The details are in `Project_Manag/Docs/Architecture/front_Ends.md`. The interface is a small web app (Astro and Solid, Lucide icons, Motion) in its own project, [`Ui/`](Ui/README.md); its built files are committed and held in the binary, so installing needs no node.

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
web: false                          # may `skillmirror web` start; optional, off unless true
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

A folder of the vault is a **skill** when it has a `SKILL.md`; the search for skills stops there. Any other folder that holds skills is a **group**, so the vault can be sorted into `Default_Skills/`, `Web_Skills/` and so on, and the groups nest without a limit (only the length of a path ends it). Folders without a skill below them are ignored, and `doctor` lists them. Skill names stay unique across the whole vault, and a group may not be named like a skill.

Groups only exist in the vault: the installed layout stays flat (`.agents/skills/<name>/`), so moving a skill between groups changes no project. `skillmirror skills` shows the tree; `add --group Web_Skills/seo` installs everything below a folder, at any level. A **profile** names a selection of groups and skills, can extend other profiles and can exclude names.

To sort a vault that is already full, move the folders and then run `git add -A` in the vault (no commit needed; the files git has staged are what is copied). Until you do, `sync` refuses the moved skill with a hint, and no project loses anything. `groups:` of a profile and `--group` take folder paths, so a path you moved has to be changed there too. Skills named in `mandatory:` and the copies in the projects are not affected, since they go by the skill's name.

### What travels with a skill

The vault must be a git repository. The files git tracks in a skill folder are what is copied, with their permission bits; untracked files never are, so a half finished edit does not leak out. A tracked symlink or submodule inside a skill, or a skill whose `SKILL.md` is not tracked, makes that skill fail with a hint instead of copying something wrong.

### Targets

`targets: [claude]` asks for `<project>/.claude/skills` to be a relative link to `../.agents/skills`, so Claude Code sees the same skills. `skillmirror bridge` makes the links; `add` and `init` make them for their project. A link is made only where nothing is: a real folder, a link that leads elsewhere and a linked `.claude` are reported and left alone. `status` and `doctor` tell you about a missing link.

## Safety

- A skill folder is replaced by building the new copy beside it and swapping the two folders in one step (`renameat2` exchange), so a crash never leaves a half written skill. Filesystems that cannot do this (NFS, CIFS, FUSE, FAT) are named by `doctor`.
- Before a run replaces or removes a folder, the old copy goes to `~/.local/state/skillmirror/backups/<run>/`; the newest 30 runs are kept. `history` lists them and `undo` brings one back.
- If a project folder changed between the plan and the write, that skill fails instead of being overwritten.
- Links are never followed and never written through. Only the commands that grow the vault write into it (`new`, `adopt`, `promote`, `vault init`, and the configuration file the Setup screen saves). `new` and `adopt` create files that did not exist; `promote` replaces one skill folder after keeping the old one in the backup store. All of them stage with `git add` and never commit.
- Everything the scan could not read, and every leftover of an interrupted run, is reported, never hidden.

## Development

```bash
just check      # format, clippy, tests, file size (300 code lines), core stays synchronous
just deny       # advisories, licences, bans
just parity     # replay 131 scenarios of the old Go tool against this one (needs the Go oracle)
Code/Development/Gate/check_Gate.sh      # all of the above plus the second toolchain and the feature build: GATE OK
Code/Development/Smoke/check_Smoke.sh    # the built binary end to end in a throwaway world, web server included
Code/Development/Web/check_Web.sh       # the web interface in Chromium
```

**Building Rust is heavy.** A clean build of the workspace and its tests with Cargo's defaults uses 2.5 GB of disk and peaks near 1.7 GB of memory, and a `target/` folder that has been used for a few days reaches 10 GB or more (debug information and the incremental cache). This repository turns both off in the root `Cargo.toml` (0.58 GB, peak near 1.2 GB, 35 s on four cores). It is still not light: on a small machine add `CARGO_BUILD_JOBS=1` (peak about 0.5 GB, three and a half times as long); `cargo clean` frees everything. Numbers, what to switch back on for a debugger, and the rest: [`build_Resources.md`](Project_Manag/Docs/Setup/build_Resources.md).

Crates: `Crates/Core` (the engine, no async), `Crates/Cli`, `Crates/Tui`, `Crates/Web` (the HTML report and the local server), `Crates/Testkit`. The web interface itself is `Ui/` (node, not part of the Cargo workspace; `Code/Development/Web/build_Ui.sh` builds it). The architecture, the behaviour contract (one numbered row per rule) and the decisions behind them are under `Project_Manag/Docs/`; start at [`doc_Start.md`](Project_Manag/Docs/doc_Start.md).

## License

[![Hippocratic License HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR](https://img.shields.io/static/v1?label=Hippocratic%20License&message=HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR&labelColor=5e2751&color=bc8c3d)](https://firstdonoharm.dev/version/3/0/bds-cl-eco-extr-media-mil-sv-xuar.html)

See [LICENSE](./LICENSE) for the full terms.
