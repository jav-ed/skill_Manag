---
name: subtitle-informer
description: Read subtitle files (VTT, SRT, ASS) for agent consumption
---

# subtitle_Informer

`subtitle_Informer` is a preinstalled binary — just call it directly, no setup needed.

Read transcript content from subtitle files without flooding your context. Supports VTT (word-level and regular), SRT, and ASS. Always prefer the word-level file (`en_Word.vtt`) when available — it gives millisecond-precise timestamps per word instead of per phrase. Always start with `info`, then work from there:

```bash
subtitle_Informer info --file en_Word.vtt          # total duration, word count, file type
subtitle_Informer chunks --file en_Word.vtt --words 500   # split into chunks — gives --from/--to boundaries to pass to slice
subtitle_Informer slice --file en_Word.vtt --from 1:00 --to 3:00  # returns actual timestamps + clean paragraph text
subtitle_Informer locate --file en_Word.vtt --text "they refused"  # text → timestamps (copy text from slice output)
subtitle_Informer search --file en_Word.vtt --query "ceasefire"    # find all mentions of a topic by keyword
```

Set once to skip `--file` on every call:

```bash
export TRANSCRIPT_SLICE_FILE=/path/to/en_Word.vtt
```

All commands support `--json`.

---

## Commands

| Command | Input | Output |
|---------|-------|--------|
| `info` | file | duration, word count, type |
| `chunks` | `--words N` | `--from/--to` boundaries for each chunk |
| `slice` | `--from`, `--to` | actual boundary timestamps + clean paragraph |
| `locate` | `--text` (copied from slice) | timestamps — case-insensitive, what you see is what you search |
| `search` | `--query` (any keyword) | all occurrences with timestamps and context |

`locate` vs `search`: use `locate` when you have text copied from `slice`. Use `search` when you are looking for a topic by keyword.

---

## If You Need More

→ [Full agent guide](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/Project_Manag/Docs/Descr/Subtitle_Informer/agent_Guide.md) — file types, `--overlap`, `--occurrence`, `--context-words`, JSON schemas, typical workflows

## See Also

→ [content-creation skill](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/.agents/skills/content-creation/SKILL.md) — if you want to extract posts or shorts from what you're reading
