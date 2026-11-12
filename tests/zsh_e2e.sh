#!/usr/bin/env bash
# End-to-end test of the zsh plugin: a real zsh inside an isolated tmux server, the real wisp
# daemon, and a fake llama-server. Types a partial command, expects ghost text, accepts it and
# checks that the full command ran.
#
#   WISP_BIN=<path>                 wisp binary to test (default: the debug build from cargo)
#   WISP_E2E_AUTOSUGGESTIONS=<path> zsh-autosuggestions.zsh to test that mode with
#   WISP_E2E_REQUIRE=1              fail instead of skipping when tmux, zsh or python3 is missing
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CONTINUATION='n-by-wisp second'
PROMPT='e2e> '

for tool in tmux zsh python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    if [ -n "${WISP_E2E_REQUIRE:-}" ]; then
      echo "zsh_e2e: FAIL: $tool is required" >&2
      exit 1
    fi
    echo "zsh_e2e: SKIP: $tool not found"
    exit 0
  fi
done

if [ -z "${WISP_BIN:-}" ]; then
  (cd "$ROOT" && cargo build --quiet)
  # Asked of cargo: the target directory may be configured outside the repository.
  WISP_BIN=$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps |
    python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')/debug/wisp
fi

TMP=$(mktemp -d)
TMUX_NAME=wisp-e2e-$$
FAKE_PID=
MODE=
FAILED=0

tm() {
  tmux -L "$TMUX_NAME" -f /dev/null "$@"
}

# shellcheck disable=SC2329  # runs from the EXIT trap
cleanup() {
  tm kill-server 2>/dev/null || true
  # tmux leaves the socket file of a server that has exited.
  rm -f "${TMUX_TMPDIR:-/tmp}/tmux-$(id -u)/$TMUX_NAME"
  # The plugin starts the daemon detached, so it is found by its socket argument.
  pkill -f "daemon --socket $TMP/" 2>/dev/null || true
  [ -n "$FAKE_PID" ] && kill "$FAKE_PID" 2>/dev/null || true
  rm -rf "$TMP"
}
trap cleanup EXIT

# --- fake llama-server -----------------------------------------------------------------------
# Answers /completion with the grammar's forced prefix plus a fixed continuation, and logs
# every request body. An empty prompt line gets no suggestion, so the screen stays predictable.
cat >"$TMP/fake_llama.py" <<'PY'
import json
import re
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

port_file, log_file, continuation = sys.argv[1:4]


class Handler(BaseHTTPRequestHandler):
    def reply(self, payload):
        body = json.dumps(payload).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self.reply({"model_alias": "fake"} if self.path == "/props" else {"status": "ok"})

    def do_POST(self):
        raw = self.rfile.read(int(self.headers.get("content-length", 0))).decode()
        with open(log_file, "a") as log:
            log.write(raw + "\n")
        grammar = json.loads(raw)["grammar"]
        literal = re.match(r'root ::= "(.*)" \[\^\\n\]\* "\\n"$', grammar).group(1)
        forced = literal.replace('\\"', '"').replace("\\\\", "\\")
        self.reply({"content": forced + (continuation if forced.strip() else "") + "\n"})

    def log_message(self, *args):
        pass


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(port_file, "w") as f:
    f.write(str(server.server_address[1]))
server.serve_forever()
PY

REQUESTS=$TMP/requests.log
: >"$REQUESTS"
python3 "$TMP/fake_llama.py" "$TMP/port" "$REQUESTS" "$CONTINUATION" &
FAKE_PID=$!
for _ in $(seq 50); do
  [ -s "$TMP/port" ] && break
  sleep 0.1
done
[ -s "$TMP/port" ] || { echo "zsh_e2e: FAIL: fake llama-server did not start" >&2; exit 1; }
LLM_URL=http://127.0.0.1:$(cat "$TMP/port")

# --- helpers ---------------------------------------------------------------------------------
screen() {
  tm capture-pane -p -t e2e
}

fail() {
  FAILED=1
  echo "zsh_e2e: FAIL [$MODE]: $1" >&2
  {
    echo "--- screen"
    screen || true
    echo "--- daemon log"
    cat "$TMP/$MODE/wisp.log" 2>/dev/null || true
    echo "---"
  } >&2
}

# Polls for up to 10 s until the command succeeds.
eventually() {
  for _ in $(seq 100); do
    "$@" >/dev/null 2>&1 && return 0
    sleep 0.1
  done
  return 1
}

# shellcheck disable=SC2329  # called through `eventually`
on_screen() {
  screen | grep -qF -- "$1"
}

# A whole screen line, so leftover ghost text after the command would not match.
screen_line() {
  screen | grep -qFx -- "$1"
}

# The ghost text is styled, so an escape sequence sits between the typed text and the rest.
ghost_is_styled() {
  tm capture-pane -e -p -t e2e | grep -q "$1"$'\033\\[[0-9;]*m'"$2"
}

type_keys() {
  tm send-keys -t e2e -l "$1"
}

