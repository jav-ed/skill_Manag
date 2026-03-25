# linker_Architecture

Code structure for `skill_Manag` — two packages (`cmd/` and `internal/`) plus shared styles.

## TUI — `cmd/`

All interactive screens are bubbletea models. Each has a `phase` enum that drives `View()` and `Update()`.

| File | What it owns |
|------|-------------|
| `interactive_Menu.go` | Main menu — 4 items, detail panel updates on hover, routes to all screens |
| `interactive_Sync.go` | Sync screen — phases: loading → select → syncing → done; spinner, paginator, progress bar |
| `interactive_List.go` | List browser — table view, live `/` filter, select rows, sync or delete in place |
| `interactive_Delete.go` | Delete screen — phases: loading → select → confirm → done; nothing pre-selected |
| `interactive_Setup.go` | Setup screen — filepicker for vault and root, saves `~/.config/skill_Manag/config.yaml` |
| `keys.go` | Shared keybinding maps — `syncKeyMap`, `deleteKeyMap`, `listKeyMap`; `?` toggles help |
| `root.go` | Cobra root command, config init (`~/.config/skill_Manag/config.yaml`), menu loop |
| `sync.go` | Root command handler, `doSync`, `syncAll` (dry-run path) |
| `list.go` | `list` subcommand wiring |
| `delete.go` | `delete` subcommand — CLI mode (by name, by name+project, interactive) |

**Pattern:** every TUI screen uses `tea.WithAltScreen()`. Ctrl+C is handled in every phase. Shared `phase` type lives in `interactive_Sync.go` and is reused by `interactive_Delete.go`.

## Core logic — `internal/`

| File | What it owns |
|------|-------------|
| `walker.go` | `ReadMasterSkills` (vault → map), `FindTargets` (opt-in filter), `FindAllSkillTargets`, `FindTargetsByName` |
| `copier.go` | `SyncSkill` — walks vault skill dir, copies files into project skill dir; dry-run aware |
| `deleter.go` | `DeleteSkill` — removes skill dir from a project |
| `config.go` | Unused — `LoadConfig()` defined here but never called; config is loaded by `cmd/root.go` via viper directly |

**Key type:** `Target{ProjectPath, SkillName, SkillPath}` — the unit passed between walker and copier/deleter.

## Styles — `styles/`

Shared lipgloss styles (`Header`, `Muted`, `Success`, `Warning`, `Error`, `SkillName`) used across all TUI files.
