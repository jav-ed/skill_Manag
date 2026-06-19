# sudo Workflow

Claude Code runs in a headless subprocess with no TTY — `sudo` cannot prompt for a password there. The user has to type the password into a real terminal *somewhere*. There are two options:

## ✅ Preferred: pre-cache with `sudo -v`

Ask the user to open a separate terminal and run:

```bash
sudo -v
```

They type their password once. `sudo` caches credentials for ~15 minutes (default). During that window, the agent can run `sudo` commands directly via the Bash tool — no tmux needed.

When the cache expires, ask the user to re-run `sudo -v`. This will inshallah work for most admin tasks, since most sudo work is bursty and fits inside the 15-minute window.

**Why this is preferred:** no tmux session to manage, no attach/detach for the user, no file+sentinel timing to tune. It's the cleanest path.

## Fallback: tmux + attach

Use this only when pre-caching is not workable — for example, the user is already driving the agent inside the tmux session and cannot easily open a side terminal.

1. Tell the user to attach if not already: `tmux attach -t <session>`
2. Send the sudo command — the password prompt appears in their attached terminal
3. Use file+sentinel with a longer poll timeout to allow time for password entry

```bash
rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt

# Send command — user will see the password prompt in their attached terminal
tmux send-keys -t '%X' '{ sudo systemctl restart someservice; } > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter

# Poll longer (20s) to give the user time to type the password
for i in $(seq 1 40); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done

cat /tmp/tmux_out.txt
```

Once the user types their password, `sudo` caches it for ~15 minutes inside that pane — subsequent sudo commands in the same pane will not prompt again.

Note: this fallback only applies to **local** sudo. For sudo on a remote machine reached via `ssh`, follow the remote-pane guidance in [output_Capture.md](output_Capture.md) — file+sentinel does not work across the ssh boundary.

## Alternative: NOPASSWD sudoers rule

For ongoing admin work, add a NOPASSWD rule so sudo never prompts. This removes all friction — sudo works without any user interaction at all.

```bash
echo 'username ALL=(ALL) NOPASSWD: ALL' | sudo tee /etc/sudoers.d/username-nopasswd
```

Remove when no longer needed:

```bash
sudo rm /etc/sudoers.d/username-nopasswd
```
