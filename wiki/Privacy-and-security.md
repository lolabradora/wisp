This page states what wisp reads, where that data goes and what it leaves on disk.

## Summary

Everything wisp reads stays on your machine. The model runs locally and wisp sends no telemetry.

## Network traffic

wisp makes two kinds of connections:

- **Loopback HTTP to llama-server.** The daemon, `wisp status`, `wisp doctor`, `wisp complete` and the menu bar item talk to `WISP_LLM_URL`, which defaults to `http://127.0.0.1:8012`. The service installed by `wisp setup` binds llama-server to `127.0.0.1`, so other machines cannot reach it.
- **A unix socket** between your shells and the daemon, at `~/.cache/wisp/wisp.sock`.

There is one download. During `wisp setup`, llama.cpp fetches the model weights from Hugging Face. `llama-server` makes that request before wisp has read anything from your terminal. `install.sh` also downloads the wisp binary from GitHub releases when you run it.

If you set `WISP_LLM_URL` to another host, your context goes to that host. Keep it on loopback unless you mean to do that.

## What is sent to the local model

After each command, the daemon builds a prompt from:

- **The screen tail:** the last 60 lines of the pane, 200 characters per line, when a screen source is available.
- **The history tail:** the last 30 unique commands from your zsh history file and the current session.
- **File names:** up to 40 entries of the working directory. wisp lists the names and never opens the files.
- **Branch names:** the current git branch and up to 10 recent local branches.
- The working directory path, and the last command's exit code if it failed.

While you type, the current command line is added to that prompt.

**Secrets are not filtered.** If a token is printed on screen, appears in your recent history or is being typed on the command line, it is part of the prompt and reaches llama-server on your machine. It does not leave the machine. If that is still more than you want during a sensitive task, pause wisp first.

To see exactly what would be sent from your current shell:

```bash
wisp complete --show-prompt ""
```

## The socket

The daemon creates `~/.cache/wisp` with mode `0700`, so only your user can reach the socket, the log and the pause flag inside it. Other local users cannot send requests to your daemon or read its replies. A process running as you can; wisp does not defend against your own account.

One thing to know about that last point: each shell passes its `WISP_SCREEN_CMD` to the daemon, and the daemon runs it with `sh -c`. Anything that can write to the socket can therefore make the daemon run a command as you. The `0700` directory limits that to processes that already run as you.

## What is stored on disk

| Path | Contents |
| --- | --- |
| `~/.cache/wisp/wisp.sock` | The daemon's socket |
| `~/.cache/wisp/wisp.log` | The daemon log |
| `~/.cache/wisp/paused` | Empty file, present while paused |
| `~/Library/Logs/wisp-llama-server.log` (macOS) | llama-server's own output |
| `journalctl --user -u wisp-llama-server` (Linux) | llama-server's own output |

wisp does not write prompts or screen captures to disk.

At the default log level (`wisp=info`), the daemon log holds startup lines, warnings and errors. A warning about a failed request includes the command line you had typed at that moment. With `WISP_LOG=wisp=debug`, the log records every buffer sent to the model and every suggestion returned, so it will contain whatever you type. The log is appended to and not rotated; delete it whenever you like.

The daemon also keeps the last 20 suggestions in memory, with what you had typed and the latency. `wisp status` and the menu bar read them from there. They are gone when the daemon stops.

The llama-server log belongs to llama.cpp. What it records depends on the llama.cpp version and its verbosity flags.

## Pausing

```bash
wisp pause
```

While paused, the daemon does not capture the screen, read history or call the model. It still receives the plugin's messages, and it still records the commands you finish in memory so that context is ready when you resume. Run `wisp resume` to turn suggestions back on. To make sure nothing is running at all, also stop the daemon with `pkill -f "wisp daemon"`; with the pause flag in place, a restarted daemon stays paused.
