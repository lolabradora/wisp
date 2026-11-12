This page gives short answers to the questions people ask before and after installing wisp.

## Does anything leave my machine?

No terminal data does. The model runs in llama-server on `127.0.0.1`, and wisp talks to it over loopback HTTP. The only downloads are the wisp binary and, during `wisp setup`, the model weights that llama.cpp fetches from Hugging Face. [[Privacy and security]] lists exactly what the model sees.

## Why zsh only? What about fish and bash?

fish and bash are not supported today.

- **fish** draws its own autosuggestions and has no hook for replacing them with text from another source.
- **bash** has no ghost-text facility of its own. It would need ble.sh, and wisp has no integration for that.

More shells are a welcome contribution; see [[Contributing]].

## How is this different from zsh-autosuggestions?

zsh-autosuggestions suggests commands from your history, so it cannot complete a name it has never seen; wisp can complete one that only appeared in command output a moment ago. The two work together: if zsh-autosuggestions is loaded, wisp uses it to draw its suggestions.

## How is this different from Atuin?

Atuin searches and syncs shell history, and like any history-only tool it cannot complete what has never been typed. wisp does not replace it, and you can run both.

## How is this different from hosted AI terminal tools?

Hosted tools send terminal context to a cloud model, and some require their own terminal app. wisp runs in your existing terminal with a small model on your machine, which also means its suggestions come from a much smaller model.

## How much RAM and disk does it need?

Disk: 1.65 GB for Qwen2.5-Coder 1.5B or 3.3 GB for 3B.

RAM: on an Apple M5 with 24 GB, the llama-server process for 3B used about 4.3 GB. We have not recorded a figure for 1.5B, whose weights are half the size. `wisp status` shows the memory of your own llama-server.

## Which Macs and Linux machines does it run on?

macOS on Apple Silicon and Intel, and Linux on x86_64 and aarch64. Prebuilt binaries exist for those four targets; the Linux ones are built against glibc.

All our latency measurements come from one Apple M5 with 24 GB. wisp will run on slower hardware, but we have no numbers for it. Measure on your machine before deciding which model to use.

## Do I need a particular terminal?

No. wisp works in any terminal. To give the model the screen, you need tmux, WezTerm, kitty (experimental) or a custom command. See the table on [[Home]] and the details in [[Terminal support]].

## Do I need zsh-autosuggestions?

No. Without it, wisp draws the ghost text itself and binds the accept keys. See [[Installation]].

## Can I use another model?

Yes, if it is served by llama.cpp's `llama-server` with `/completion` and GBNF grammar support. Set `WISP_LLM_URL` to that server. Base or fill-in-the-middle code models work best, because the prompt is a raw transcript continuation with no chat template. [[Configuration]] has the steps. We have only measured Qwen2.5-Coder 1.5B and 3B.

## How do I measure latency myself?

```bash
wisp complete "git che"
```

This runs one completion from the current directory and prints the suggestion twice: once `cold` and once `cached`, each with its time, plus how long gathering context took.

```bash
wisp status
```

This shows the median latency of recent real suggestions and the last 10 with their individual times. The latency figures in this wiki were taken with these two commands.

## How accurate is it?

We have one small internal eval: 8 scenarios where the right answer is visible only in terminal output (a compose file, a git branch, a failing test, a pod name, a stack-trace path, a PID, the next command after `git add`, a container name). With token healing, 1.5B got 6 of 8 and 3B got 7 of 8. That is too small to generalise from. A wrong suggestion costs nothing unless you accept it, so read the grey text before pressing →.

## Does wisp run commands for me?

No. It only shows text. Nothing runs until you accept a suggestion and press Enter yourself.

## Can I turn it off for a while?

`wisp pause` stops suggestions in every shell until `wisp resume`.

## Why did I get no suggestion?

The model may have had nothing to add, the cursor may not be at the end of the line, or something is down. `wisp status` shows which. See [[Troubleshooting]].
