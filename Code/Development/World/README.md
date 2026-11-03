# World

A sandbox home to try skillmirror in without touching your own vault and projects, and a check that reads the tool against it. The world is what refac's fixture projects are for refac: a set of projects that each stand for one thing the tool has to tell apart, kept in the repository so nobody builds a pretend home by hand again.

```bash
just world                      # a shell in a fresh throwaway home: a vault and five projects in different states
just world first-run            # the home of the Getting started story: three projects, one skill, no vault yet
just world-check                # build the world and check that the tool reads every state as it was built
```

Without `just`: `Code/Development/World/enter_World.sh [lab|first-run]` (the `skillmirror` on the PATH, or `SKILLMIRROR_BIN=path`), `Code/Development/World/check_World.sh [binary]`, `Code/Development/World/build_World.sh DIR [lab|first-run]` for a world in a folder of your choice.

## What is in the lab

The vault `~/skills` is a git repository with three commits, so a project can hold an older version of a skill. It holds `coding` (with a `references/` folder), `doc-start` (a small stand-in whose `Code/bootstrap.sh` is the scaffold `init` runs), `file-tree-optimization`, the group `Web_Skills` with `astro` and the nested group `seo/schema`, the folder `project-files` with the text of `AGENTS.md`, and a `config.yaml` with mandatory skills, `targets: [claude]`, `exclude_dirs: [testdata]` and a profile.

| Project | What it stands for | What the tool must say |
| --- | --- | --- |
| `atlas-api` | everything current, the `.claude/skills` link in place, a fixture project below `testdata` | in sync, `AGENTS.md` current, the fixture is not listed |
| `docs-site` | `coding` one version behind, a skill only this project has, one mandatory skill missing, no `AGENTS.md` | outdated, `behind` and `new` in `changes`, mandatory missing, `AGENTS.md` missing |
| `infra` | `coding` edited here (a line the vault never had), a skill only this project has, an `AGENTS.md` block edited by hand | outdated, `edited` and `new` in `changes`, `AGENTS.md` edited |
| `lumen-app` | all skills current, an `AGENTS.md` block written from the first text | `AGENTS.md` outdated |
| `orbit` | an empty skills folder, an `AGENTS.md` of its own without a block | all mandatory skills missing, `AGENTS.md` without a block |

The first-run world has `atlas-api`, `docs-site` and `infra`, each with the same copy of `coding`, and an empty home. The docs site starts its examples there.

## Why it is built and not committed

The real tool scans the folder that holds this repository. A project committed here, a folder called `.agents/skills`, would be found by that scan and written into by a real `sync`. So `Files/` holds only plain skill folders under neutral places (`vault/`, `older/`, `local/`), and `build_World.sh` puts the projects together in a folder you name. `enter_World.sh` builds into `Scratch/World`, which is disposable, gitignored and skipped by scans (`exclude_dirs: Scratch` in the doc-start convention). Nothing in a vault configuration has to ignore this repository.

The builder runs as the world and never as you: its own `HOME`, its own git identity, none of your git configuration. Everything is plain files and git, except the `AGENTS.md` blocks, which only the tool can write. That keeps the world an independent witness: if `status` stops saying that `docs-site` is behind, the world is right and the tool is not.

## Make a change

- A new state: put the files in `Files/`, build the project in `build_World.sh`, add a line to the table above and the check that names it to `check_World.sh`.
- A new expectation about the tool: one `step` and its `has` lines in `check_World.sh`. The check prints one `PASS` or `FAIL` line per expectation and ends with `WORLD OK`, or `WORLD FAILED: n check(s)` and exits with `n`. Pointed at a binary that does nothing (`/bin/true`) it fails with 55 checks, so it cannot pass by accident.
- Keep every script under 300 lines.

## How it differs from the other checks

| Check | What it proves |
| --- | --- |
| `cargo test` with `Crates/Testkit` | one rule at a time, in a tree made for that test |
| `Code/Development/Smoke/` | the installed binary works end to end, from a vault made from nothing |
| `Code/Development/World/` | a standing home with every state in it: to explore by hand, to quote from, and to check the verdicts |
| `Code/Development/Parity/` | the Rust tool against the old Go tool |

The examples on the documentation site (`Skillmirror_Docs`) are recorded in this world. Its `Setup/example_World.md` says how they are replayed.
