# sync_Concept

How `skillmirror sync` works and what it intentionally does not do. The exact rules, one numbered row each, are in the [behavior contract](behavior_Contract.md); this page is the idea behind them.

## The opt-in rule

A project only receives updates for skills it already has. `sync` will inshallah never install a skill into a project that has not opted in.

A project opts in by having a `.agents/skills/<name>/` folder. Once it is there, every later sync keeps it up to date from the vault. To stop receiving updates, delete the folder, or use `skillmirror delete`.

The vault is the source of truth for **content**. Each project controls its own **skill set**. Putting a skill into a project is a separate, explicit act: `add` (chosen skills, groups or profiles into an existing project), `init` (a new project) or `push` (the mandatory list, into every project that has a skills folder).

## Sync flow

```
1. Read the vault: every folder with a SKILL.md is a skill, the folders above it are its group
2. Ask git for the tracked files of each skill (one call for the whole vault)
3. Walk the scan root in parallel, skipping noise folders and the configured exclusions, never following links
4. For every .agents/skills/<name>/ found:
     <name> is in the vault     -> plan: compare, and build the new copy
     <name> is not in the vault -> leave it alone (list, status and report mark it)
5. Show the plan (--dry-run stops here), ask, then apply it target by target
```

Sync is a **mirror**, not an overlay: a project copy ends up equal to the vault copy, so files an older version had and the vault no longer has are removed. The confirmation names the files that would be removed, and `diff` shows every line.

## How a copy is written

For each target the new copy is built beside the old one, checked, and swapped in one step. Either the project has the old folder or the new one, never a mixture. Before the swap, the old copy is kept in the backup store, so `skillmirror undo` can bring it back. If the project folder changed after the plan was made, that target fails instead of being overwritten. One failed target never stops the others (exit code 4).

## What files get copied

The vault must be a git repository. The files git tracks in the skill folder are copied, with their permission bits. Untracked files are never copied, so a skill that is half edited in the vault, or mid rename, does not leak into projects; an edit that is only uncommitted is already mirrored, because the working tree version of a tracked file is what is copied. A tracked symlink or submodule inside a skill, a tracked file missing from the disk, or a skill whose `SKILL.md` is not tracked makes that skill fail with a hint. `doctor` warns about edits git does not know about.

## Scan exclusions

Built-in: `.git`, `node_modules`, `vendor`, `dist`, `build`, `out`, `target`, `.next`, `.nuxt`, `.venv`, `__pycache__`, `.tox`, `.pytest_cache`, `.cache`, `.turbo`, `.parcel-cache`. In `<vault>/config.yaml`:

```yaml
exclude_paths:          # absolute, or relative to the root; skipped with everything below
  - /path/to/project/testdata
exclude_dirs: [testdata]  # a bare folder name, skipped wherever it occurs
```

Prefer `exclude_paths` when one fixture or scratch tree is the problem. A path that matches nothing is an error, so a typo does not silently exclude nothing.

## Push and the mandatory list

```yaml
mandatory: [coding, doc-start]
```

`push` finds every project that has a `.agents/skills/` folder (even an empty one) and installs the mandatory skills there, creating each skill folder that is missing and updating the ones that are present. A mandatory name the vault does not have is a hard error.

## Configuration

- `~/.config/skillmirror/vault`: one line, the path of the vault.
- `<vault>/config.yaml`: `root`, `mandatory`, `exclude_dirs`, `exclude_paths`, `targets`, `profiles`. It belongs to the vault and travels with it. Unknown keys are an error.
- Flags beat environment variables (`SKILLMIRROR_VAULT`, `SKILLMIRROR_ROOT`), which beat the files.

## Where this is implemented

The plan is `Crates/Core/src/plan`, the write is `Crates/Core/src/apply`, the opt-in filter is `scan/targets.rs` (`sync_targets` keeps only installed folders the vault has, `push_targets` adds the mandatory ones). See [rust_Overview](../Architecture/rust_Overview.md).
