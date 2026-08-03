#!/bin/bash
# Installation script for ASUS Linux Control Tool
# Run with sudo: sudo ./install.sh

set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"

TEMP_INSTALLS=()
cleanup_temp_installs() {
    local temporary
    for temporary in "${TEMP_INSTALLS[@]}"; do
        [ -n "$temporary" ] && rm -f -- "$temporary"
    done
}
trap cleanup_temp_installs EXIT

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
DAEMON_SOURCE="$PROJECT_ROOT/rust_daemon"
DAEMON_BINARY="$DAEMON_SOURCE/target/release/asus-control-daemon"
AGENT_BINARY="$DAEMON_SOURCE/target/release/asus-control-hotkey-agent"

print_prebuild_help() {
    echo -e "${RED}Unable to find a usable Rust toolchain for the regular user.${NC}"
    echo "Build the release binaries as your regular user, then rerun this installer:"
    echo "  cd '$DAEMON_SOURCE' && cargo build --release --bins"
}

build_release_bins() {
    local BUILD_HOME BUILD_PATH CARGO_BIN

    if [ -z "${SUDO_USER:-}" ] || [ "$SUDO_USER" = "root" ]; then
        print_prebuild_help
        return 1
    fi

    if ! command -v runuser > /dev/null 2>&1; then
        print_prebuild_help
        return 1
    fi

    BUILD_HOME="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
    if [ -z "$BUILD_HOME" ] || [ ! -d "$BUILD_HOME" ]; then
        print_prebuild_help
        return 1
    fi

    BUILD_PATH="$BUILD_HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    CARGO_BIN="$(runuser -u "$SUDO_USER" -- env \
        HOME="$BUILD_HOME" USER="$SUDO_USER" LOGNAME="$SUDO_USER" PATH="$BUILD_PATH" \
        sh -c 'command -v cargo' 2>/dev/null || true)"
    if [ -z "$CARGO_BIN" ] || [ ! -x "$CARGO_BIN" ]; then
        print_prebuild_help
        return 1
    fi

    runuser -u "$SUDO_USER" -- env \
        HOME="$BUILD_HOME" USER="$SUDO_USER" LOGNAME="$SUDO_USER" PATH="$BUILD_PATH" \
        "$CARGO_BIN" build --release --bins --manifest-path "$DAEMON_SOURCE/Cargo.toml"
}

install_binary_atomically() {
    local source="$1"
    local destination="$2"
    local temporary
    temporary="$(mktemp "$DAEMON_DEST/.${destination##*/}.tmp.XXXXXX")"
    TEMP_INSTALLS+=("$temporary")
    install -m 0755 "$source" "$temporary"
    mv -f "$temporary" "$destination"
}

if [ ! -f "$DAEMON_BINARY" ] || [ ! -f "$AGENT_BINARY" ]; then
    echo -e "${YELLOW}Daemon or hotkey agent not found. Building...${NC}"
    
    if [ ! -d "$DAEMON_SOURCE" ]; then
        echo -e "${RED}Error: rust_daemon directory not found${NC}"
        echo "Please ensure you're in the install/ directory"
        exit 1
    fi

    echo "Compiling daemon and hotkey agent (this may take a minute)..."
    build_release_bins || {
        echo -e "${RED}Failed to build Rust daemon and hotkey agent${NC}"
        exit 1
    }
    
    echo -e "${GREEN}✓ Daemon compiled successfully${NC}"
else
    echo -e "${GREEN}✓ Using pre-built daemon${NC}"
fi

# Install D-Bus daemon
echo ""
echo -e "${YELLOW}Installing D-Bus daemon...${NC}"
DBUS_DIR="$SCRIPT_DIR/dbus"
SYSTEMD_DIR="$SCRIPT_DIR/systemd"
DAEMON_DEST="/usr/lib/asus-linux-control"

install -d -m 0755 "$DAEMON_DEST"

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
    install_binary_atomically "$DAEMON_BINARY" "$DAEMON_DEST/asus-control-daemon"
    echo -e "${GREEN}✓ Rust daemon installed to $DAEMON_DEST${NC}"
else
    echo -e "${RED}Error: Compiled daemon binary not found${NC}"
    exit 1
fi

if [ -f "$AGENT_BINARY" ]; then
    install_binary_atomically "$AGENT_BINARY" "$DAEMON_DEST/asus-control-hotkey-agent"
    echo -e "${GREEN}✓ ROG hotkey session agent installed to $DAEMON_DEST${NC}"
else
    echo -e "${RED}Error: Compiled hotkey agent binary not found${NC}"
    exit 1
fi

# Install D-Bus service file
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.service" ]; then
    install -m 0644 "$DBUS_DIR/com.asuslinuxcontrol.Daemon.service" /usr/share/dbus-1/system-services/
    echo -e "${GREEN}✓ D-Bus service file installed${NC}"
fi

# Install D-Bus configuration
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.conf" ]; then
    install -m 0644 "$DBUS_DIR/com.asuslinuxcontrol.Daemon.conf" /etc/dbus-1/system.d/
    echo -e "${GREEN}✓ D-Bus configuration installed${NC}"
fi

# Install polkit policy
if [ -f "$DBUS_DIR/com.asuslinuxcontrol.Daemon.policy" ]; then
    install -m 0644 "$DBUS_DIR/com.asuslinuxcontrol.Daemon.policy" /usr/share/polkit-1/actions/
    echo -e "${GREEN}✓ Polkit policy installed${NC}"
fi

# Install and enable boot-time systemd service. D-Bus activation remains as a fallback,
# but reboot restore needs the daemon to run before the app is opened.
if [ -f "$SYSTEMD_DIR/asus-control-daemon.service" ]; then
    install -m 0644 "$SYSTEMD_DIR/asus-control-daemon.service" /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable --now asus-control-daemon.service
    echo -e "${GREEN}✓ Systemd service enabled and started${NC}"
fi

# Install the per-user hotkey agent globally. The relative wants link enables it for graphical
# sessions without trying to contact a user's session bus from this root installer.
USER_SYSTEMD_DIR="/usr/lib/systemd/user"
USER_WANTS_DIR="$USER_SYSTEMD_DIR/graphical-session.target.wants"
if [ -f "$SYSTEMD_DIR/asus-control-hotkey-agent.service" ]; then
    install -d -m 0755 "$USER_WANTS_DIR"
    install -m 0644 "$SYSTEMD_DIR/asus-control-hotkey-agent.service" "$USER_SYSTEMD_DIR/"
    ln -sfn ../asus-control-hotkey-agent.service \
        "$USER_SYSTEMD_DIR/graphical-session.target.wants/asus-control-hotkey-agent.service"
    echo -e "${GREEN}✓ ROG hotkey session agent enabled for graphical sessions${NC}"
fi

# Reload D-Bus
echo -e "${YELLOW}Reloading D-Bus...${NC}"
systemctl reload dbus 2>/dev/null || true
echo -e "${GREEN}✓ D-Bus reloaded${NC}"

# Install udev rules (for non-sysfs devices)
echo ""
echo -e "${YELLOW}Installing udev rules...${NC}"
if [ -f "$SCRIPT_DIR/99-asus-control.rules" ]; then
    install -m 0644 "$SCRIPT_DIR/99-asus-control.rules" /etc/udev/rules.d/
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
