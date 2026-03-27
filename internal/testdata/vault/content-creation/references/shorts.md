# Shorts — Video Clip Extraction

A short is the speaker's own words. You don't reframe — you find the moment and give the timestamp.

## What makes a strong short

- Self-contained — no context needed to understand it
- Punchy — one clear idea, delivered with energy
- Provocative — makes the viewer stop and think
- Short — ideally under 60 seconds

## Before cutting — confirm with the user

Before running `short_Maker`, present your selection to the user:

```
Clip 1: 00:28:16 → 00:28:25
Quote: "..."
Why: one sentence on why this moment works as a short
```

Wait for the user to confirm before cutting. They may want to adjust timestamps, skip a clip, or reorder.

## Getting timestamps

Use `locate` with a short snippet from the quote:
```bash
subtitle_Informer locate --file en_Word.vtt --text "house of cards"
```

Add ~3 seconds of buffer before and after the returned timestamp when cutting.

## Cutting the clip

Once the user confirms, use `short_Maker` to cut the clip, convert it to portrait, and burn the subtitles in:

```bash
short_Maker \
  --video /home/jav/Videos/01_Recordings/{video}/video_File.mp4 \
  --subs  /home/jav/Videos/01_Recordings/{video}/Subtitles/WhisperX/en_Word.ass \
  --from 00:28:16 --to 00:28:25
```

→ [short_Maker agent guide](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/Project_Manag/Docs/Descr/Short_Maker/agent_Guide.md)

## Rating a clip

| Signal | Strong | Weak |
|--------|--------|------|
| Stands alone | Yes | Needs intro |
| Delivery | Energetic, clear | Flat, rambling |
| Idea | Concrete, specific | Vague |
| Reaction | Will provoke response | Forgettable |
