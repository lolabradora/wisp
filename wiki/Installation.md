This page covers installing wisp, choosing a model, upgrading and removing it.

## Prerequisites

- macOS (Apple Silicon or Intel) or Linux (x86_64 or aarch64).
- zsh as your interactive shell. Other shells are not supported today; see the [[FAQ]].
- llama.cpp's `llama-server` on your `PATH`. wisp does not bundle it.
- Disk space for one model: 1.65 GB for Qwen2.5-Coder 1.5B, 3.3 GB for 3B.

Install llama.cpp first:

```bash
brew install llama.cpp
```

On Linux without Homebrew, download a build from the [llama.cpp releases](https://github.com/ggml-org/llama.cpp/releases) and put `llama-server` on your `PATH`.

## Install the binary

```bash
curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
```

`install.sh` detects your OS and CPU architecture, downloads the matching prebuilt binary from the latest GitHub release and puts it in `~/.local/bin`. Set `WISP_INSTALL_DIR` to install somewhere else:

```bash
curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | WISP_INSTALL_DIR=/usr/local/bin sh
```

Make sure the install directory is on your `PATH`. Release assets are named `wisp-<target>.tar.gz`, each with a `.sha256` file, for these targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

### Alternative: build with cargo

```bash
cargo install --git https://github.com/lolabradora/wisp
```

This needs a Rust toolchain recent enough for edition 2024 and installs to `~/.cargo/bin`.

## Run `wisp setup`

```bash
wisp setup
```

`wisp setup` does four things in order:

1. **Finds `llama-server`** on your `PATH`. If it is missing, setup stops and prints the install hint.
2. **Downloads the model** on the first run. It starts `llama-server --fim-qwen-<size>-default` in the foreground, waits until the server's `/health` endpoint answers, then stops it. llama.cpp fetches the weights from Hugging Face; this is the only time wisp's setup needs the internet.
3. **Installs a background service** that keeps `llama-server` running on `127.0.0.1:8012`. On macOS this is the launchd agent `dev.wisp.llama-server`. On Linux it is the systemd user unit `wisp-llama-server.service`.
4. **Waits for the service to become healthy**, then prints the line to add to `~/.zshrc`.

Options:

| Flag | Default | Effect |
| --- | --- | --- |
| `--model 1.5b\|3b` | `1.5b` | Which Qwen2.5-Coder size to download and serve |
| `--port N` | `8012` | Port for llama-server on 127.0.0.1 |
| `--menubar` | off | macOS only: also install the menu bar agent |

If you change the port, set `WISP_LLM_URL` to match. See [[Configuration]].

### Choosing a model

Both models are Qwen2.5-Coder, a base code model. The latency and memory figures are internal measurements on an Apple M5 with 24 GB; expect different numbers on other hardware.

| | 1.5B (default) | 3B |
| --- | --- | --- |
| Download size | 1.65 GB | 3.3 GB |
| Latency while typing, cached prompt | about 50–120 ms | about 150–450 ms |
| llama-server memory | not measured | about 4.3 GB |
| Screen-context eval (8 scenarios) | 6/8 | 7/8 |

The first request after a command finishes has no cached prompt and took about 180–280 ms in the same measurements. In our use, 3B wrote longer and more complete commands; 1.5B keeps up with typing more easily. Start with 1.5B and switch if the suggestions feel too short.

Re-running setup replaces the service, so this switches models:

```bash
wisp setup --model 3b
```

## Add wisp to zsh

```bash
echo 'eval "$(wisp init zsh)"' >> ~/.zshrc
```

Open a new shell. The plugin starts the wisp daemon the first time it needs it.

### With or without zsh-autosuggestions

[zsh-autosuggestions](https://github.com/zsh-users/zsh-autosuggestions) is optional.

- **If it is loaded before wisp**, wisp uses it to draw the grey text and you keep its accept keys and its highlight style. Put the `eval` line after the line that loads zsh-autosuggestions.
- **If it is not loaded**, wisp draws the ghost text itself. → and End accept the whole suggestion, as does Ctrl-F when the cursor is at the end of the line. Alt-F (forward-word) accepts one word. The style comes from `WISP_HIGHLIGHT_STYLE`, default `fg=8`.

## Optional: menu bar item (macOS)

```bash
wisp setup --menubar
```

This also installs the launchd agent `dev.wisp.menubar`, which starts `wisp menubar` at login. The title reads `wisp 3B · 180ms`: the model size and the median latency of recent suggestions. It changes to `wisp ⏸` while suggestions are paused and to `wisp ⚠` when llama-server is down.

The menu shows the loaded model and its memory use, how many shells are connected, and the last 10 suggestions with their latency. From it you can pause suggestions, restart llama-server or open either log.

If the item does not appear, see [[Troubleshooting]].

## Check the install

```bash
wisp doctor
```

`wisp doctor` prints one line per check, marked ✓, ✗ or –, with a fix hint next to each failure. It exits non-zero if a required check fails.

## Upgrading

Re-run the installer, or the cargo command with `--force`:

```bash
curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
```

```bash
cargo install --git https://github.com/lolabradora/wisp --force
```

Then stop the old daemon and open a new shell so the plugin and daemon both come from the new binary:

```bash
pkill -f "wisp daemon"
```

You only need to run `wisp setup` again if the release notes say the service changed.

## Uninstalling

1. Remove the background services wisp installed:

   ```bash
   wisp uninstall
   ```

   This does not delete models or the binary.

2. Delete the `eval "$(wisp init zsh)"` line from `~/.zshrc`.

3. Stop the daemon and remove its socket, log and pause flag:

   ```bash
   pkill -f "wisp daemon"
   rm -r ~/.cache/wisp
   ```

4. Remove the binary:

   ```bash
   rm ~/.local/bin/wisp        # installed with install.sh
   cargo uninstall wisp        # installed with cargo
   ```

5. Optionally delete the model weights. llama.cpp downloaded them, so they live in its cache, not in a wisp directory. By default that is the Hugging Face hub cache:

   ```bash
   ls -d ~/.cache/huggingface/hub/models--ggml-org--Qwen2.5-Coder-*
   ```

   Delete the directories that command lists. If it lists nothing, check whether `LLAMA_CACHE`, `HF_HUB_CACHE` or `HF_HOME` is set in your environment; the weights are under that directory instead. On macOS you can also delete `~/Library/Logs/wisp-llama-server.log`.
