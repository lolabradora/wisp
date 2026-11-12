# wisp

Grey "ghost text" command suggestions for zsh, written by a small local model that sees what you
see: the terminal output above the prompt, your shell history, the files in the current directory
and your git branches.

```
% git che|ckout feat/login-page      ← grey part is the suggestion; → accepts it
```

- **After a command finishes**, wisp predicts the next one while the prompt is still empty.
- **While you type**, it completes the current line after a 60 ms pause in typing.
- **It completes names that are on your screen**: the pod that is crash-looping in
  `kubectl get pods`, the PID from `lsof`, the path in a stack trace.
- Everything runs locally. Nothing leaves your machine.

[Landing page](https://lolabradora.github.io/wisp/) · [Wiki](https://github.com/lolabradora/wisp/wiki)

## Install

wisp runs on macOS and Linux, in zsh. The model runs in
[llama.cpp](https://github.com/ggml-org/llama.cpp)'s `llama-server`.

```bash
brew install llama.cpp        # Linux without Homebrew: a llama.cpp release binary on your PATH
curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
wisp setup
echo 'eval "$(wisp init zsh)"' >> ~/.zshrc
```

1. `install.sh` downloads the prebuilt `wisp` binary for your OS and CPU into `~/.local/bin`
   (set `WISP_INSTALL_DIR` to change that). To build from source instead:
   `cargo install --git https://github.com/lolabradora/wisp`.
2. `wisp setup` downloads Qwen2.5-Coder 1.5B (1.65 GB) on the first run and installs a background
   service that keeps `llama-server` running on `127.0.0.1:8012`: a launchd agent on macOS, a
   systemd user unit on Linux.
   - `wisp setup --model 3b` uses Qwen2.5-Coder 3B (3.3 GB). Its suggestions are longer and more
     complete; it answers in ~150–450 ms, versus ~50–120 ms for 1.5B. Run `wisp setup` again to
     switch back.
   - `wisp setup --menubar` also adds the macOS menu bar item (see below).
3. The `eval` line loads the zsh plugin. Open a new shell and run `wisp doctor` to check every
   part.

Accept a suggestion with → / End / Ctrl-F. If
[zsh-autosuggestions](https://github.com/zsh-users/zsh-autosuggestions) is loaded before wisp,
wisp shows its suggestions through it and uses its keys and colors. Without it, wisp draws the
grey text itself.

`wisp uninstall` removes the services. It leaves the binary and the model weights in place.

## Terminal support

wisp reads the screen through whichever of these it finds first:

| Source           | Needs                                                              |
| ---------------- | ------------------------------------------------------------------ |
| `WISP_SCREEN_CMD`| A command of yours that prints the visible screen                  |
| tmux             | Nothing. Works in any terminal: Ghostty, iTerm2, Alacritty, …      |
| WezTerm          | Nothing                                                            |
| kitty            | `allow_remote_control` and `listen_on` in `kitty.conf`. Experimental |

In any other setup wisp still uses history, the directory listing, git branches and the commands
it saw finish, with their exit codes. `wisp doctor` shows which source your shell uses. See
[Terminal support](https://github.com/lolabradora/wisp/wiki/Terminal-support) in the wiki.

## How it works

```
zsh plugin ──unix socket──▶ wisp daemon ──HTTP──▶ llama-server (Qwen2.5-Coder)
 (keystrokes, finished        (context, debounce,
  commands)                    cancellation)
```

- **zsh plugin** (`shell/wisp.zsh`).
  - After each command, it sends the command, its exit code, the cwd and which terminal it runs in.
  - While you edit the line, it sends the current buffer.
  - It shows replies as grey text after the cursor.
- **Daemon** (`wisp daemon`). The plugin starts it on demand.
  - After each command it gathers context:
    - the last 60 lines of the terminal screen;
    - recent history;
    - the cwd listing;
    - git branches.
  - It turns that context into a shell transcript that ends in `$ <what you typed>`, and a base
    code model continues that line.
  - The word you are typing is forced through a grammar instead of ending the prompt mid-token,
    so the model picks its own tokens for it ("token healing").
  - The transcript's prefix doesn't change while you type, so llama-server reuses its prompt
    cache and processes only the new characters.
  - A new keystroke cancels the request in flight.
  - A suggestion you keep typing along with is reused without asking the model again.

[How it works](https://github.com/lolabradora/wisp/wiki/How-it-works) in the wiki goes further.

## Configuration

| Variable               | Default                 | Purpose                                              |
| ---------------------- | ----------------------- | ---------------------------------------------------- |
| `WISP_LLM_URL`         | `http://127.0.0.1:8012` | llama-server the daemon talks to                     |
| `WISP_DEBOUNCE_MS`     | `60`                    | How long typing must pause before the model is asked |
| `WISP_SCREEN_CMD`      | unset                   | Command that prints the terminal screen              |
| `WISP_HIGHLIGHT_STYLE` | `fg=8`                  | Ghost-text style without zsh-autosuggestions         |
| `WISP_LOG`             | `wisp=info`             | Log filter, e.g. `wisp=debug`                        |

The plugin starts the daemon with the environment of your shell. After you change
`WISP_LLM_URL`, `WISP_DEBOUNCE_MS` or `WISP_LOG`, restart the daemon: `pkill -f "wisp daemon"`.
The next prompt starts it again.

## Status, pausing and the menu bar

```bash
wisp doctor          # checks llama-server, the daemon, ~/.zshrc and the screen source
wisp status          # llama-server, daemon, recent suggestions (--json for scripts)
wisp pause           # no suggestions until `wisp resume`; survives restarts
wisp resume
```

On macOS, `wisp setup --menubar` adds a menu bar item. Its title reads `wisp 3B · 180ms`: the
model size and the median latency. It changes to `wisp ⏸` while suggestions are paused and to
`wisp ⚠` when llama-server is down. The menu shows the loaded model and its memory use, how many
shells are connected, and the last 10 suggestions with their latency. From the menu you can pause
suggestions, restart llama-server or open either log.

macOS hides menu bar items that do not fit, which happens often on notched MacBooks. If the item
is missing, click `«` in the menu bar or check System Settings → Menu Bar.

## Debugging

```bash
wisp complete "git che"                 # suggestion + latency, using this dir and screen
wisp complete --show-prompt "git che"   # also print what the model sees
tail -f ~/.cache/wisp/wisp.log          # daemon log
```

llama-server logs to `~/Library/Logs/wisp-llama-server.log` on macOS and to the journal on Linux
(`journalctl --user -u wisp-llama-server`). The wiki has a
[troubleshooting page](https://github.com/lolabradora/wisp/wiki/Troubleshooting).

## Landing page

`site/` is the landing page. It is static HTML and CSS with no build step; a workflow deploys it
to GitHub Pages. To preview it locally:

```bash
python3 -m http.server -d site 8000
```

Its visual system is recorded in `DESIGN.md`, and the product facts it may claim are in
`PRODUCT.md`.

## Development

```bash
cargo test          # unit tests, daemon tests against a fake llama-server, zsh end-to-end test
prek run            # fmt, clippy -D warnings, tests
```

The end-to-end test drives a real zsh inside tmux and is skipped when either is missing.
`wiki/` holds the source of the GitHub wiki.

## License

MIT. Made by [HEYLOLA](https://github.com/lolabradora).
