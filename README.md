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

Running `skill_Manag` with no arguments opens a full-screen TUI. Every screen supports full mouse and keyboard navigation.

### Navigation

- **Mouse** — hover highlights items, click selects or toggles
- **`alt+←`** or **`q`** — go back to the main menu from any screen
- **`←`** in the header — clickable back button
- **`?`** — toggles a keybinding reference on every screen

### Main menu

Five actions available from the main menu. Mouse hover moves the highlight; click or `enter` opens the screen.

| Action | What it does |
|--------|-------------|
| **Sync** | Refresh each project's installed skills from vault |
| **List** | Browse all skills installed across all projects |
| **Delete** | Remove selected skills from projects |
| **Push** | Force-install mandatory skills to every opted-in project |
| **Setup** | Reconfigure vault and root paths |

### Sync screen

- Animated spinner while your project tree is scanned in the background
- Checklist of every skill found — all pre-selected, deselect what you don't want
- Click a row or press `space` to toggle; `a` toggles all
- Paginated with dot indicators when you have more than 10 skills (`• · · ·`)
- Animated progress bar fills as each skill syncs (`Syncing 3 / 7`)
- Results screen shows per-skill outcome with file counts and any errors

### List screen

- Same checklist layout as Sync with an added project-path column
- Live filter: press `/` and type to narrow by skill name, `esc` to clear
- Click a row or press `space` to toggle; `a` to select all visible
- `s` syncs selected skills directly from the list (requires vault to be configured)
- `d` deletes selected skills directly from the list

### Delete screen

- Same checklist layout as Sync — nothing pre-selected, opt in explicitly
- Pressing `enter` shows a confirmation prompt before anything is deleted
- `y` or `enter` to confirm, `n` or `esc` to go back
- Results screen shows what was deleted and any errors

### Push screen

- Shows only the skills listed under `mandatory` in your vault config
- All pre-selected by default — deselect what you don't want this run
- Pushes to every project that has `.agents/skills/` with any skill installed — creates the skill dir if it doesn't exist yet
- Press `e` to open the mandatory edit overlay — toggle which vault skills are mandatory, `enter` saves and re-scans, `esc/q` cancels
- Results screen shows per-skill outcome with file counts and any errors

### Setup screen

- Three-step wizard: vault → root → mandatory skills
- Filesystem picker for vault and root — no manual path typing; navigate with arrow keys, `enter` to select
- Mandatory step shows all vault skills as a checklist — toggle with `space`, confirm with `enter`
- Confirmation prompt before saving — writes vault pointer to `~/.config/skill_Manag/vault` and config to `<vault>/config.yaml`
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

Config is split across two files so the vault is fully self-contained and portable:

```
~/.config/skill_Manag/vault   ← one line: path to your vault
<vault>/config.yaml           ← root and mandatory skills
```

```yaml
# <vault>/config.yaml
root: /path/to/your/projects
mandatory:
  - coding
  - doc-start
```

`mandatory` is optional — omit it if you don't use Push. The Setup screen writes both files for you. `--vault` and `--root` flags override config for any single run. Environment variables `SKILL_MANAG_VAULT` and `SKILL_MANAG_ROOT` also work.

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

---

## License

[![Hippocratic License HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR](https://img.shields.io/static/v1?label=Hippocratic%20License&message=HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR&labelColor=5e2751&color=bc8c3d)](https://firstdonoharm.dev/version/3/0/bds-cl-eco-extr-media-mil-sv-xuar.html)

See [LICENSE](./LICENSE) for the full terms.
