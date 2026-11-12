This page explains how wisp reads the terminal screen, what each terminal needs, and how to check which source your shell is using.

## How a source is chosen

After each command finishes, the zsh plugin tells the daemon which terminal environment the shell runs in. The daemon tries these sources in order and uses the first one that applies:

1. `WISP_SCREEN_CMD`, a custom command
2. tmux
3. WezTerm
4. kitty (experimental)

The choice is per shell. A shell inside tmux inside WezTerm uses tmux. Inside tmux the emulator's variables are still set, but the emulator's screen may be showing a different tmux window from the one the shell is in, so tmux is asked first.

From the captured text, wisp drops trailing blank lines, keeps the last 60 lines and cuts each line at 200 characters.

## tmux

Nothing to configure. If `$TMUX` and `$TMUX_PANE` are set, wisp runs:

```bash
tmux -S <socket> capture-pane -p -J -t <pane> -S -60
```

Because tmux does the capturing, this works in any terminal emulator: Ghostty, iTerm2, Alacritty, Terminal.app and others. If your terminal has no source of its own, running your shell inside tmux is the simplest way to give wisp the screen.

## WezTerm

Nothing to configure. If `$WEZTERM_PANE` is set, wisp runs:

```bash
wezterm cli get-text --pane-id <pane> --start-line -60
```

The `wezterm` binary must be on the `PATH` of the shell that started the daemon.

## kitty (experimental)

kitty support follows kitty's remote-control documentation but has not been verified on a real kitty install. Please report what you find.

Add these two lines to `kitty.conf`, then quit and reopen kitty:

```
allow_remote_control yes
listen_on unix:/tmp/kitty-{kitty_pid}
```

kitty then sets `$KITTY_WINDOW_ID` and `$KITTY_LISTEN_ON` in every shell, and wisp runs:

```bash
kitty @ --to $KITTY_LISTEN_ON get-text --match id:$KITTY_WINDOW_ID --extent screen
```

Without both settings `$KITTY_LISTEN_ON` is empty and wisp treats the shell as having no screen source. Note that `allow_remote_control yes` lets any program running in kitty control it; read kitty's documentation on remote control before turning it on.

## Custom command: `WISP_SCREEN_CMD`

For any other terminal, set `WISP_SCREEN_CMD` to a command that prints the screen. The contract:

- wisp runs it with `sh -c`.
- It must print the visible screen as plain text on stdout.
- It must finish within 500 ms. A slower command is abandoned and that prompt gets no screen context.

It takes priority over every other source, so you can also use it to override tmux or WezTerm. Export it in `~/.zshrc`, before the `eval "$(wisp init zsh)"` line:

```zsh
export WISP_SCREEN_CMD='my-terminal-cli dump-screen'
```

Replace `my-terminal-cli dump-screen` with whatever your terminal offers for dumping the current pane. To test a candidate, run it by hand and check that it prints what you see:

```bash
sh -c "$WISP_SCREEN_CMD" | tail -n 20
```

## Checking which source is active

Run both commands from the shell you want to check; they read that shell's environment.

```bash
wisp doctor
```

The screen-source line names the source detected for this shell and says whether a capture returned text.

```bash
wisp complete --show-prompt "git che"
```

This prints the prompt the model receives. If a source works, the prompt contains a `# Terminal screen, newest output at the bottom:` section with the text above your prompt. If that section is missing, wisp has no screen for this shell.

## With no screen source

wisp still suggests commands. The model gets:

- the last 30 unique commands from your history, including what this session ran;
- up to 40 file names from the current directory;
- the current git branch and up to 10 recently committed branches;
- a note when the last command failed, with its exit code.

What it cannot do is complete a name that only appeared in command output, such as a pod name from `kubectl get pods` or a PID from `lsof`. Those completions need one of the sources above.
