This page maps symptoms to causes and fixes, using `wisp doctor`, `wisp status` and the logs.

## Start here

Run these in the shell that has the problem:

```bash
wisp doctor                             # checklist with a fix hint per failure
wisp status                             # llama-server, daemon, pause state, recent suggestions
wisp complete "git che"                 # one suggestion with timings, bypassing the plugin
tail -n 50 ~/.cache/wisp/wisp.log       # daemon log
```

`wisp doctor` checks, in order: the `llama-server` binary, whether llama-server is reachable and which model it loaded, the daemon, the pause state, the `wisp init zsh` line in `~/.zshrc`, and the screen source for this shell. Lines are marked ✓ (ok), ✗ (failed) or – (not applicable).

llama-server's own log:

```bash
tail -n 50 ~/Library/Logs/wisp-llama-server.log        # macOS
journalctl --user -u wisp-llama-server -n 50           # Linux
```

## No ghost text at all

| Cause | How to tell | Fix |
| --- | --- | --- |
| Plugin not loaded | `wisp doctor` fails the `~/.zshrc` check | Add `eval "$(wisp init zsh)"` to the end of `~/.zshrc` and open a new shell |
| `wisp` not on `PATH` | The shell prints `command not found: wisp` at startup | Add `~/.local/bin` (or your `WISP_INSTALL_DIR`) to `PATH` above the `eval` line |
| Suggestions paused | `wisp status` shows `suggestions: paused` | `wisp resume` |
| llama-server down | `wisp status` shows `llama-server: down (...)` | See [llama-server is down](#llama-server-is-down-or-still-loading) |
| Daemon not running | The shell printed `wisp: daemon did not start` | See [the next section](#wisp-daemon-did-not-start) |
| Ghost text drawn in an invisible colour | `wisp status` lists recent suggestions but you see none | Without zsh-autosuggestions, set `WISP_HIGHLIGHT_STYLE` to a colour your theme shows, for example `fg=244`. With it, adjust `ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE` |
| The model had nothing to add | `wisp status` shows `→ (nothing)` for the line | Nothing to fix. This is a normal answer |

Also note where suggestions never appear: wisp only suggests when the cursor is at the end of the line, and it skips multi-line buffers.

## "wisp: daemon did not start"

The plugin tried to start `wisp daemon`, waited about a second and could not connect. It retries after 60 seconds, so one broken start does not stall every prompt.

1. Read the reason:

   ```bash
   tail -n 20 ~/.cache/wisp/wisp.log
   ```

2. Run the daemon in the foreground to see the error directly:

   ```bash
   wisp daemon
   ```

   Stop it with Ctrl-C once you have seen the output.

Common causes are a `wisp` binary that was moved or deleted after the shell started (open a new shell so `wisp init zsh` records the new path) and a `~/.cache/wisp` directory your user cannot write to.

`another wisp daemon is already listening` in the log is harmless: two shells raced to start the daemon and one won.

## llama-server is down or still loading

`wisp status` gives the reason in parentheses.

**`no llama-server at http://127.0.0.1:8012`**: nothing is listening.

- If you have not run `wisp setup`, run it.
- Check the service:

  ```bash
  launchctl print gui/$(id -u)/dev.wisp.llama-server | head -n 20     # macOS
  systemctl --user status wisp-llama-server                           # Linux
  ```

- Restart it:

  ```bash
  launchctl kickstart -k gui/$(id -u)/dev.wisp.llama-server           # macOS
  systemctl --user restart wisp-llama-server                          # Linux
  ```

  On macOS the menu bar item has a "Restart llama-server" entry that does the same.

- If the service keeps exiting, read the llama-server log. A port already in use and a missing or moved `llama-server` binary (for example after a Homebrew upgrade changed its path) are the usual reasons. Re-running `wisp setup` rewrites the service with the current path.
- If you use a non-default port or server, check that `WISP_LLM_URL` matches; see [[Configuration]].

**`llama-server is not ready (still loading the model?)`**: the server is up but the model is not loaded yet. This lasts a few seconds after login or a restart. If it does not clear, read the llama-server log; running out of memory while loading is the likely cause, and the smaller model is the fix: `wisp setup --model 1.5b`.

## Suggestions are slow

Measure first:

```bash
wisp complete "git che"
```

It prints two timings: `cold` (first request with this prompt) and `cached` (the same request again), plus how long gathering context took. `wisp status` shows the median over recent suggestions and the latency of each.

For reference, on an Apple M5 with 24 GB we measured about 50–120 ms cached with 1.5B, about 150–450 ms cached with 3B, and about 180–280 ms for a cold first request. Older or smaller machines will be slower.

| What you see | Likely cause | Fix |
| --- | --- | --- |
| `cached` is slow | The model is too large for the machine | `wisp setup --model 1.5b` |
| Only the first suggestion after each command is slow | Cold prompt: new context has to be evaluated once | Expected. A smaller model shortens it |
| Context gathering takes hundreds of ms | A slow screen command or a slow `git` in a huge repo; each probe is cut off at 500 ms | Speed up or unset `WISP_SCREEN_CMD`; check `git status` speed in that repo |
| Everything is slow under load | llama-server is competing for CPU/GPU or the machine is swapping | Check memory pressure; the 3B server used about 4.3 GB in our measurement |
| Suggestions feel late while typing fast | The debounce waits for a pause | Lower `WISP_DEBOUNCE_MS`, at the price of more model calls |

## Suggestions ignore what is on screen

wisp has no screen source for this shell, or the capture returns nothing.

1. `wisp doctor` names the detected source and whether a capture returned text.
2. `wisp complete --show-prompt "x"` should print a `# Terminal screen` section. If it is missing, there is no screen context.

Fixes by terminal, with details in [[Terminal support]]:

- **Not in tmux, WezTerm or kitty:** run the shell inside tmux, or set `WISP_SCREEN_CMD`.
- **kitty (experimental):** remote control is off or kitty is not listening. Check that `echo $KITTY_LISTEN_ON` prints a socket path. If it is empty, add `allow_remote_control yes` and `listen_on unix:/tmp/kitty-{kitty_pid}` to `kitty.conf` and restart kitty.
- **WezTerm:** `wezterm` must be on the daemon's `PATH`. Restart the daemon from a shell where `command -v wezterm` works: `pkill -f "wisp daemon"`.
- **`WISP_SCREEN_CMD`:** run `sh -c "$WISP_SCREEN_CMD"` by hand. It must print text and finish within 500 ms.

Even with a working source, the model only sees the last 60 lines and may still pick something else. Screen context makes the right completion more likely without guaranteeing it.

## Suggestions stopped and the menu bar shows `wisp ⏸`

wisp is paused, by `wisp pause` or the menu. The pause survives restarts. Run `wisp resume`.

## The menu bar item is missing (macOS)

macOS hides menu bar items that do not fit, which happens often on MacBooks with a notch.

- Quit a few other menu bar apps and see whether the item appears, or check System Settings → Menu Bar.
- Confirm the agent is running:

  ```bash
  launchctl print gui/$(id -u)/dev.wisp.menubar | head -n 20
  ```

- Choosing "Quit wisp menu" stops the item until the next login. `wisp setup --menubar` brings it back.

## Conflicts with other plugins

Most conflicts are about load order. Put the `eval "$(wisp init zsh)"` line at the end of `~/.zshrc`.

- **zsh-autosuggestions** must be loaded before wisp if you use both. If it loads after wisp, wisp has already bound its own accept keys and drawing, and the two can interfere. In the right order you may see its history suggestion for a moment before the model's suggestion replaces it.
- **Plugins that rebind →, End, Ctrl-F or Alt-F** after wisp take those keys away from wisp's own accept widgets when zsh-autosuggestions is not loaded. Load wisp after them.
- **Prompt and exit status.** wisp puts its `precmd` hook first so that `$?` is still the exit status of your command. Anything that later prepends itself to `precmd_functions` and runs commands can make wisp report the wrong status.

wisp has been used alongside zsh-vi-mode, fzf-tab and starship.

## Still stuck

Open an issue at https://github.com/lolabradora/wisp/issues with the output of `wisp doctor`, `wisp status` and the last lines of `~/.cache/wisp/wisp.log`. Read the log before posting it: it can contain command lines you typed.
