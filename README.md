# skill_Manag

> ⚠️ **Work in progress — not ready for production use.**

A CLI tool that keeps Claude Code skill files in sync across all your projects.

## The Problem

Claude Code skills live in `.agents/skills/<SkillName>/` inside each project. When you maintain multiple projects, you end up copying the same skill files everywhere. The moment you improve a skill in one project, every other project is out of date.

Symlinks would solve this — but they break over SSH and won't be tracked properly in git.

## The Solution

`skill_Manag` knows one central location (your master skill collection) and one root directory (where all your projects live). It walks every project, finds any `.agents/skills/<SkillName>/` that matches a skill in the master collection, and copies it over — overriding whatever is there.

Run it once after updating a skill. All projects stay in sync. Files remain real, git-tracked, and SSH-safe.

## Commands

### Sync — propagate skills from master to all projects

```bash
# Open interactive TUI — select which skills to sync
skill_Manag

# Non-interactive preview of all changes without applying them
skill_Manag --dry-run

# Override paths for a one-off run
skill_Manag --source /path/to/master/skills --root /path/to/projects
```

Only skills already installed in a project are updated — `skill_Manag` never installs a skill into a project that does not already have it.

### `delete` — remove a skill from projects

```bash
# Open interactive TUI — pick what to delete (nothing pre-selected)
skill_Manag delete

# Remove a skill from every project that has it
skill_Manag delete coding

# Remove a skill from one specific project
skill_Manag delete coding --project /path/to/project

# Preview what would be deleted without removing anything
skill_Manag delete coding --dry-run
```

## Configuration

Set your defaults once in `~/.config/skill_Manag/config.yaml` so you never need to pass flags:

```yaml
source: /path/to/your/master/skills
root:   /path/to/your/projects
```

`--source` and `--root` flags override the config file for any single run.

## Install

```bash
git clone git@github.com:jav-ed/skill_Manag.git
cd skill_Manag
go install .
```

The binary lands in `~/go/bin/skill_Manag`. Make sure `~/go/bin` is on your `$PATH`:

```bash
export PATH="$PATH:$HOME/go/bin"
```

## How It Works

1. Reads all skill names from the master collection directory
2. Walks all subdirectories under the scan root
3. Skips `.venv`, `node_modules`, `.git`, `vendor`, and other noise
4. For every `.agents/skills/<SkillName>/` found — if `<SkillName>` exists in the master — copies all files from master into the target, overriding existing files
