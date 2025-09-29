#!/bin/bash
# Wrapper script for launching exek with proper terminal detection
# Install to /usr/local/bin/exek-launcher

# Function to detect the best terminal emulator
detect_terminal() {
    # Check for common terminal emulators in order of preference
    for term in alacritty kitty wezterm foot gnome-terminal konsole xfce4-terminal urxvt xterm; do
        if command -v "$term" &> /dev/null; then
            echo "$term"
            return
        fi
    done
    echo "xterm"  # Fallback
}

# Check if we're already in a terminal
if [ -t 0 ] && [ -t 1 ]; then
    # We're in a terminal, just run exek directly
    exec exek "$@"
else
    # We need to launch in a terminal
    TERMINAL=$(detect_terminal)

    case "$TERMINAL" in
        alacritty)
            exec alacritty --class exek-launcher -e exek "$@"
            ;;
        kitty)
            exec kitty --class exek-launcher -e exek "$@"
            ;;
        wezterm)
            exec wezterm start --class exek-launcher -- exek "$@"
            ;;
        foot)
            exec foot --app-id=exek-launcher exek "$@"
            ;;
        gnome-terminal)
            exec gnome-terminal --class=exek-launcher -- exek "$@"
            ;;
        konsole)
            exec konsole --separate -e exek "$@"
            ;;
        xfce4-terminal)
            exec xfce4-terminal --disable-server -x exek "$@"
            ;;
        urxvt)
            exec urxvt -name exek-launcher -e exek "$@"
            ;;
        *)
            exec xterm -class exek-launcher -e exek "$@"
            ;;
    esac
fi
