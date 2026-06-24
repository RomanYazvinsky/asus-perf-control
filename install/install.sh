#!/bin/bash
# Installation script for ASUS Linux Control Tool
# Run with sudo: sudo ./install.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== ASUS Linux Control Tool Installation ===${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Error: Please run as root (use sudo)${NC}"
    exit 1
fi

# Check for ASUS WMI driver
echo -e "${YELLOW}Checking for ASUS WMI driver...${NC}"
if [ ! -d "/sys/devices/platform/asus-nb-wmi" ]; then
    echo -e "${RED}Warning: ASUS WMI driver not found!${NC}"
    echo "This tool requires the asus-nb-wmi kernel module."
    echo "Make sure you're running on an ASUS laptop with kernel 5.4+."
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✓ ASUS WMI driver found${NC}"
fi

# Check for pre-built Rust daemon
echo ""
echo -e "${YELLOW}Checking for Rust daemon...${NC}"
DAEMON_SOURCE="../rust_daemon"
DAEMON_BINARY="$DAEMON_SOURCE/target/release/asus-control-daemon"

if [ ! -f "$DAEMON_BINARY" ]; then
    echo -e "${YELLOW}Daemon not found. Building...${NC}"
    
    if [ ! -d "$DAEMON_SOURCE" ]; then
        echo -e "${RED}Error: rust_daemon directory not found${NC}"
        echo "Please ensure you're in the install/ directory"
        exit 1
    fi

    # Check if cargo is installed (only needed if building)
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Rust toolchain not found${NC}"
        echo "Please install Rust: https://rustup.rs/"
        echo ""
        echo "Or build the daemon manually:"
        echo "  cd ../rust_daemon && cargo build --release && cd ../install"
        exit 1
    fi

    # Build daemon in release mode (WITHOUT sudo)
    echo "Compiling daemon (this may take a minute)..."
    (cd "$DAEMON_SOURCE" && cargo build --release) || {
        echo -e "${RED}Failed to build Rust daemon${NC}"
        exit 1
    }
    
    echo -e "${GREEN}✓ Daemon compiled successfully${NC}"
else
    echo -e "${GREEN}✓ Using pre-built daemon${NC}"
fi

# Install D-Bus daemon
echo ""
echo -e "${YELLOW}Installing D-Bus daemon...${NC}"
DBUS_DIR="dbus"
SYSTEMD_DIR="systemd"
DAEMON_DEST="/usr/lib/asus-linux-control"

mkdir -p "$DAEMON_DEST"

# Stop running daemon before replacing binary (avoids "Text file busy")
if systemctl is-active --quiet asus-control-daemon 2>/dev/null; then
    echo -e "${YELLOW}Stopping running daemon...${NC}"
    systemctl stop asus-control-daemon
    echo -e "${GREEN}✓ Daemon stopped${NC}"
elif pkill -0 -f asus-control-daemon 2>/dev/null; then
    pkill -f asus-control-daemon || true
    sleep 1
fi

if [ -f "$DAEMON_SOURCE/target/release/asus-control-daemon" ]; then
    cp "$DAEMON_SOURCE/target/release/asus-control-daemon" "$DAEMON_DEST/"
    chmod 755 "$DAEMON_DEST/asus-control-daemon"
    echo -e "${GREEN}✓ Rust daemon installed to $DAEMON_DEST${NC}"
else
    echo -e "${RED}Error: Compiled daemon binary not found${NC}"
    exit 1
fi

# Install D-Bus service file
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.service" ]; then
    cp "$DBUS_DIR/com.asuslinuxcontrol.Daemon.service" /usr/share/dbus-1/system-services/
    echo -e "${GREEN}✓ D-Bus service file installed${NC}"
fi

# Install D-Bus configuration
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.conf" ]; then
    cp "$DBUS_DIR/com.asuslinuxcontrol.Daemon.conf" /etc/dbus-1/system.d/
    echo -e "${GREEN}✓ D-Bus configuration installed${NC}"
fi

# Install polkit policy
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.policy" ]; then
    cp "$DBUS_DIR/com.asuslinuxcontrol.Daemon.policy" /usr/share/polkit-1/actions/
    echo -e "${GREEN}✓ Polkit policy installed${NC}"
fi

# Install and enable boot-time systemd service. D-Bus activation remains as a fallback,
# but reboot restore needs the daemon to run before the app is opened.
if [ -f "$SYSTEMD_DIR/asus-control-daemon.service" ]; then
    cp "$SYSTEMD_DIR/asus-control-daemon.service" /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable --now asus-control-daemon.service
    echo -e "${GREEN}✓ Systemd service enabled and started${NC}"
fi

# Reload D-Bus
echo -e "${YELLOW}Reloading D-Bus...${NC}"
systemctl reload dbus 2>/dev/null || true
echo -e "${GREEN}✓ D-Bus reloaded${NC}"

# Install udev rules (for non-sysfs devices)
echo ""
echo -e "${YELLOW}Installing udev rules...${NC}"
if [ -f "99-asus-control.rules" ]; then
    cp 99-asus-control.rules /etc/udev/rules.d/
    udevadm control --reload-rules
    udevadm trigger
    echo -e "${GREEN}✓ Udev rules installed${NC}"
fi

# Add user to plugdev group (if exists)
if getent group plugdev > /dev/null 2>&1; then
    if [ -n "$SUDO_USER" ]; then
        echo ""
        echo -e "${YELLOW}Adding user $SUDO_USER to plugdev group...${NC}"
        usermod -a -G plugdev "$SUDO_USER"
        echo -e "${GREEN}✓ User added to plugdev group${NC}"
        echo -e "${YELLOW}Note: You need to log out and back in for group changes to take effect${NC}"
    fi
fi

# Check for config directory
CONFIG_DIR="$HOME/.config/asus-linux-control"
if [ -n "$SUDO_USER" ]; then
    CONFIG_DIR="/home/$SUDO_USER/.config/asus-linux-control"
fi

if [ ! -d "$CONFIG_DIR" ]; then
    echo ""
    echo -e "${YELLOW}Creating config directory...${NC}"
    mkdir -p "$CONFIG_DIR"
    if [ -n "$SUDO_USER" ]; then
        chown -R "$SUDO_USER:$SUDO_USER" "$CONFIG_DIR"
    fi
    echo -e "${GREEN}✓ Config directory created at $CONFIG_DIR${NC}"
fi

echo ""
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo ""
echo "Next steps:"
echo "1. Reboot or re-login for group changes to take effect"
echo "2. Run the ASUS Linux Control application"
echo "3. If you encounter permission errors, check that:"
echo "   - Your user is in the 'plugdev' group: groups \$USER"
echo "   - The udev rules are loaded: udevadm info --query=all --name=/sys/class/platform/asus-nb-wmi"
echo ""
echo "For troubleshooting, see the README.md file."
