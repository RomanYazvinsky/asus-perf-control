#!/bin/bash
# Check if D-Bus daemon is installed

DAEMON_PATH="/usr/lib/asus-linux-control/asus-control-daemon"

if [ -f "$DAEMON_PATH" ]; then
    echo "✅ D-Bus daemon is installed at $DAEMON_PATH"
    exit 0
else
    echo "⚠️  WARNING: D-Bus daemon NOT installed"
    echo ""
    echo "To install, run:"
    echo "  cd install && sudo ./install.sh"
    echo ""
    echo "Or use Makefile:"
    echo "  make install"
    exit 1
fi
