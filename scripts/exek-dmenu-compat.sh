#!/bin/bash
# dmenu compatibility wrapper for exek
# This allows using exek as a drop-in replacement for dmenu_run
# Install to /usr/local/bin/dmenu_run to replace system dmenu

# Launch exek in a floating terminal
# The terminal will close automatically after launching an app
exec /usr/local/bin/exek-launcher "$@"
