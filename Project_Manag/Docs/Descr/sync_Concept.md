# sync_Concept

How `skill_Manag` sync works and what it intentionally does not do.

## The opt-in rule

A project only receives updates for skills it already has. `skill_Manag` will inshallah never install a skill into a project that hasn't opted in.

A project opts in by having a `.agents/skills/<SkillName>/` directory present. Once it's there, every future sync will inshallah keep it up to date from the vault. To stop receiving updates, delete the directory — or use `skill_Manag delete`.

The vault is the source of truth for **content**. Each project controls its own **skill set**.

## Sync flow

```
1. Read all skill names from the vault directory
2. Walk all subdirectories under the scan root
3. Skip noise dirs (.git, node_modules, vendor, dist, build, out, target,
   .next, .nuxt, .venv, __pycache__, .tox, .pytest_cache,
   .cache, .turbo, .parcel-cache)
4. For each .agents/skills/<SkillName>/ found:
     if <SkillName> exists in vault → delete skill dir, copy vault fresh
     if <SkillName> not in vault    → skip entirely
```

Sync is a **mirror**, not an overlay. Deleting the skill dir before copying ensures no stale files from older skill versions survive in the project.

## What files get copied from the vault

When copying a skill from the vault, `SyncSkill` uses a git-aware file selection strategy:

**Primary — git repo detected:**
`git ls-files --cached .` is run scoped to the vault skill directory. Only staged/committed files are returned. Untracked files are intentionally excluded — if the vault is mid-rename or has unstaged additions, they are invisible to sync until the vault author stages them. This prevents mid-transition vault state (partial renames, unstaged adds) from producing duplicate or ghost entries in projects.

**Fallback — no git repo:**
A filtered directory walk is used instead. Directories in the skip list above are skipped, and symlinks are excluded (they are machine-specific and not portable).

## Example

```
vault/
  coding/        ← master copy
  doc-start/
  refac-cli/

projects/
  project-A/
    .agents/skills/
      coding/    ← exists → updated from vault
      doc-start/ ← exists → updated from vault
                    refac-cli not here → NOT touched

  project-B/
    .agents/skills/
      refac-cli/ ← exists → updated from vault
                    coding not here → NOT touched
```

## Where this is implemented

The opt-in filter lives in `internal/walker.go` — `FindTargets()` only appends a `Target` when the project's skill name is present in the vault map. `internal/copier.go` — `SyncSkill()` — receives an already-filtered target and just executes the copy.

## Push — bypassing the opt-in rule

Some skills should reach every project regardless of opt-in. Declare them in `<vault>/config.yaml`:

```yaml
root: /path/to/projects
mandatory:
  - coding
  - doc-start
```

Push reads this list, finds every project that has a `.agents/skills/` directory (even if empty), and copies the mandatory skills there — creating each skill dir if it doesn't exist yet. This is implemented in `internal/walker.go` — `FindPushTargets()`.

## Config structure

- `~/.config/skill_Manag/vault` — plain text file, one line: the vault path
- `<vault>/config.yaml` — `root` and `mandatory` keys; owned by the vault, travels with it
