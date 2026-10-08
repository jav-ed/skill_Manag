# linker_Descr

What `skill_Manag` does and, just as important, what it deliberately does not do. This area holds the product concept: the relationship between vault and projects, the opt-in rule, how sync mirrors files, how push bypasses opt-in for mandatory skills, and how scan exclusions and config files fit together. Read it before changing sync behaviour; it does not describe code structure (see the Architecture area for that).

## Docs

- [Sync concept and opt-in rule](sync_Concept.md): the 4-step sync flow, why a project only receives skills it already has, which vault files get copied (git-tracked only, permissions preserved), `exclude_dirs` and `exclude_paths`, push and the `mandatory` list, and the config file layout (`~/.config/skill_Manag/vault` plus `<vault>/config.yaml`).
- [Behavior contract of the Go tool](behavior_Contract.md): exact observable behavior at Go commit `c7310f9` for reimplementation and parity tests: commands, flags, exit codes, config precedence and YAML rewrite, walker pruning and ordering, copier semantics, literal CLI output strings, every TUI screen and key binding, and a tagged list of quirks (KEEP, CHANGE, UNCLEAR).
