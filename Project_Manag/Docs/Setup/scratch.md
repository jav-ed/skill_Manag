# Setup: Scratch

`Scratch/` holds disposable task notes, screenshots, design previews, logs, generated fixtures, and copied source trees. Reusable code, deployment artifacts, persistent data, and durable documentation belong outside it.

Run `just scratch-clean` when work using these files has finished. It deletes every entry, including hidden files and all Markdown, then recreates empty `Agent_Tasks/`, `Audit/`, `Design/`, and `Screenshots/` folders. There are no age filters or file-type exceptions.

The command targets this repository's Scratch directory. It refuses a symlinked root and removes nested links without following their targets. It is never run automatically during setup, builds, or ordinary development. Preserve lasting findings in their owning docs before cleanup.

The helper script lives at `Code/Development/Scratch/clean.sh` and is tracked by git. The root `.gitignore` line for Scratch is anchored as `/Scratch/`: an unanchored `Scratch/` would also match the helper's own folder and hide it from git.
