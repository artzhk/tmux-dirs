#!/usr/bin/env bash

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

( cd "$CURRENT_DIR/src" && make build )

tmux set-environment -g TMUX_DIRS_PATH "$CURRENT_DIR"

tmux set-hook -g session-created 'run-shell -b "#{TMUX_DIRS_PATH}/bin/release/server"'
tmux set-hook -g client-attached 'run-shell -b "#{TMUX_DIRS_PATH}/bin/release/server"'


