# Folder Guidance

The doc system lives under `Project_Manag/`, split into two top-level folders:

- `Project_Manag/Docs/`: reference material (Architecture, Decisions, etc.)
- `Project_Manag/Live_Working/`: active work and ongoing planning

## Docs/ subfolders

| Folder | Required? | Use for |
|---|---|---|
| `Architecture` | required | Structure, systems, technical layout, integration boundaries |
| `Decisions` | required | Important decisions, tradeoffs, ADR-like notes |
| `Descr` | required | What the repo or product does, domain model, conceptual descriptions |
| `Research` | required | Quick lookups, online finds, lightweight external content gathered for context |
| `Brand` | optional | Brand assets, voice, visual identity |
| `Investigation` | optional | In-depth study of a tool, system, or topic. Sustained work that can grow its own subfolder structure |
| `Setup` | optional | Local setup, environment bootstrap, install steps |

The four required folders exist on every repo from bootstrap. Optional folders are added only when the project has material in that category.

`Research` vs `Investigation`: `Research` is for quick, lightweight external lookups (a few links or excerpts gathered for context). `Investigation` is for sustained, in-depth study of a single subject and can grow its own internal subfolder structure as the work develops.

## Live_Working/ contents

| File | Required? | Use for |
|---|---|---|
| `open_Issues.md` | required | Active items, in-progress work, things to track |

Add other files in `Live_Working/` as the project's active-work needs grow (task pools, sprint plans, etc.).
