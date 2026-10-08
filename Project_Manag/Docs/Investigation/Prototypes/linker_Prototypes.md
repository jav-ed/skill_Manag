# linker_Prototypes

Small throwaway programs that were written to measure or reproduce something before a decision was made. They are kept as source archives (no build output) so the evidence can be re-run. They are not part of the Cargo workspace and are never compiled by `just check`.

## Docs

- [Probes and the TUI prototype](probes_And_Prototype.md): what each archive proves (the crossterm input stall, the termina backend that fixes it, the theme probe, the first TUI prototype with its tests), the exact commands to extract and run them, and which research report cites each result. Open it when someone asks "why termina and not crossterm" or wants to re-check a terminal-library claim after a version bump.
