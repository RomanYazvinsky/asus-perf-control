#!/bin/bash
# Quick daemon rebuild and restart script
# Run WITHOUT sudo: ./rebuild-daemon.sh

set -e

echo "=== Rebuilding ASUS Control Daemon ==="

# Build daemon (as user, not root)
echo "1. Building daemon..."
cd ../rust_daemon
cargo build --release
cd ../install

# Now request sudo for installation
echo ""
echo "2. Installing daemon (requires sudo)..."
sudo bash << 'SUDO_SCRIPT'
set -e

# Stop old daemon
echo "   Stopping old daemon..."
pkill -9 -f '/usr/lib/asus-linux-control/asus-control-daemon' 2>/dev/null || true
for _ in 1 2 3 4 5; do
    if ! pgrep -f '/usr/lib/asus-linux-control/asus-control-daemon' >/dev/null 2>&1; then
        break
    fi
    sleep 0.2
done

# Copy new daemon
echo "   Installing new daemon..."
install -d /usr/lib/asus-linux-control
install -m 755 ../rust_daemon/target/release/asus-control-daemon \
    /usr/lib/asus-linux-control/asus-control-daemon.new
mv -f /usr/lib/asus-linux-control/asus-control-daemon.new \
    /usr/lib/asus-linux-control/asus-control-daemon

# SELinux: keep the daemon in its own domain (the copy above reset the binary label)
if command -v selinuxenabled >/dev/null 2>&1 && selinuxenabled; then
    echo "   Updating SELinux module and labels..."
    semodule -i selinux/asus_control.cil
    restorecon -RF /usr/lib/asus-linux-control
    if [ -d /var/lib/asus-linux-control ]; then
        restorecon -RF /var/lib/asus-linux-control
    fi
fi

# Copy integration files (in case D-Bus or polkit methods changed)
echo "   Updating D-Bus service file..."
install -m 644 dbus/com.asuslinuxcontrol.Daemon.service /usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service

echo "   Updating D-Bus config..."
install -m 644 dbus/com.asuslinuxcontrol.Daemon.conf /etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf

echo "   Updating polkit policy..."
install -m 644 dbus/com.asuslinuxcontrol.Daemon.policy /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy

echo "   Updating systemd service..."
install -m 644 systemd/asus-control-daemon.service /etc/systemd/system/asus-control-daemon.service
systemctl daemon-reload
systemctl enable --now asus-control-daemon.service

# Reload D-Bus
echo "   Reloading D-Bus..."
systemctl reload dbus 2>/dev/null || busctl call org.freedesktop.DBus / org.freedesktop.DBus ReloadConfig || true

echo ""
echo "   ✓ Installation complete!"
SUDO_SCRIPT

echo ""
echo "✓ Done! Daemon will auto-start on next D-Bus call."
echo ""
echo "Test with:"
echo "  busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetCpuBoostStatus"
