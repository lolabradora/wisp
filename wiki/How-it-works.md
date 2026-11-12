This page explains what happens between a keystroke and the grey text, and why the prompt is built the way it is.

## Architecture

```
 your terminal
┌──────────────────────┐
│ zsh + wisp plugin    │
│  precmd: "done" msg  │      unix socket             loopback HTTP
│  redraw: "req" msg   │─────────────────▶ wisp ─────────────────────▶ llama-server
│  draws ghost text    │◀───────────────── daemon ◀───────────────────  127.0.0.1:8012
└──────────────────────┘  ~/.cache/wisp/    │                           Qwen2.5-Coder
                          wisp.sock         │ after each command:
                                            ├─ screen (tmux / WezTerm / kitty / custom)
                                            ├─ history file
                                            ├─ cwd listing
                                            └─ git branches
```

There are three processes:

- **The zsh plugin** (`shell/wisp.zsh`, printed by `wisp init zsh`) hooks into zsh, sends messages and draws the reply. It contains no model logic.
- **The daemon** (`wisp daemon`) is one process per user, started by the plugin on demand. Each shell holds one connection to it. It gathers context, builds prompts, debounces, cancels and caches.
- **llama-server** is llama.cpp's HTTP server, kept running by the service that `wisp setup` installed.

The plugin sends one JSON object per line. The daemon answers with `<seq>\t<suggested command line>\n`. The shell numbers its messages and ignores any reply that is not for its latest one, so a late answer can never overwrite a newer line.

## The life of a keystroke

**1. A command finishes.** zsh's `precmd` hook sends a `done` message with the command, its exit code, the working directory, the history file path and the terminal identifiers (`$TMUX`, `$WEZTERM_PANE` and so on).

**2. The daemon gathers context.** It runs the probes concurrently: screen capture, history file, directory listing, two `git` calls. Each external command has a 500 ms timeout and each source is best-effort; one that fails leaves its section out.

**3. It predicts the next command.** With the prompt still empty, the daemon asks the model right away, with no debounce. If you see grey text on an empty prompt, this is where it came from.

**4. You type.** On each redraw where the line changed, the plugin sends a `req` message with the buffer. It only does this when the cursor is at the end of a non-empty, single-line buffer.

**5. The daemon decides whether to ask the model.**

- Any new message first cancels the request in flight for that shell.
- If the previous suggestion still starts with what you have typed, the daemon sends it back at once without a model call.
- Otherwise it waits for the debounce interval (60 ms by default) and then asks the model. Another keystroke during the wait cancels it.

**6. The plugin draws the reply.** It checks that the sequence number is current, that the suggestion starts with the buffer and that the cursor is still at the end. Then it shows the remainder as ghost text, through zsh-autosuggestions if loaded or with its own `POSTDISPLAY` highlight if not.

**7. You accept or ignore it.** Accepting inserts the text. Typing something else clears it.

## The transcript the model sees

The daemon renders the context as a shell transcript and the model continues it. For a shell in a project directory it looks like this:

```
# zsh on macOS. Recent commands, oldest first:
$ git pull
$ make test
$ docker compose -f docker-compose.dev.yml up -d
$ git status
$ ls
# Terminal screen, newest output at the bottom:
% git status
On branch feat/payments-refactor
nothing to commit, working tree clean
% ls
Makefile  docker-compose.dev.yml  docker-compose.prod.yml  src/
# cwd: /Users/me/code/shop
# files: Makefile docker-compose.dev.yml docker-compose.prod.yml src/ .env .git/
# git branch: feat/payments-refactor
# recent branches: feat/payments-refactor main fix/invoice-rounding
$ docker compose -f
```

The first line names your OS (macOS or Linux). The last line is the start of the command being completed. Everything above it has fixed limits:

