This page describes how to build and test wisp, how the repository is laid out, and where a first contribution fits well.

## Dev loop

You need a Rust toolchain with edition 2024 support and zsh. You do not need llama-server for the tests: they run against a fake one.

```bash
git clone https://github.com/lolabradora/wisp
cd wisp
cargo test          # unit tests + daemon tests against a fake llama-server
prek run            # fmt check, clippy -D warnings, tests
```

[`prek`](https://github.com/j178/prek) runs the hooks in `.pre-commit-config.yaml`. Without it, run the same three commands by hand:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --quiet
```

To try a local build in your own shell:

```bash
cargo install --path .
pkill -f "wisp daemon"      # the next prompt starts the new daemon
exec zsh                    # reload the plugin from the new binary
```

`wisp complete --show-prompt "git che"` is the fastest way to see the effect of a prompt change, and `WISP_LOG=wisp=debug` logs every request and suggestion to `~/.cache/wisp/wisp.log`.

## Tests

- **Unit tests** sit next to the code in `src/`. Prompt construction, token healing, history parsing and the status report are all covered this way.
- **`tests/daemon.rs`** starts a real daemon on a temporary socket with a fake llama-server behind it and drives it over the wire protocol. Debounce, cancellation and cached-extension reuse are tested here.
- **The zsh end-to-end test** (`tests/zsh_e2e.rs` runs `tests/zsh_e2e.sh`) starts a real zsh inside an isolated tmux server with the plugin, the real daemon and a fake llama-server. It types a partial command, expects ghost text, accepts it and checks that the full command ran. It needs tmux, zsh and python3 and skips itself when one is missing; set `WISP_E2E_REQUIRE=1` to make that a failure. Set `WISP_E2E_AUTOSUGGESTIONS` to the path of `zsh-autosuggestions.zsh` to test that mode too.

A behaviour change should come with a test at the lowest level that can express it.

## Repo layout

| Path | What it is |
| --- | --- |
| `src/main.rs` | CLI definition and subcommand dispatch |
| `src/daemon.rs` | Socket server, wire protocol, per-shell session, debounce, cancellation, suggestion cache |
| `src/context.rs` | Context gathering (history, files, branches, screen) and the transcript the model sees |
| `src/screen.rs` | Screen sources: which one applies to a shell and the command that captures it |
| `src/service.rs` | The llama-server and menu bar service definitions (launchd, systemd) used by `wisp setup` |
| `src/setup.rs` | The steps of `wisp setup` and `wisp uninstall`: model download, service install, health wait |
| `src/doctor.rs` | The checks behind `wisp doctor` |
| `src/model.rs` | llama-server client, token healing, the GBNF grammar, output clean-up |
| `src/history.rs` | zsh history file parser (extended format, multi-line entries, metafied bytes) |
| `src/stats.rs` | In-memory record of recent suggestions and latencies |
| `src/status.rs` | The report behind `wisp status` and the menu bar |
| `src/menubar.rs` | macOS menu bar item |
| `src/paths.rs` | Socket, log and pause-flag locations |
| `shell/wisp.zsh` | The zsh plugin, embedded in the binary and printed by `wisp init zsh` |
| `tests/` | Daemon integration tests and the zsh end-to-end test |
| `install.sh` | Binary installer for releases |
| `site/` | The landing page, static HTML and CSS |

## Lint policy

`Cargo.toml` sets the lints for the whole crate, and the clippy hook runs with `-D warnings`, so every warning fails the check.

- `unsafe_code` is forbidden.
- `clippy::pedantic` is on.
- Denied outright: `unwrap_used`, `panic`, `panic_in_result_fn`, `unimplemented`, `todo`, `dbg_macro`, `print_stdout`, `print_stderr`, `exit`, `mem_forget`, `await_holding_lock`, `large_futures`, `allow_attributes`.
- `expect_used` warns, which fails under `-D warnings`.

What this means in practice:

- Return `anyhow::Result` and add context with `.context(...)`. Do not unwrap.
- Write to stdout through `std::io::stdout()` and `writeln!`, and log with `tracing`. `println!` does not compile cleanly.
- To silence a lint, use `#[expect(lint, reason = "...")]`. `#[allow]` is denied.
- Tests may unwrap, expect and panic; `clippy.toml` permits that.

Formatting uses `rustfmt.toml` (`use_small_heuristics = "Max"`).

## Good first contributions

### A new screen source

This is the most useful small change: each source gives one more terminal full screen context without tmux. A source is a way to detect the terminal from the shell's environment and a command that prints the visible pane.

The pieces to touch:

1. `shell/wisp.zsh`: send the terminal's identifying environment variables in the `term` object of the `done` message.
2. `src/screen.rs`: add the field to `Terminal`, a variant to `Source`, and the detection rule and capture command in `Terminal::capture_command`, in the right place in the source order.
3. `wisp doctor`: check that it reports the new source.
4. Tests for detection and ordering, and a section in [[Terminal support]].

The capture command must finish within the 500 ms probe timeout and print plain text. If you can test on a real install of the terminal, say so in the pull request; kitty support is marked experimental because nobody has done that for it yet.

### More shells

wisp supports zsh only. The daemon and protocol are shell-agnostic: a shell integration needs to send `done` and `req` messages and draw the reply. The hard part is on the shell side; the [[FAQ]] explains what blocks fish and bash today. A working ble.sh integration for bash, or a fish approach that does not fight its built-in autosuggestion, would be a real addition. Open an issue to discuss the design first.

### A Homebrew formula

There is no Homebrew formula yet. Release assets are `wisp-<target>.tar.gz` with `.sha256` files for four targets, which is what a formula needs.

### Other useful work

- Verifying kitty support on a real install and reporting the result.
- Measurements from hardware other than an Apple M5, taken with `wisp complete` and `wisp status`.
- Reports of plugin conflicts with a minimal `~/.zshrc` that reproduces them.

## Pull requests

- Keep each pull request to one change.
- Run `prek run` before pushing.
- Do not add numbers to docs or the landing page that you did not measure. State the hardware next to every figure.
- wisp is MIT licensed, and contributions are accepted under the same license.
