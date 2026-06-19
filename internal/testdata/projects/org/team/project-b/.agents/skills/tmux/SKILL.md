---
name: tmux
description: Use for interactive sessions the user is watching (TUI apps, dev servers), remote shells (ssh into another machine), or local sudo prompts. Skip for regular commands — Bash already has a terminal.
version: 0.2.1
---

# tmux Terminal Interaction

Most commands do not need tmux. The agent's Bash tool already has a working terminal. Reach for tmux only in these three cases:

1. **Interactive sessions the user is watching** — TUI applications (htop, vim, lazygit), dev servers, anything where the user wants to see history scroll by or jump in and type commands themselves.
2. **Remote shells** — a pane logged into another machine (`ssh g12`, `ssh some-vps`, etc.). The agent has no other way to drive a session on a remote host.
3. **Local sudo when credentials are not pre-cached** — see [sudo_Workflow.md](references/sudo_Workflow.md). The preferred path is to ask the user to run `sudo -v` in a separate terminal, which removes the need for tmux entirely.

For everything else, use the Bash tool directly.

## ⚠️ The /tmp gotcha for remote panes

The file+sentinel pattern below writes to `/tmp/tmux_out.txt` and reads it back from the agent's Bash tool. This only works when the pane and the agent share the same filesystem.

If the pane is sitting in an `ssh` session, `send-keys` runs on the **remote** host — the file lands in the remote `/tmp/`, but the agent's `cat /tmp/tmux_out.txt` reads the **local** `/tmp/` and finds nothing. **There is no error — the local file is just absent and the poll loop times out silently.** Verified live against `ssh g12`.

For remote panes, use `capture-pane` instead. It grabs whatever the pane is currently showing regardless of which machine produced it. See [output_Capture.md](references/output_Capture.md).

## ⚠️ Brace-group multi-command sequences

Shell redirection binds tightly. `a && b && c > /tmp/out.txt 2>&1` only redirects `c`; the output of `a` and `b` goes to the pane and is lost from the file. Wrap any chain in `{ ...; }`:

```bash
# WRONG — only `whoami` ends up in the file
tmux send-keys -t '%X' 'hostname && uname -a && whoami > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter

# RIGHT — all three commands captured
tmux send-keys -t '%X' '{ hostname && uname -a && whoami; } > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter
```

## References

- [Output capture](references/output_Capture.md) — which capture method to use and when (file+sentinel for local, capture-pane for remote, what to avoid)
- [sudo workflow](references/sudo_Workflow.md) — `sudo -v` pre-cache (preferred) and the tmux+attach fallback
- [Session & pane management](references/session_Pane.md) — creating, splitting, and killing sessions and panes

## Standard Workflow (local panes only)

```
1. Get pane ID once  → tmux list-panes -a -F "#{session_name}:#{pane_id}"
2. Clear outputs     → rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt
3. Execute           → tmux send-keys -t '%X' '{ cmd; } > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter
4. Poll              → for i in $(seq 1 20); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done
5. Read result       → cat /tmp/tmux_out.txt
```

Reuse the same pane ID throughout the session — IDs are stable for the lifetime of the pane.

For remote panes (case 2 above), skip steps 2–5 and use `tmux capture-pane -p -t '%X' -S -50` after sending the command.

## Quick Reference

| Task | Command |
|------|---------|
| List sessions | `tmux list-sessions` |
| List panes | `tmux list-panes -a -F "#{session_name}:#{pane_id}"` |
| Send command | `tmux send-keys -t '%X' 'cmd' Enter` |
| Capture (remote or short output) | `tmux capture-pane -p -t '%X' -S -50` |
| Capture (local, reliable) | file + sentinel — see [output_Capture.md](references/output_Capture.md) |
| Attach (for sudo) | `tmux attach -t name` |
| Create session | `tmux new-session -d -s name` |
| Kill session | `tmux kill-session -t name` |
