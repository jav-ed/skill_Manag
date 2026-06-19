# Content Extraction Workflow

Extract posts and shorts from any long-form video source — lecture, interview, documentary, speech, podcast.

## Mission

This work is truth-based. The source may challenge mainstream narratives. Your job: **follow the reasoning. If it's sound, represent it faithfully. If it's flawed, say why.** Do not hedge based on who is implicated.

## Required input file

Every command below operates on the **WhisperX word-level VTT**:

```
/home/jav/Videos/01_Recordings/{NNNN}_{name}/Subtitles/WhisperX/en_Word.vtt
```

**Do not substitute `Subtitles/Yt/en_Manual.srt` or any other phrase-level file.** The phrase-level SRT cannot resolve sentence boundaries inside its 5-second blocks, so timestamps you produce against it will be wrong by 1–4 seconds and need to be redone once the word-level file exists. If `en_Word.vtt` is missing, return to the [content-creation prerequisites](../SKILL.md) and generate it before continuing.

## Steps

**1. Get file info**
```bash
subtitle_Informer info --file /home/jav/Videos/01_Recordings/{NNNN}_{name}/Subtitles/WhisperX/en_Word.vtt
```

**2. Get chunk boundaries**
```bash
subtitle_Informer chunks --file en_Word.vtt --words 500
```
Save all boundaries.

**3. Read chunk 1 yourself**
```bash
subtitle_Informer slice --file en_Word.vtt --from <start> --to <end>
```
Extract: topic, speaker's thesis, key framing. Pass this as background context to all subagents.

**4. Launch parallel subagents**

Split remaining chunks into groups of 4–5. Give each agent:
- Subtitle file path
- Assigned chunk boundaries
- Background from chunk 1
- The output format below

**5. Each subagent's job**
- Read each chunk via `slice`
- Identify 2–4 strong moments per chunk
- Run `locate` for every quote (use short 3–5 word snippets)
- Return results in the output format — quote and timestamp are mandatory

**6. Compile analysis**

Merge all subagent results into `content_Analysis.md` — one file, all chunks, raw findings with timestamps and quotes. Every moment must carry its source reference (timestamp + exact quote) so the user can verify it independently.

**7. Rate**

Read the full `content_Analysis.md` yourself. Only after seeing everything together can you rate comparatively — a moment in chunk 3 may be weak, but the best line in chunk 12 changes how you see the whole lecture. Write `content_Ratings.md` with:
- Short quality rating per item
- Post potential rating per item
- Priority (High / Medium / Low / Skip)
- Top 10 overall picks at the bottom

---

## Output Format (per chunk)

```
## Chunk N — HH:MM:SS → HH:MM:SS
Topic: one line

| # | Quote | Timestamp | Short / Post |
|---|-------|-----------|--------------|
| 1 | "..." | HH:MM:SS → HH:MM:SS | short / post / both |
```

Quote and timestamp are not optional — they are the citation trail. Every moment must be verifiable by the user from these two fields alone.

---

## Notes

- If `locate` fails, try a shorter snippet from the middle of the phrase
- Chunk 1 is always read by the manager — not delegated