| Section | Limit |
| --- | --- |
| History | The last 30 unique commands. Read from the tail of the history file, then merged with the commands this session finished, newest last |
| Screen | The last 60 lines after trailing blank lines are dropped, each cut at 200 characters |
| Files | 40 names from the working directory: visible entries first, then dotfiles, directories marked with `/` |
| Branches | The current branch and the 10 most recently committed local branches |
| Failure note | One line, only when the last command exited non-zero: ``# `make test` failed with exit code 2`` |

Empty sections are left out. Print the real prompt for your shell with:

```bash
wisp complete --show-prompt "docker compose -f docker-compose.p"
```

## Token healing

A model reads its prompt as tokens, and a token is usually several characters long. Suppose you have typed:

```
docker compose -f docker-compose.p
```

If the prompt ended right there, the model would have to continue from the middle of a word. Its tokenizer splits `docker-compose.p` differently from how it would split `docker-compose.prod.yml`, so the model starts from a token sequence it rarely saw in training. In our tests it produced continuations like `.prd`.

wisp avoids this in two steps.

1. It cuts the prompt back to the last space. The prompt ends with `$ docker compose -f`, a clean token boundary.
2. It forces the model to regenerate the partial word, using a GBNF grammar:

   ```
   root ::= " docker-compose.p" [^\n]* "\n"
   ```

The grammar says: the output must begin with ` docker-compose.p`, then anything up to the end of the line. The model is free to choose its own tokens for the forced text, so it can emit `docker-compose.prod.yml` the way it normally would. wisp then strips the forced prefix from the output and appends the rest to your buffer.

The same rule covers the edge cases. After a trailing space (`kill `) or on an empty prompt, the forced text is a single space. If the model's output does not start with the forced text, wisp discards it.

On an internal 8-scenario eval where the right answer is visible only in terminal output, token healing moved Qwen2.5-Coder 1.5B from 4/8 to 6/8 and 3B from 3/8 to 7/8. The eval is internal and small, so read the result as an indication that healing helps.

## Prompt-cache reuse

The transcript depends only on the context gathered after the last command. It does not include what you are typing, so it stays byte-identical from one keystroke to the next. Only the final `$ ...` line changes.

wisp sends every request with `cache_prompt: true`. llama-server keeps the evaluated state of the previous prompt and reprocesses only the part that differs, which is the last line. That is why completions while typing are faster than the first request after a command: on an Apple M5 with 24 GB we measured about 50–120 ms cached with 1.5B, and about 180–280 ms for a cold first request.

The cost of this design is that context is refreshed only when a command finishes. If another process changes the directory while you type, wisp sees it at the next prompt.

## Cancellation

Each shell has at most one pending request. Every new message from that shell aborts the pending task before anything else happens. If the task was still waiting out the debounce, no request is sent at all. If the HTTP request was already in flight, the daemon drops it and the connection closes.

A reply that slips through anyway carries an old sequence number and the plugin ignores it.

## Cached-extension reuse

The daemon remembers the last suggestion per shell. If you type `g`, get `git checkout main`, and then type `i`, `t`, space, each new buffer is still a prefix of that suggestion. The daemon answers from memory, with no debounce and no model call, so the grey text does not flicker while you type along with it.

The cached suggestion is dropped when a command finishes, and while wisp is paused. A new answer from the model replaces it.

## Why a base code model rather than a chat model

wisp asks for one thing: the most likely continuation of a line in a shell transcript. Continuing text is the task a base code model is trained on, so the prompt needs no instructions.

A chat model would need a system prompt and a chat template, and would have to be told not to explain itself. Those extra tokens cost latency on every request, and the answer needs cleaning up before it can be shown as ghost text. Raw continuation also combines cleanly with the grammar: the output is constrained to one line that starts with what you typed.

Small base models are also fast enough to run per keystroke on a laptop. The limits follow from the same choice: the model sees about 60 lines of screen, writes at most 48 tokens and one line, and has no room to reason about a task. Expect likely completions of the current line and nothing more ambitious.
