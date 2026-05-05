---
name: skill-writer
description: Create new agent skills using a navigation-first structure. SKILL.md gives the agent orientation and common-case usage inline, then routes to bundled references for the long tail. Use when the user wants to create, write, or build a new skill.
---

# skill-writer

A skill teaches an agent how to do something. The agent reads it alongside everything else in its current context, and must quickly decide which parts are relevant to the task at hand. So a skill is judged by how cleanly it surfaces the right information, not by how thoroughly it documents the topic. This file is itself structured the way it tells you to structure yours.

## The principle: navigation-first, not dump-first

An agent works best when it can quickly tell what in its context is relevant to the current task and what is not. Every irrelevant section loaded competes with the relevant ones and degrades the agent's decisions. So a skill must give the agent an easy time deciding what to read and what to skip. And what is not loaded must still be findable through good labels, since the agent cannot easily come back to a doc later. A skill must:

1. Give enough orientation upfront that the agent understands the domain.
2. Inline the common case so the most-used path needs no follow-up reads.
3. Route the long tail through clearly-labeled links, so the agent can decide what to load without opening the file.

The load-bearing piece is the labels, not the file split. A flat doc with sharp labels beats a deeply nested doc with murky ones every time.

## Skill structure

```
skill-name/
├── SKILL.md                # entry point: orientation + common case + links
├── References/             # long-tail docs, one topic per file
│   ├── topic_A.md
│   └── topic_B.md
└── Code/                   # optional, code the skill provides (validators, helpers, etc.)
```

`References/` and `Code/` cover most needs, but a skill can add other top-level folders when a category genuinely warrants its own (for example `Templates/`, `Examples/`, `Schemas/`). Add a folder only when it improves readability, scannability, or scalability. Folder names should be self-explaining, and the structure should remain easy to scan at a glance.

If unsure about `.md` file or folder naming, see the `coding` skill, which documents the project-wide convention. (`SKILL.md` itself is uppercase, an exception to the lowercase rule because that is what the harness recognizes.)

A skill that fits the common case in 30 lines does not need a `References/` folder at all. Splitting is a tool, not a virtue.

## SKILL.md shape

Three layers, in order: orientation (one or two paragraphs on what the skill does and why), the common case (the most-used path, fully explained inline), and navigation (links to references for the long tail).

````markdown
---
name: skill-name
description: One sentence on what the skill does. Use when [specific triggers].
---

# skill-name

[Orientation: what the skill does, why an agent would reach for it, framing.]

## [Common-case section heading]

[Inline content for the most-used path. Multiple ## sections are fine.]

## References

- [Topic A](References/topic_A.md): description rich enough for the agent to decide without clicking
- [Topic B](References/topic_B.md): same
````

Frontmatter: max 1024 chars, third person, two sentences (what it does, when to use it). The description is the only thing the agent sees when deciding whether to load the skill, so vague phrasing kills it.

Link descriptions are the load-bearing piece. The text after the colon must say what kinds of tasks or questions belong behind the link, rich enough that the agent can decide whether to follow without opening the file. A bare filename or topic name is not a description. Length follows need: one line when one line covers it, five lines when the topic genuinely needs five. Optimize for clarity, not brevity.

See [Labels and descriptions](References/labels.md) for examples and failure modes.

## Recursion

A reference file is a smaller skill. It gets its own orientation, common case, and links if it needs them. The pattern applies at every level. If a reference file turns into a dump, split it the same way SKILL.md was split.

## Review checklist

Before declaring the skill done:

- Frontmatter `name` matches the folder name; `description` is under 1024 chars, third person, two sentences with a specific "Use when" trigger.
- SKILL.md opens with orientation, fully covers the common case inline, and routes the long tail through clearly-labeled links.
- Every link uses `[Label](path.md): description` format with a colon. No bare filenames, no vague "see also".
- Each reference file follows the same orientation + common case + links pattern. No reference is a dump.
- Self-test: if the agent only reads SKILL.md, can it do the common case end-to-end?

## References

- [Process](References/process.md): drafting a skill from scratch with the user, gathering requirements, reviewing, iterating.
- [Labels and descriptions](References/labels.md): rules for the frontmatter description and inline link descriptions, with format, character limits, length guidance, and worked good/bad examples.
- [Failure modes](References/failure_Modes.md): over-splitting, misjudged common case, atomization, premature linker creation, and other ways the navigation-first pattern gets misapplied.
- [When to add code](References/code.md): criteria for bundling executable helpers in the skill, and when generated code is fine.
