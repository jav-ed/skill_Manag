# Output Capture

Pick the method based on **where the pane is logged in**, not just the command type.

## ⚠️ Remote panes (ssh) — file+sentinel does NOT work

If the pane is in an `ssh` session into another machine, `tmux send-keys` runs the command on the **remote** host. A redirect to `/tmp/tmux_out.txt` lands in the **remote** `/tmp/`. The agent's Bash tool then reads the **local** `/tmp/` and finds nothing — silently. No error, just empty output.

**For remote panes, always use `capture-pane`** (see below). It scrapes whatever the pane shows, regardless of which machine produced the text. For long output, increase the scrollback window with `-S -200` or more, and clear history first to avoid mixing in earlier noise.

```bash
tmux send-keys -t '%X' 'clear' Enter
tmux send-keys -t '%X' 'your-remote-command' Enter
sleep 1   # tune to expected runtime
tmux capture-pane -p -t '%X' -S -200
```

## ✅ BEST for local panes: File + Sentinel

Redirect output to a file, use a sentinel to signal completion. Poll instead of sleeping blindly. Wrap your command in `{ ...; }` so the redirect captures every step, not just the last one.

```bash
rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt
tmux send-keys -t '%X' '{ your-command; } > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter

for i in $(seq 1 20); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done
cat /tmp/tmux_out.txt
```

**Why:** reliable completion detection, captures full output regardless of length, no ANSI garbage. Only valid when the pane and the agent share the same filesystem.

### Brace grouping is not optional

`a && b && c > file` only redirects `c`; the output of `a` and `b` goes to the pane and is lost from the file. Verified live: `hostname && uname -a && whoami > /tmp/...` captured only the `whoami` line. Always wrap multi-command sequences:

```bash
# WRONG — only `whoami` ends up in the file
'hostname && uname -a && whoami > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt'

# RIGHT — all three commands captured
'{ hostname && uname -a && whoami; } > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt'
```

For a single command, the braces are harmless — keep them on every send-keys call by default so this never bites you.

## ✅ OK: capture-pane (short output, local or remote)

```bash
tmux send-keys -t '%X' 'your-command' Enter
sleep 0.3
tmux capture-pane -p -t '%X' -S -10
```

Always limit with `-S -N`. Use `-S -5` to `-S -10` by default to avoid filling context. Bump it up for remote panes where this is the only option.

## ❌ AVOID: pipe-pane

Pipes raw terminal output including ANSI escape codes — output is unreadable. Do not use for capturing command output.
