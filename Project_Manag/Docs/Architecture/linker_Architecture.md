# linker_Architecture

Code structure for `skill_Manag` — two packages (`cmd/` and `internal/`) plus shared styles.

## CLI wiring — `cmd/`

| File | What it owns |
|------|-------------|
| `root.go` | Cobra root command, config init (vault pointer → vault config), menu loop |
| `sync.go` | Root command handler, `doSync`, `syncAll` (dry-run path) |
| `push.go` | `doPush` — reads `mandatory` from viper, delegates to `tui.RunPush` |
| `list.go` | `list` subcommand wiring |
| `delete.go` | `delete` subcommand — CLI mode (by name, by name+project, interactive) |

## TUI — `cmd/tui/`

All interactive screens are bubbletea models. Each has a `phase` enum that drives `View()` and `Update()`. Every screen uses `tea.WithAltScreen()` and `tea.WithMouseAllMotion()`.

| File | What it owns |
|------|-------------|
| `menu.go` | Main menu — 5 items, detail panel, mouse hover drives cursor via `list.Select()`, click navigates |
| `sync.go` | Sync screen — phases: loading → select → syncing → done; spinner, paginator, progress bar |
| `push.go` | Push screen — same phase lifecycle as sync; uses Warning color; reads mandatory from vault config |
| `list.go` | List browser — same checkbox renderer as sync/delete with project path column, live `/` filter, paginator, sync or delete in place |
| `delete.go` | Delete screen — phases: loading → select → confirm → done; nothing pre-selected |
| `setup.go` | Setup wizard — filepicker for vault and root, `✓ / →` step progression, saves vault pointer + vault config |
| `keys.go` | Keybinding maps — `syncKeyMap`, `deleteKeyMap`, `listKeyMap`; all include `alt+left` for back navigation |
| `header.go` | `handleHeaderMouse` — shared mouse handler for the `←` back button and javedab.com link |
| `shared.go` | Shared types: `phase`, `pageSize`, `shortPath` |
| `browser.go` | `openBrowser` helper |

**Unified screen structure** — all four action screens share the same visual layout:
```
← skill_Manag  ·  javedab.com  ·  [screen]

Title                          N / M selected
  [·] skill                    projects / path
  ────────────────────────────────────────────
  [✓] skill-name               N projects
> [ ] skill-name               N projects
  ────────────────────────────────────────────

space toggle  a all  enter confirm  q/alt+← back
```

**Mouse wiring** — bubbles list and table have no mouse support; all mouse handling is manual:
- Menu: `list.Select(idx)` called on every `MouseActionMotion` to drive visual cursor; `MouseActionPress` confirms
- Sync/Delete/List: `MouseActionMotion` updates cursor, `MouseActionPress + ButtonLeft` toggles selection
- Row index formula: `idx = msg.Y - itemsStart` where `itemsStart` is a constant per screen based on header line count

**Back navigation** — `← ` in the header (col 0–1) replaces the leading spaces when `screen != ""`. Click or `alt+left` calls `tea.Quit`, returning control to `cmd/root.go`'s menu loop.

## Core logic — `internal/`

| File | What it owns |
|------|-------------|
| `walker.go` | `ReadMasterSkills` (vault → map), `FindTargets` (opt-in filter), `FindPushTargets` (mandatory, bypasses opt-in), `FindAllSkillTargets`, `FindTargetsByName` |
| `copier.go` | `SyncSkill` — selects vault files via `gitListFiles` (git-aware, respects `.gitignore`) or `walkVaultFiles` fallback, copies into project skill dir; dry-run aware |
| `deleter.go` | `DeleteSkill` — removes skill dir from a project |
| `config.go` | Unused — `LoadConfig()` defined here but never called; config is loaded by `cmd/root.go` via viper directly |

**Key type:** `Target{ProjectPath, SkillName, SkillPath}` — the unit passed between walker and copier/deleter.

## Styles — `styles/`

Shared lipgloss styles (`Header`, `Muted`, `Success`, `Warning`, `Error`, `SkillName`) plus hit-test constants for the header:
- `HeaderLinkRow/Col/Len` — position of the javedab.com link
- `HeaderBackRow/Col/Len` — position of the `←` back arrow (col 1, only present when `screen != ""`)
