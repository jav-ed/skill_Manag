# Shorts — Video Clip Extraction

A short is the speaker's own words. You don't reframe — you find the moment and give the timestamp.

## What makes a strong short

- Self-contained — no context needed to understand it
- One clear idea, delivered with energy
- Under 60 seconds ideally

## Cutting the clip

**`short_Maker` requires `Subtitles/WhisperX/en_Word.ass`. Manual YouTube SRT (`Subtitles/Yt/en_Manual.srt`) will not work — the tool burns word-level karaoke captions and fails immediately without a word-level ASS file.** If `en_Word.ass` is missing, stop and run the AI subtitle pipeline (see [content-creation prerequisites](../SKILL.md)) before cutting anything.

Use `locate` to find exact timestamps, then pass them directly to `short_Maker`:

```bash
short_Maker \
  --video /home/jav/Videos/01_Recordings/{video}/video_File.mp4 \
  --subs  /home/jav/Videos/01_Recordings/{video}/Subtitles/WhisperX/en_Word.ass \
  --from 00:28:16 --to 00:28:25
```

Output is saved automatically to a `Shorts/` subfolder:
```
Created: /home/jav/Videos/01_Recordings/{video}/Shorts/28_16_To_28_25_video_File.mp4
```

Optional descriptive name via `--out` — timestamp first, then topic, every word capitalised:
```bash
  --out /home/jav/Videos/01_Recordings/{video}/Shorts/28_16_To_28_25_No_Ceasefire.mp4
```

## Writing the caption

Save a `.txt` file with the exact same name as the video. Most viewers never read captions — they exist for the algorithm. Keep it short and accurate.

- No em dashes
- We stick to the actual content, do not exaggerate, do not invent; accuracy protects reputation
- If you need more context, use `slice` with a wider window to read the surrounding transcript — but cut the video to the exact timestamps given
- If the content challenges mainstream narratives, frame the caption as a question the video answers rather than a statement. Avoid hashtags known to suppress content on specific platforms (e.g. Gaza on TikTok)
