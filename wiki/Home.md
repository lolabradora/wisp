This page says what wisp is, how to install it in four lines, and where to find everything else.

wisp shows grey ghost-text command suggestions in zsh. A small code model running on your own machine writes them, and it reads what you can see: the terminal output above the prompt, your shell history, the files in the current directory and your git branches. After a command finishes, wisp predicts the next one; while you type, it completes the line.

```
% ls
Makefile  docker-compose.dev.yml  docker-compose.prod.yml  src/
% docker compose -f docker-compose.p|rod.yml
                                    └─ grey suggestion; → accepts it
```

The file name was never in your history. wisp completed it because `ls` had just printed it.

## Quick start

```bash
brew install llama.cpp                      # or a llama.cpp release on Linux
curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
wisp setup                                  # model download + background service
echo 'eval "$(wisp init zsh)"' >> ~/.zshrc
```

Open a new shell and run `wisp doctor` to check the result. [[Installation]] explains each step.

## Which terminal do I need?

wisp runs in any terminal emulator on macOS or Linux, as long as the shell is zsh. What changes is whether it can read the screen.

| Where your shell runs | Setup | Context the model gets |
| --- | --- | --- |
| Inside tmux, in any terminal (Ghostty, iTerm2, Alacritty, Terminal.app, ...) | None | Screen, history, files, git branches, session commands |
| WezTerm | None | Screen, history, files, git branches, session commands |
| kitty (experimental) | Two lines in `kitty.conf` | Screen, history, files, git branches, session commands |
| Any other terminal, with `WISP_SCREEN_CMD` set | A command that prints the visible screen | Screen, history, files, git branches, session commands |
| Any other terminal, nothing configured | None | History, files, git branches, and the commands this session ran with their exit codes |

kitty support has not been verified on a real kitty install yet. Details for every row are in [[Terminal support]].

## Pages

- [[Installation]]: prerequisites, `wisp setup`, model choice, upgrading, uninstalling
- [[Terminal support]]: screen sources and how to check which one is active
- [[Configuration]]: environment variables, pausing, using another llama-server or model
- [[How it works]]: architecture, the prompt, token healing, caching and cancellation
- [[Privacy and security]]: what the model sees and what stays on disk
- [[Troubleshooting]]: symptoms, causes and fixes
- [[FAQ]]: short answers to common questions
- [[Contributing]]: dev loop, repo layout, where to start
