---
name: content-creation
description: Extract posts and shorts from long-form video source material
---

# Content Creation

Extract posts and shorts from any long-form video — lecture, interview, documentary, speech, podcast.

This work is truth-based. The source may challenge mainstream narratives. Your job: **follow the reasoning. If it's sound, represent it faithfully. If it's flawed, say why.** Do not hedge based on who is implicated.

## Prerequisites — check this BEFORE anything else

**Word-level AI subtitles are required for this skill.** The manual YouTube SRT (`Subtitles/Yt/en_Manual.srt`) **cannot be substituted** — `short_Maker` only burns word-level ASS, and timestamp precision below the phrase level is required throughout the workflow.

Before reading any other reference file, verify both of these exist:

```
/home/jav/Videos/01_Recordings/{NNNN}_{name}/Subtitles/WhisperX/en_Word.vtt
/home/jav/Videos/01_Recordings/{NNNN}_{name}/Subtitles/WhisperX/en_Word.ass
```

If either is missing, **stop the content workflow and generate them first** via the video-downloader skill's AI subtitle step:

```bash
uv run python Code/Lightning/prepare_Audio.py {NNNN}_{name}
uv run python Code/Lightning/run_Pipeline.py {NNNN}_{name}
```

The pipeline keeps the GPU studio running by default after it finishes. Pass `--shutdown` only if the user explicitly asks to stop the machine.

> ⚠️ **MONITORING THE PIPELINE — USE THE EVENTS LOG, NOT CONSOLE OUTPUT.**
>
> The pipeline takes several minutes (GPU startup + transcription) and the agent must see every step as it happens — not just completion. The pipeline writes a structured event log specifically for this:
>
> ```
> /tmp/subtitles_pipeline/{NNNN}_{name}.events.jsonl   ← live, append-only
> /tmp/subtitles_pipeline/{NNNN}_{name}.status         ← latest snapshot
> /tmp/subtitles_pipeline/{NNNN}_{name}.done           ← terminal marker
> ```
>
> Each line of `.events.jsonl` is a JSON event: `pipeline_start`, `step_start`, `step_heartbeat` (every 30s during a step — liveness signal), `step_done`, `step_warn`, `step_failed`, `studio_state`, `pipeline_complete`. **Arm this Monitor immediately after starting the pipeline:**
>
> ```
> Monitor(
>     command="tail -F -n +1 /tmp/subtitles_pipeline/{NNNN}_{name}.events.jsonl 2>/dev/null | awk '{print; fflush()} /\"pipeline_complete\"/{exit}'",
>     description="Lightning pipeline — {NNNN}_{name}",
>     timeout_ms=1800000,
>     persistent=False,
> )
> ```
>
> Every event line becomes one notification — you see step starts, completions, warnings, and failures live. `awk` exits on the final `pipeline_complete` event so the Monitor stops cleanly. **Never** use `tail -f | grep` on the console log — that approach has wasted hours and is no longer supported.

Do not start analysis with manual / YouTube subtitles "to save time" — every timestamp will need to be redone once the word-level file lands, because YouTube's wording does not match WhisperX's.

## Before You Start — Ask the User

Once the prerequisite files exist, ask the user:

1. **Shorts or posts?**
   - `shorts` — extract video clips from strong moments
   - `posts` — write content from the source material

2. **If posts: summary or opinion?**
   - `summary` — faithful distillation of what the source says, with citations
   - `opinion` — your own voice, your own argument, source as evidence

Then proceed to the workflow.

## Steps

1. Always start with the shared analysis — → [Workflow](./references/workflow.md)
2. Then follow the path the user chose:
   - Shorts → [Shorts](./references/shorts.md)
   - Posts → [Posts](./references/posts.md)

## See Also

→ [video-downloader skill](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/.agents/skills/video-downloader/SKILL.md) — if you still need to download the video first
→ [subtitle-informer skill](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/.agents/skills/subtitle-informer/SKILL.md) — the tool used to read and navigate the transcript