press() {
  tm send-keys -t e2e "$@"
}

# --- one run per mode ------------------------------------------------------------------------
# $1 = mode name, $2 = zsh-autosuggestions.zsh to load first (empty for wisp's own ghost text)
run_mode() {
  MODE=$1
  local autosuggestions=$2 dir=$TMP/$1 marker="marker-$$-$1"
  local work=$dir/work
  mkdir -p "$dir/home" "$work"
  {
    echo "PS1='$PROMPT'"
    # zsh picks vi keys when EDITOR mentions vi; the test presses emacs keys.
    echo "bindkey -e"
    [ -n "$autosuggestions" ] && echo "source '$autosuggestions'"
    echo "eval \"\$('$WISP_BIN' init zsh --socket '$dir/wisp.sock')\""
  } >"$dir/rc.zsh"

  # The marker is printed, never typed, so it can only reach the model through a screen capture.
  env -u TMUX -u TMUX_PANE -u WEZTERM_PANE -u KITTY_WINDOW_ID -u KITTY_LISTEN_ON \
    -u WISP_SCREEN_CMD -u HISTFILE HOME="$dir/home" ZDOTDIR="$dir/home" WISP_LLM_URL="$LLM_URL" \
    tmux -L "$TMUX_NAME" -f /dev/null new-session -d -s e2e -x 200 -y 50 -c "$work" \
    "printf 'screen-%s\n' '$marker'; exec zsh -f"
  eventually on_screen "screen-$marker" || { fail "zsh did not start"; return; }

  type_keys "source $dir/rc.zsh"
  press Enter
  # capture-pane drops trailing spaces, so an empty prompt shows without its last one.
  eventually screen_line "${PROMPT% }" || { fail "the plugin did not load"; return; }
  # The first prompt asks for a prediction; by then the daemon has gathered its context.
  eventually test -s "$REQUESTS" || { fail "the daemon never asked the model"; return; }

  if [ -z "$autosuggestions" ]; then
    # Right arrow away from the end of the line moves the cursor and accepts nothing.
    type_keys "touch ab"
    eventually on_screen "touch ab$CONTINUATION" || { fail "no ghost text for 'touch ab'"; return; }
    ghost_is_styled "touch ab" "$CONTINUATION" || fail "ghost text is not highlighted"
    press Left
    press Right
    press Enter
    eventually test -e "$work/ab" || fail "'touch ab' did not run"
    screen_line "${PROMPT}touch ab" || fail "ghost text was left in scrollback"
    [ ! -e "$work/abn-by-wisp" ] || fail "ghost text was executed"

    # Alt-F accepts one word and keeps the rest as ghost text.
    type_keys "touch wo"
    eventually on_screen "touch wo$CONTINUATION" || { fail "no ghost text for 'touch wo'"; return; }
    press M-f
    # tmux reports the colour change after the space: a blank cell has no foreground.
    eventually ghost_is_styled "touch won-by-wisp " "second" || fail "Alt-F did not accept one word"
    press Enter
    eventually test -e "$work/won-by-wisp" || fail "the word accepted with Alt-F did not run"
    [ ! -e "$work/second" ] || fail "Alt-F accepted more than one word"
  fi

  type_keys "touch ra"
  eventually on_screen "touch ra$CONTINUATION" || { fail "no ghost text for 'touch ra'"; return; }
  [ ! -e "$work/ran-by-wisp" ] || fail "the suggestion ran before it was accepted"
  press Right
  # The fake suggests again for the accepted line: proof that the buffer now holds it, and
  # ghost text that must not survive Enter.
  eventually on_screen "touch ran-by-wisp second$CONTINUATION" || fail "Right did not accept the suggestion"
  press Enter
  eventually test -e "$work/ran-by-wisp" || fail "the accepted command did not run"
  eventually test -e "$work/second" || fail "the accepted command did not run in full"
  screen_line "${PROMPT}touch ran-by-wisp second" || fail "ghost text was left in scrollback"
  [ ! -e "$work/secondn-by-wisp" ] || fail "ghost text was executed"

  grep -qF "screen-$marker" "$REQUESTS" ||
    fail "the model prompt does not contain the text on the tmux screen"

  tm kill-session -t e2e
  [ "$FAILED" -eq 0 ] && echo "zsh_e2e: ok [$MODE]"
  return 0
}

find_autosuggestions() {
  local candidate
  for candidate in \
    "${WISP_E2E_AUTOSUGGESTIONS:-}" \
    /opt/homebrew/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
    /usr/local/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
    /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
    "$HOME/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh" \
    "$HOME/.oh-my-zsh/custom/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh"; do
    if [ -n "$candidate" ] && [ -f "$candidate" ]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

run_mode native ""
if autosuggestions=$(find_autosuggestions); then
  run_mode autosuggestions "$autosuggestions"
else
  echo "zsh_e2e: SKIP [autosuggestions]: zsh-autosuggestions not found"
fi

exit "$FAILED"
