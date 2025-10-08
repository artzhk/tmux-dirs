# tmux-dirs

Per-session directory stacks for tmux, implemented as a user-space daemon plus a CLI. The CLI communicates with the server over a UNIX domain socket. No system-level services required.

## Installation

1. Put to your .tmux.conf after `set -g @plugin 'tmux-plugins/tpm'` before `run '~/.tmux/plugins/tpm/tpm'` 

```tmux 
set -g @plugin 'artzhk/tmux-dirs'
```

3. Provide aliases to `.bashrc` or `.profile`
```bash 
alias tpushd="~/.tmux/plugins/tmux-dirs/bin/tmux-dirs-pushd"
alias tpopd=". ~/.tmux/plugins/tmux-dirs/bin/tmux-dirs-popd"
alias tpeekd=". ~/.tmux/plugins/tmux-dirs/bin/tmux-dirs-peekd"
alias tdirs=". ~/.tmux/plugins/tmux-dirs/bin/tmux-dirs-dirs"
```

## Overview
### CLI

Synopsis:

```
tmux-dirs push  <session-id> <path>   # push path onto session stack
tmux-dirs pop   <session-id>          # print top path and pop
tmux-dirs peek  <session-id>          # print top path without popping
tmux-dirs dirs  <session-id>          # print stack as space-separated list
tmux-dirs clear <session-id>          # clear stack
```

Usage from tmux (examples):

```tmux
# Push current pane dir
bind-key -n M-[ run-shell -b 'tmux display -p -F "#{session_id} #{pane_current_path}" \
  | xargs -r sh -c '\''tmux-dirs push "$0" "$1"'\'' '

# Pop and cd in the active pane
bind-key -n M-] run-shell -b 'p="$(tmux-dirs pop "$(tmux display -p "#{session_id}")")"; \
  [ -n "$p" ] && tmux send-keys -l "cd -- $p" Enter'
```
* `server` starts a per-user background server that keeps stacks in memory.
* `tmux-dirs <cmd>` is a one-shot client that connects to the server, sends a single request, prints the server’s reply to stdout, and exits.
* The server is started automatically by tmux hooks on `client-attached` and `session-created` if it is not already running.

### Socket
* Path: `/tmp/dirs.sock`.

## IPC protocol

* Transport: UNIX domain socket (stream).
* Framing: single-request, single-reply. The client writes the request, then shuts down its write-half; the server reads until EOF, processes, writes a single textual reply, then shuts down its write-half.
* Request format (space-separated tokens):
  ```
  <cmd> <session_id> [<path>]
  ```
  where `<cmd>` is one of: `PUSH`, `POP`, `PEEK`, `DIRS`, `CLEAR`.

* Reply format:
  * For `PUSH`: `<path>\n` or `ERR <message>\n`
  * For `POP`/`PEEK`: `<path>\n` (empty if none)
  * For `DIRS`: space-separated list of absolute paths, then `\n`

This simple line protocol matches the “original dirs-like” UX: the CLI prints whatever the server returns.

