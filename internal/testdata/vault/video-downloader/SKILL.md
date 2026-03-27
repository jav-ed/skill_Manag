---
name: video-downloader
description: Download YouTube videos — standalone or as part of a workflow to generate subtitles and create content (shorts, posts)
---

# Video Downloader

## Downloading a Video

```sh
uv run python Code/main.py "VIDEO_URL"
```

This gives you a folder under `Output/` with the video, an audio sidecar, English subtitles, and a metadata file. Defaults are tuned for editing — no configuration needed.

If you need to change something (language, format, mode):

| Option | Default | Description |
|--------|---------|-------------|
| `--mode` | `auto` | Download mode — [details](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/09_Vid_Down/Project_Manag/Docs/Architecture/current_Workflow.md) |
| `--subtitle-langs` | `en` | Comma-separated language codes |
| `--subtitle-format` | `srt` | `srt` or `vtt` — [why srt?](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/09_Vid_Down/Project_Manag/Docs/Investigations/subtitle_Format.md) |
| `--write-info-json` | off | Raw yt-dlp debug metadata |

## Full Workflow — Subtitles + Content Creation

If the user wants more than just the download — generating subtitles and creating content (shorts, posts) from the video — downloading is the first step of a larger workflow. Follow the skills in this order:

**1. Download the video** — as described above.

**2. Generate AI subtitles** — prepare the audio and run the transcription pipeline:

```bash
# From 01_Subtitle root:
uv run python Code/Lightning/prepare_Audio.py {video_name}
uv run python Code/Lightning/run_Pipeline.py {video_name}
```

Full details (upload order, troubleshooting, individual steps):
→ [AI Subtitle Workflow](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/Project_Manag/Docs/Descr/Lightning_AI/workflow.md)

**3. Read and navigate the transcript** — use subtitle-informer to slice, search, and locate timestamps:
→ [subtitle-informer skill](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/.agents/skills/subtitle-informer/SKILL.md)

**4. Create content** — extract shorts or write posts from the material:
→ [content-creation skill](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/01_Subtitle/.agents/skills/content-creation/SKILL.md)

---

## If Something Is Not Working

- Blocked argument error → [blocked_Passthrough_Arguments.md](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/09_Vid_Down/Project_Manag/Docs/Architecture/blocked_Passthrough_Arguments.md)
- Want to understand what yt-dlp args are passed → [yt_Dlp_Defaults.md](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/09_Vid_Down/Project_Manag/Docs/Architecture/yt_Dlp_Defaults.md)
- Want to understand the full output structure → [current_Workflow.md](/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/02_Online_Presence/3_Social_Media_Platform/09_Vid_Down/Project_Manag/Docs/Architecture/current_Workflow.md)
