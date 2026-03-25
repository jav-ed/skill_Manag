# doc_Start

`skill_Manag` is a Go CLI tool that propagates Claude Code skill files from a single master collection to all matching `.agents/skills/` directories across a codebase — keeping skills in sync without symlinks so they stay git-tracked and work over SSH.

## Structure

```
Repos/        ← gitignored — local git clones for inspection only
              └── skill_Manag/   ← the actual source code (own git repo)
Project_Manag/ ← project management docs
.agents/skills/ ← local skills for this project (symlinked, gitignored)
```

The compiled binary is never stored here — install via `go install` into `$GOPATH/bin`.

## Docs

- [Project goals and design decisions](Project_Manag/Docs/linker_Goals.md)
