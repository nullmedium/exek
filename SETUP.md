# exek Setup Guide

## Installation

### 1. Build and Install the Binary

```bash
# Build in release mode
cargo build --release

# Install to system
sudo cp target/release/exek /usr/local/bin/

# Optional: Install launcher scripts
sudo cp scripts/exek-launcher.sh /usr/local/bin/exek-launcher
sudo cp scripts/exek-dmenu-compat.sh /usr/local/bin/exek-dmenu
```

## i3 Window Manager Configuration

Add one of these configurations to your `~/.config/i3/config`:

### Option 1: Floating Window Launcher (Recommended)

```bash
# Launch with Super+d (replaces dmenu)
# Using full path ensures it works regardless of PATH
bindsym $mod+d exec --no-startup-id alacritty --class exek-launcher -e /usr/local/bin/exek

# Alternative: Use the launcher script which handles terminal detection
# bindsym $mod+d exec --no-startup-id /usr/local/bin/exek-launcher

# Window rules for exek
for_window [class="exek-launcher"] floating enable
for_window [class="exek-launcher"] resize set 800 400
for_window [class="exek-launcher"] move position center
for_window [class="exek-launcher"] border pixel 2
```

### Option 2: Scratchpad Launcher

```bash
# Create scratchpad launcher with Super+Shift+d
bindsym $mod+Shift+d exec --no-startup-id alacritty --class exek-scratchpad -e exek
for_window [class="exek-scratchpad"] move scratchpad, resize set 800 400

# Toggle scratchpad with Super+minus
bindsym $mod+minus [class="exek-scratchpad"] scratchpad show, move position center
```

### Option 3: Use the Wrapper Script

```bash
# This auto-detects your terminal emulator
bindsym $mod+d exec --no-startup-id /usr/local/bin/exek-launcher
```

## tmux Configuration

Add to your `~/.tmux.conf`:

### Popup Launcher (tmux 3.2+)

```bash
# Launch with prefix + p in a popup
bind-key p display-popup -E -w 80% -h 60% 'exek'

# Quick launcher with Alt+Space (no prefix)
bind-key -n M-Space display-popup -E -w 80% -h 60% 'exek'
```

### Traditional Window/Pane

```bash
# New window with prefix + o
bind-key o new-window -n launcher 'exek'

# Split pane with prefix + l
bind-key l split-window -h -l 40% 'exek'
```

## Terminal Shortcuts

### Alacritty

Add to `~/.config/alacritty/alacritty.yml`:

```yaml
key_bindings:
  - { key: D, mods: Super, action: SpawnNewInstance, args: ["-e", "exek"] }
```

### Kitty

Add to `~/.config/kitty/kitty.conf`:

```conf
map super+d launch --type=overlay exek
```

## Desktop Entry (Optional)

Create `/usr/share/applications/exek.desktop`:

```ini
[Desktop Entry]
Name=Exek Launcher
Comment=Fast application launcher with fuzzy search
Exec=exek-launcher
Terminal=false
Type=Application
Icon=application-x-executable
Categories=System;Utility;
```

## Usage Tips

1. **i3 users**: Use `$mod+d` to replace dmenu/rofi with exek
2. **tmux users**: Use `prefix + p` for popup or `Alt+Space` for quick access
3. **First run**: The launcher will be empty until you search, then it shows recent apps
4. **Frecency**: Apps you use often will appear first in searches

## Troubleshooting

### No applications found

Check that XDG desktop files exist in:
- `/usr/share/applications/`
- `~/.local/share/applications/`

### Terminal apps don't launch properly

Ensure you have one of these terminal emulators installed:
- alacritty, kitty, wezterm, foot, gnome-terminal, konsole, xterm

### Launcher doesn't close after launching

This is by design in some configurations. Press `Esc` to close.

## Uninstall

```bash
sudo rm /usr/local/bin/exek
sudo rm /usr/local/bin/exek-launcher
sudo rm /usr/local/bin/exek-dmenu
rm -rf ~/.config/exek
```
