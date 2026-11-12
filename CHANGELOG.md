# Changelog

## 0.2.0

wisp now works outside WezTerm on macOS.

- **Linux.** The daemon, the plugin and `wisp setup` run on Linux (x86_64 and aarch64).
  llama-server runs as a systemd user unit there.
- **Screen context in any terminal.** wisp reads the screen through tmux, WezTerm, kitty
  (experimental) or a command you set in `WISP_SCREEN_CMD`.
- **zsh-autosuggestions is optional.** Without it, wisp draws the grey text and binds the accept
  keys itself.
- **`wisp setup`** downloads the model and installs the llama-server service. It replaces the
  scripts in `scripts/`. `wisp uninstall` removes the services.
- **`wisp doctor`** checks every part of the install and says how to fix what is missing.
- **Prebuilt binaries** for macOS and Linux, and an `install.sh` that fetches the right one.
- `WISP_DEBOUNCE_MS` sets how long typing must pause before the model is asked.

## 0.1.0

First version: zsh plugin, daemon, token healing, menu bar item. macOS and WezTerm only.
