---
name: doc-start
description: Write or reorganize repository documentation using a navigation-first structure. `doc_Start.md` is the repo entry point. Large topic areas get their own `linker_<Topic>.md` file. Every file opens with a summary, then routes through clearly-labeled links.
---

# doc-start

Write docs so an agent can orient itself without being forced to read everything. The docs system is a map: `doc_Start.md` gives the agent enough overview to decide what is relevant, and each linker file does the same for its sub-area. The agent navigates the system, it does not consume it. This skill follows the same pattern, applied to its own SKILL.md.

## The principle: navigation-first, not dump-first

An agent works best when it can quickly tell what in its context is relevant to the current task and what is not. Every irrelevant doc loaded competes with the relevant ones and degrades the agent's decisions. So the docs system must give the agent an easy time deciding what to read and what to skip, and what is not loaded must still be findable through good labels. The system must:

1. Give enough orientation upfront, at every level, that the agent understands the area.
2. Inline a summary at the top of `doc_Start.md` and every linker file, so the most-used context comes for free.
3. Route the long tail through clearly-labeled links, so the agent can decide what to load without opening the file.

The load-bearing piece is the labels, not the file split. A flat doc with sharp labels beats a deeply nested doc with murky ones every time.

## Doc system structure

Three tiers, top to leaf:

```
repo-root/
├── doc_Start.md                          # repo entry point: summary + routing
└── Project_Manag/
    ├── Docs/
    │   ├── Architecture/                 # required
    │   │   ├── linker_Architecture.md    # area summary + links to leaves
    │   │   ├── leaf_doc_a.md
    │   │   └── leaf_doc_b.md
    │   ├── Decisions/                    # required
    │   ├── Descr/                        # required
    │   ├── Research/                     # required
    │   ├── Brand/                        # optional
    │   ├── Investigation/                # optional
    │   └── Setup/                        # optional
    └── Live_Working/
        └── open_Issues.md                # required
```

`doc_Start.md` lives at the repo root. Topic-area docs live under `Project_Manag/Docs/<Area>/`. Each area gets its own `linker_<Area>.md` once it has docs. Sub-areas can have their own linkers, in which case the parent points to the sub-linker, not the sub-linker's leaves.

When starting a new repo, the required folders and `Live_Working/open_Issues.md` need to be present. The user can run `Code/bootstrap.sh` to create the full scaffold (folders + linker stubs + `doc_Start.md` + `open_Issues.md`); the agent can also create missing pieces by hand, but should only run the script when explicitly asked. `Brand/`, `Investigation/`, and `Setup/` are added only when the project actually has material in those categories. For what each folder is for, see [Folder guidance](References/folder_Guidance.md).

## Writing rules (common case)

These are the rules used in nearly every doc-start task. The full ruleset including style conventions is in [Writing rules](References/writing_Rules.md).

- `doc_Start.md` and every linker file open with a summary of the area before the link list. They are not pure indexes.
- Every link uses `[Short label](path.md): description rich enough for the reader to decide without clicking`. Use a colon, never an em-dash.
- Link descriptions are not limited to one line. One line when one line covers it, five lines when the topic genuinely needs five. Clarity for navigation, not brevity.
- When a sub-folder has its own linker, the parent links to that sub-linker, not its leaves. The parent points to the door, the sub-linker handles the room.
- Do not dump every doc into `doc_Start.md`. Include only what helps the reader decide where to go next.
- Do not impose a reading order ("start here first") unless the user explicitly asks for one.

## File and folder naming

`.md` files use lowercase first letter (`writing_Rules.md`, `linker_Architecture.md`). Folders use uppercase first letter (`Architecture/`, `Project_Manag/`). For the full naming convention, see the `coding` skill.

## References

- [Writing rules](References/writing_Rules.md): the full numbered ruleset including style conventions: no em-dashes, *inshallah* usage, H1 sibling consistency.
- [Folder guidance](References/folder_Guidance.md): the canonical top-level folders under `Project_Manag/Docs/` and what each is for.
- [Process](References/process.md): step-by-step process for creating new docs and for editing or verifying an existing linker.
- [doc_Start template](References/doc_Start_Template.md): scaffold for the repo entry point.
- [Linker template](References/linker_Template.md): scaffold for sub-area linker files.
- [Failure modes](References/failure_Modes.md): pure-index linkers, leaf enumeration, dumping into `doc_Start.md`, vague labels, stale linkers, duplicate content, and other ways the navigation-first pattern gets misapplied.
