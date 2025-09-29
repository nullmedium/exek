#!/bin/bash

# Debug wrapper for exek to diagnose launch issues
# Usage: exek-debug [args...]

DEBUG_LOG="/tmp/exek-debug-$$.log"

echo "=== EXEK DEBUG LOG ===" > "$DEBUG_LOG"
echo "Date: $(date)" >> "$DEBUG_LOG"
echo "PID: $$" >> "$DEBUG_LOG"
echo "" >> "$DEBUG_LOG"

echo "=== ENVIRONMENT ===" >> "$DEBUG_LOG"
echo "PATH: $PATH" >> "$DEBUG_LOG"
echo "DISPLAY: $DISPLAY" >> "$DEBUG_LOG"
echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY" >> "$DEBUG_LOG"
echo "XDG_RUNTIME_DIR: $XDG_RUNTIME_DIR" >> "$DEBUG_LOG"
echo "HOME: $HOME" >> "$DEBUG_LOG"
echo "USER: $USER" >> "$DEBUG_LOG"
echo "SHELL: $SHELL" >> "$DEBUG_LOG"
echo "TERM: $TERM" >> "$DEBUG_LOG"
echo "" >> "$DEBUG_LOG"

echo "=== LAUNCH CONTEXT ===" >> "$DEBUG_LOG"
echo "TTY: $(tty)" >> "$DEBUG_LOG"
echo "Parent PID: $PPID" >> "$DEBUG_LOG"
echo "Parent process: $(ps -p $PPID -o comm=)" >> "$DEBUG_LOG"
echo "" >> "$DEBUG_LOG"

echo "=== STARTING EXEK ===" >> "$DEBUG_LOG"
echo "Command: exek $*" >> "$DEBUG_LOG"
echo "" >> "$DEBUG_LOG"

# Find exek binary
EXEK_BIN="$(which exek 2>/dev/null)"
if [ -z "$EXEK_BIN" ]; then
    EXEK_BIN="/usr/local/bin/exek"
    if [ ! -x "$EXEK_BIN" ]; then
        EXEK_BIN="./target/release/exek"
    fi
fi

echo "Using binary: $EXEK_BIN" >> "$DEBUG_LOG"

# Run exek with environment logging
RUST_BACKTRACE=1 "$EXEK_BIN" "$@" 2>> "$DEBUG_LOG"
EXIT_CODE=$?

echo "" >> "$DEBUG_LOG"
echo "=== EXEK EXITED ===" >> "$DEBUG_LOG"
echo "Exit code: $EXIT_CODE" >> "$DEBUG_LOG"

if [ $EXIT_CODE -ne 0 ]; then
    echo "Debug log saved to: $DEBUG_LOG"
    echo "View with: cat $DEBUG_LOG"
else
    # Clean up log on successful exit
    rm -f "$DEBUG_LOG"
fi

exit $EXIT_CODE
