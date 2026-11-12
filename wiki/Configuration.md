This page lists wisp's settings, says when each one takes effect, and shows how to pause wisp or point it at a different model server.

## Environment variables

wisp has no config file. Set these in `~/.zshrc`, above the `eval "$(wisp init zsh)"` line.

| Variable | Default | Purpose |
| --- | --- | --- |
| `WISP_LLM_URL` | `http://127.0.0.1:8012` | llama-server the daemon talks to |
| `WISP_DEBOUNCE_MS` | `60` | How long typing must pause, in milliseconds, before the model is asked |
| `WISP_SCREEN_CMD` | unset | Custom command that prints the terminal screen; see [[Terminal support]] |
| `WISP_HIGHLIGHT_STYLE` | `fg=8` | Ghost-text style when zsh-autosuggestions is not loaded |
| `WISP_LOG` | `wisp=info` | Log filter, for example `wisp=debug` |

Example:

```zsh
export WISP_DEBOUNCE_MS=100
export WISP_LOG=wisp=debug
eval "$(wisp init zsh)"
```

`WISP_HIGHLIGHT_STYLE` takes a zsh highlight spec, the same format as `ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE`: `fg=8`, `fg=244`, `fg=#6c7086,italic`. When zsh-autosuggestions is loaded, its own style setting applies and this variable is ignored.

## When changes take effect

One daemon serves all your shells. The first shell that needs it starts it, and the daemon keeps the environment of that shell. Changing a variable in a second shell does not change the running daemon.

After you change `WISP_LLM_URL`, `WISP_DEBOUNCE_MS` or `WISP_LOG`, restart the daemon from a shell that has the new values:

```bash
pkill -f "wisp daemon"
```

The next prompt starts it again.

`WISP_SCREEN_CMD` and `WISP_HIGHLIGHT_STYLE` belong to the shell, not the daemon. The plugin sends the screen command to the daemon with each finished command, so a new value applies to shells opened after the change.

`wisp status`, `wisp doctor` and `wisp complete` read `WISP_LLM_URL` from the shell you run them in. If they disagree with what you see while typing, the daemon was probably started with a different value.

## Pause and resume

```bash
wisp pause           # no suggestions until `wisp resume`
wisp resume
```

Pausing creates the file `~/.cache/wisp/paused`. The daemon checks for it on every message, so the pause applies to all shells at once and survives daemon restarts and reboots. While paused, the daemon does not read the screen and sends nothing to the model. On macOS the menu bar item has the same switch.

## Status

```bash
wisp status          # llama-server, daemon, pause state, recent suggestions
wisp status --json   # the same report for scripts
```

## Using a different llama-server or model

`wisp setup` installs a service for Qwen2.5-Coder 1.5B or 3B. To switch between those two, run `wisp setup --model 3b` or `wisp setup --model 1.5b`.

To use anything else, run your own llama-server and point wisp at it:

```zsh
export WISP_LLM_URL=http://127.0.0.1:9000
```

Then restart the daemon as shown above.

The server must be a llama.cpp `llama-server` that supports `POST /completion` with a GBNF `grammar` field. wisp also calls `GET /health` and `GET /props` for status. It sends a raw prompt, a grammar, `temperature: 0`, `cache_prompt: true`, a 48-token limit and a newline stop.

Pick a base or fill-in-the-middle code model. wisp's prompt is a shell transcript that the model continues, sent raw with no chat template. Instruction-tuned chat models tend to continue such a prompt poorly. [[How it works]] has the reasoning.

Example with a model file of your own:

```bash
llama-server -m /path/to/model.gguf --host 127.0.0.1 --port 9000
```

Keep the server bound to `127.0.0.1`. wisp sends it your screen and history; see [[Privacy and security]]. If you set `WISP_LLM_URL` to a host on another machine, that context goes over the network to it.
