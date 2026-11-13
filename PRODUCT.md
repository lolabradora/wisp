# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Static HTML/CSS/JS in `site/`, no build step and no dependencies. It is served by GitHub Pages at `https://wisp.heylola.ro/` (custom domain; DNS in Cloudflare as a DNS-only CNAME to `lolabradora.github.io`). Asset and link URLs in `site/` stay relative, so the page also works from any sub-path. The page makes no external requests and runs under a strict CSP (`site/_headers`): no inline scripts or styles.

## Users

Developers on macOS or Linux who spend their day in zsh. They type long, specific commands: branch names, pod names, file paths, PIDs that just scrolled past in the terminal. Their job is to retype those without copying and pasting, and without leaving the keyboard. Many already know zsh-autosuggestions. They arrive from Hacker News or GitHub and are skeptical of AI tools that send their terminal to the cloud.

## Product Purpose

wisp shows grey "ghost text" command suggestions in zsh, written by a small local code model. The model sees what the user sees: the terminal output above the prompt, shell history, the files in the current directory and git branches. After a command finishes, wisp predicts the next one; while the user types, it completes the line. Success means the user presses → instead of retyping something that is already on screen.

## Positioning

wisp reads the terminal output above the prompt and completes names that just appeared there: `docker-compose.prod.yml` from an `ls`, a crash-looping pod from `kubectl get pods`, a PID from `lsof`. It does this inline as ghost text, in the user's existing terminal, with a model running on their own machine. Hosted tools send the terminal to the cloud (Kiro CLI, Warp) or require their own terminal app (Warp, Wave). History-only tools (zsh-autosuggestions, Atuin) cannot complete what they have never seen.

## Operating Context

- **Platforms.** macOS (Apple Silicon and Intel) and Linux (x86_64 and aarch64). zsh only.
- **Model.** llama.cpp's `llama-server` runs Qwen2.5-Coder 1.5B (default) or 3B on `127.0.0.1:8012`, kept running by a launchd agent on macOS or a systemd user unit on Linux. The model always stays on the machine.
- **Shell.** zsh. zsh-autosuggestions is optional: if it is loaded before wisp, wisp uses it to draw the grey text and the user keeps its keys. If not, wisp draws the ghost text itself; →, End and Ctrl-F (at the end of the line) accept, Alt-F accepts one word.
- **Screen context.** Pluggable sources, first match wins:
  1. `WISP_SCREEN_CMD`, a custom command whose stdout is the screen text.
  2. tmux (`tmux capture-pane`), which works in any terminal emulator: Ghostty, iTerm2, Alacritty, Terminal.app and others.
  3. WezTerm (`wezterm cli get-text`).
  4. kitty (`kitty @ get-text`), which needs `allow_remote_control` and `listen_on` in `kitty.conf`. **Experimental**: not verified on a real kitty install. Say so wherever kitty is mentioned.
  Without a source wisp still uses history, the files in the current directory, git branches and the session's finished commands with their exit codes.
- **CLI.** `wisp setup`, `wisp doctor`, `wisp status`, `wisp pause` / `wisp resume`, `wisp uninstall`, `wisp complete`. A menu bar item (macOS only, `wisp setup --menubar`) shows the model, latency and recent suggestions.
- **Install.**
  ```
  brew install llama.cpp          # or a llama.cpp release binary on Linux
  curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
  wisp setup                      # model download + background service
  echo 'eval "$(wisp init zsh)"' >> ~/.zshrc
  ```
  `install.sh` puts a prebuilt binary in `~/.local/bin`. Alternative: `cargo install --git https://github.com/lolabradora/wisp`. `wisp doctor` is the "is it working?" check.
- **Links.** Repo `https://github.com/lolabradora/wisp` (public, MIT, org HEYLOLA at `https://github.com/lolabradora`). Docs are the GitHub wiki at `https://github.com/lolabradora/wisp/wiki`: Installation, Terminal-support, How-it-works, Troubleshooting, FAQ.

## Capabilities and Constraints

- Predicts the next command on an empty prompt after each command finishes.
- Completes the current line while typing, after a 60 ms pause.
- Token healing: the partial word being typed is forced through a GBNF grammar instead of ending the prompt mid-token.
- Reuses the prompt cache between keystrokes. A suggestion the user keeps typing along with is reused without another model call.
- zsh only. bash and fish are not supported.
- Screen reading needs tmux, WezTerm, kitty (experimental) or a custom command. A bare terminal window outside those gets history, files, git and session commands only.
- The menu bar item is macOS only.
- The model stays loaded in RAM: budget roughly 2 to 4.5 GB depending on the model (see Evidence for what was measured).
- Needs llama.cpp's `llama-server` on the `PATH`; wisp does not bundle it.

## Brand Commitments

- The name is **wisp**, always lowercase.
- The visual identity follows the HEYLOLA design system, recorded in DESIGN.md: warm paper, graphite text, Hanken Grotesk, and one amber accent for what is lit (caret, focus, the highlighted source name). Light only. There is no drawn logo; the wordmark is the lowercase name.
- Made by HEYLOLA. MIT licensed.

## Evidence on Hand

All figures are internal measurements on an Apple M5 with 24 GB. They must be labeled as such.

- **Latency while typing, with a cached prompt.**
  - Qwen2.5-Coder 1.5B: about 50–120 ms.
  - Qwen2.5-Coder 3B: about 150–450 ms.
  - Cold first request after a command: about 180–280 ms.
- **Accuracy.** An 8-scenario eval where the right answer is visible only in terminal output. Scenarios: compose file, git branch, failing test, pod name, stack-trace path, PID, next command after `git add`, container name.
  - Without token healing: 1.5B scored 4/8, 3B scored 3/8.
  - With token healing: 1.5B scored 6/8, 3B scored 7/8.
- **Real completions observed.**
  - `docker compose -f docker-compose.p` → `docker-compose.prod.yml`
  - `git checkout feat/p` → `feat/payments-refactor`
  - `kubectl logs -n billing api` → `api-7d9f8b6c4-x2kqp`
  - `nvim src/b` → `src/billing/invoice_service.py`
  - `kill ` → `48213`
  - `docker exec -it pgl-d` → `pgl-db-1`
  - `cargo cl` → `cargo clippy --all-targets --all-features -- -D warnings` (3B, from history)
- **Memory.** The 3B llama-server uses about 4.3 GB. The 1.5B server has not been measured.
- **Download size.** 1.65 GB for the 1.5B model, 3.3 GB for 3B. These are the listed sizes of the GGUF files llama.cpp fetches, not our measurements.
- **Coverage.** Every figure above comes from one machine. Nothing has been measured on Linux or on Intel Macs.
- **License.** MIT (open source).
- **Absent: do not fabricate.** No users, testimonials, stars, download counts, press, or company logos. No benchmark against other tools. No screenshots or recordings yet.

## Product Principles

1. **What's on screen is the context.** A suggestion should use names the user can see, not generic guesses.
2. **Local means local.** Nothing typed or displayed leaves the machine. Say so plainly and never qualify it with a cloud option that does not exist.
3. **Never in the way.** Ghost text only, the user's own terminal, the user's own keys. A slow or wrong suggestion costs one keystroke to ignore.
4. **Honest numbers.** Every figure comes from a measurement the user can reproduce with `wisp complete` and `wisp status`.
