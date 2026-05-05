# Writing Rules

1. Start with the gist of the repo.
   - One or two sentences max. What it is, what it does.
2. Keep `doc_Start.md` and all linker files shallow in structure, but not in label text.
   - They route the reader. They do not explain.
   - Link labels must be rich enough that the reader can make a routing decision without clicking. A bare area name ("Architecture") is not enough. Say what kinds of questions or tasks belong there.
3. Prefer progressive disclosure.
   - Short summary first, link second, deep detail in the target file.
4. Do not make recommendations.
   - No reading order, no guided path, no "start here first" unless explicitly asked.
5. Create `linker_<Topic>.md` for large areas.
   - If Architecture, Investigation, or any area grows large, add a linker file and link to it from `doc_Start.md`.
6. Prefer sub-linkers over leaf enumeration.
   - When a sub-folder has its own linker (e.g. `Pages/About/linker_About.md`), the parent should point to that sub-linker, not list the leaf files inside it. The parent links to the door, the sub-linker handles the room.
7. Avoid duplication.
   - Shared content belongs in one file. Link to it everywhere else.
8. Do not dump every doc into `doc_Start.md`.
   - Only include what helps the reader decide where to go next.
9. Use link labels that explain why to click.
   - Bad: `podman.md`
   - Better: `Podman container standards`
10. Keep agency with the reader.
    - The goal is to avoid loading the agent with context it did not ask for.

## Style conventions

These apply to `doc_Start.md`, all linker files, and any prose inside the docs tree. The agent reads the docs to learn house style; if the docs violate their own rules, every blog and doc the agent writes will mirror the violation.

11. No em-dashes ("—").
    - In nav lists, use a colon between the link and its description: `[Title](path.md): description`.
    - In prose, replace with commas, colons, periods, or parens depending on context.
    - The only acceptable em-dashes are inside literal quoted output strings or table cells used as "not applicable" markers.
12. Use *inshallah* where there is genuine future intent or uncertainty.
    - Mid-sentence flow: `"will inshallah do X"`, `"could inshallah become Y"`, `"posts will inshallah get written when..."`.
    - Never end-of-sentence with a comma. Never force it into present-tense rules or factual descriptions.
    - Full rules: see `Project_Manag/Docs/Writing/Python_Series/inshallah.md` (the project's authoritative reference).
13. H1 sibling consistency.
    - When several files share a folder, their H1s should follow the same pattern. If three siblings use `# Topic: Subtopic`, a fourth should not be `# Topic Subtopic` or `# Subtopic`. Match the pattern of the majority.
