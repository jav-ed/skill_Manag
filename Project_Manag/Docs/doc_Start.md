# doc_Start

`skill_Manag` is a Go CLI tool that propagates Claude Code skill files from a single master vault to all matching `.agents/skills/` directories across a codebase — git-tracked, SSH-safe, no symlinks.

Entry points: `main.go` → `cmd/root.go` (cobra setup + menu loop) → `cmd/interactive_Menu.go` (TUI entry).

## Docs

- [Architecture](Architecture/linker_Architecture.md) — code structure, TUI screens and their phase models, core logic files; start here for any code change
- [Sync concept and opt-in rule](Descr/sync_Concept.md) — what sync does and doesn't do, vault/project relationship, 4-step flow; read before changing sync behaviour
- [README — user-facing install, commands, config](../../README.md) — how users install and run the tool
