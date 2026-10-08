# linker_Descr

What `skillmirror` does and, just as important, what it deliberately does not do. This area holds the product concept: the relationship between vault and projects, the opt-in rule, how sync mirrors files, how push bypasses opt-in for mandatory skills, and how scan exclusions and config files fit together. Read it before changing behaviour; it does not describe code structure (see the Architecture area for that).

## Docs

- [Sync concept and opt-in rule](sync_Concept.md): the idea behind sync: why a project only receives skills it already has, the sync flow, how a copy is written (stage, check, swap, backup), which vault files are copied (git-tracked only, permissions preserved), scan exclusions, push and the `mandatory` list, and the configuration files. Open it for the why before reading the rules.
- [Behavior contract](behavior_Contract.md): sections 1 to 9 are the exact observable behavior of the Go tool at commit `c7310f9` (commands, flags, exit codes, config precedence, walker, copier, literal output strings, every TUI screen and key binding) with a tagged quirk table (KEEP, CHANGE, UNCLEAR); from Q34 on the same table holds the numbered rules that exist only in the Rust tool (backups and undo, profiles, interface discipline, status, diff, doctor, bridge, scan progress, interface pages, report), each with its code and tests. Open it before changing a behaviour, or to find out whether a behaviour is a rule.
