# skill_Manag

> ⚠️ **Work in progress — not ready for production use.**

A CLI tool that keeps Claude Code skill files in sync across all your projects — one vault, zero drift.

Built by [javedab.com](https://javedab.com) — get in touch if you want help with your tooling.

---

## The Problem

Claude Code skills live in `.agents/skills/<SkillName>/` inside each project. When you maintain multiple projects, you end up copying the same skill files everywhere. The moment you improve a skill in one project, every other project is out of date.

Symlinks would solve this — but they break over SSH and won't be tracked properly in git.

## The Solution

`skill_Manag` knows two things: your **vault** (one folder where you write and maintain your skills) and your **root** (the folder that contains all your projects).

It walks every project under that root, finds every `.agents/skills/<SkillName>/` directory, and for any skill that also exists in the vault — copies the vault version in, overriding whatever is there.

**Key rule: it only updates skills a project already has. It never installs a skill into a project that hasn't opted in.** Each project controls its own skill set by what it has in its `.agents/skills/` directory.

```
vault/
  coding/        ← your master copy
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

---

## Interactive TUI

Running `skill_Manag` with no arguments opens a full-screen TUI. Every screen supports keyboard navigation and a `?` key that toggles a keybinding reference.

### Main menu

Four actions available from the main menu:

| Action | What it does |
|--------|-------------|
| **Sync** | Refresh each project's installed skills from vault |
| **List** | Browse all skills installed across all projects |
| **Delete** | Remove selected skills from projects |
| **Setup** | Reconfigure vault and root paths |

### Sync screen

- Animated spinner while your project tree is scanned in the background
- Checklist of every skill found — all pre-selected, deselect what you don't want
- `space` toggles a single skill, `a` toggles all
- Paginated with dot indicators when you have more than 10 skills (`• · · ·`)
- Animated progress bar fills as each skill syncs (`Syncing 3 / 7`)
- Results screen shows per-skill outcome with file counts and any errors
- `?` toggles short ↔ full keybinding reference

### List screen

- Table view with headers: checkbox · skill name · project path
- Live filter: press `/` and type to narrow by skill name, `esc` to clear
- `space` to select rows, `a` to select all visible
- `s` syncs selected skills directly from the list (requires vault to be configured)
- `d` deletes selected skills directly from the list
- `?` toggles keybinding reference

### Delete screen

- Same checklist layout as Sync — nothing pre-selected, opt in explicitly
- Pressing `enter` shows a confirmation prompt before anything is deleted
- `y` or `enter` to confirm, `n` or `esc` to go back
- Results screen shows what was deleted and any errors

### Setup screen

- Filesystem picker for both vault and root — no manual path typing
- Walks your actual directory tree, navigate with arrow keys, `enter` to select
- Confirmation prompt before saving to `~/.config/skill_Manag/config.yaml`
- Runs automatically on first launch if no config is found

---

## CLI Commands

All commands also work non-interactively for scripting.

### Sync

```bash
# Open interactive TUI
skill_Manag

# Preview all changes without applying them
skill_Manag --dry-run

# One-off run with explicit paths
skill_Manag --vault /path/to/skill/vault --root /path/to/projects
```

### List

```bash
skill_Manag list
skill_Manag list --root /path/to/projects
```

### Delete

```bash
# Interactive TUI — nothing pre-selected
skill_Manag delete

# Remove one skill from every project that has it
skill_Manag delete coding

# Remove one skill from one specific project
skill_Manag delete coding --project /path/to/project

# Preview what would be deleted
skill_Manag delete coding --dry-run
```

---

## Configuration

Set your defaults once so you never need to pass flags:

```yaml
# ~/.config/skill_Manag/config.yaml
vault: /path/to/your/skill/vault
root:  /path/to/your/projects
```

`--vault` and `--root` flags override the config for any single run. Environment variables `SKILL_MANAG_VAULT` and `SKILL_MANAG_ROOT` also work.

The Setup screen in the TUI writes this file for you.

---

## Install

Requires Go 1.24+.

```bash
git clone git@github.com:jav-ed/skill_Manag.git
cd skill_Manag
go install .
```

The binary lands in `~/go/bin/skill_Manag`. Make sure that's on your `$PATH`:

```bash
export PATH="$PATH:$HOME/go/bin"
```
