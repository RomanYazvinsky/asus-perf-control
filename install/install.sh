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
UI_SOURCE="$PROJECT_ROOT/rust_ui"
UI_BINARY="$UI_SOURCE/target/release/asus-linux-control"
PACKAGING_LINUX_DIR="$PROJECT_ROOT/packaging/linux"

print_prebuild_help() {
    echo -e "${RED}Unable to find a usable Rust toolchain for the regular user.${NC}"
    echo "Build the release binaries as your regular user, then rerun this installer:"
    echo "  cd '$DAEMON_SOURCE' && cargo build --release --bins"
    echo "  cd '$UI_SOURCE' && cargo build --release --bins"
}

# Builds the release binaries of one crate (default: rust_daemon) as the invoking user.
build_release_bins() {
    local MANIFEST="${1:-$DAEMON_SOURCE/Cargo.toml}"
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
        "$CARGO_BIN" build --release --bins --manifest-path "$MANIFEST"
}

install_binary_atomically() {
    local source="$1"
    local destination="$2"
    local temporary
    temporary="$(mktemp "${destination%/*}/.${destination##*/}.tmp.XXXXXX")"
    TEMP_INSTALLS+=("$temporary")
    install -m 0755 "$source" "$temporary"
    # Label the file for its final path before it appears there: D-Bus activation may start
    # the daemon the moment it is moved in, and with the temporary file's label it would run
    # in init_t instead of asus_control_t.
    if command -v selinuxenabled > /dev/null 2>&1 && selinuxenabled \
        && command -v matchpathcon > /dev/null 2>&1; then
        local context
        if context="$(matchpathcon -n "$destination" 2> /dev/null)"; then
            chcon "$context" "$temporary" || true
        fi
    fi
    mv -f "$temporary" "$destination"
    if command -v restorecon > /dev/null 2>&1; then
        restorecon -F "$destination" 2> /dev/null || true
    fi
}

# On SELinux systems the daemon needs its own domain (asus_control_t); in the generic init_t
# domain it may not open the ROG keyboard hidraw/input devices. Installing a binary through a
# temporary file resets its label, so labels are restored on every install.
install_selinux_policy() {
    if ! command -v selinuxenabled > /dev/null 2>&1 || ! selinuxenabled; then
        return 0
    fi
    echo ""
    echo -e "${YELLOW}Installing SELinux policy module...${NC}"
    if ! command -v semodule > /dev/null 2>&1 || ! command -v restorecon > /dev/null 2>&1; then
        echo -e "${RED}Warning: semodule/restorecon not found; the ROG/M4 key will not work under SELinux${NC}"
        return 0
    fi
    if ! semodule -i "$SCRIPT_DIR/selinux/asus_control.cil"; then
        echo -e "${RED}Warning: failed to load the SELinux module; the ROG/M4 key will not work${NC}"
        return 0
    fi
    restorecon -RF "$DAEMON_DEST"
    if [ -d /var/lib/asus-linux-control ]; then
        restorecon -RF /var/lib/asus-linux-control
    fi
    echo -e "${GREEN}✓ SELinux module asus_control loaded and labels restored${NC}"
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

if [ ! -f "$UI_BINARY" ]; then
    echo -e "${YELLOW}Desktop application not found. Building...${NC}"
    build_release_bins "$UI_SOURCE/Cargo.toml" || {
        echo -e "${RED}Failed to build the desktop application${NC}"
        exit 1
    }
    echo -e "${GREEN}✓ Desktop application compiled successfully${NC}"
else
    echo -e "${GREEN}✓ Using pre-built desktop application${NC}"
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
    # Load the policy first so the new binary gets its label as it is installed.
    install_selinux_policy
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

# Install the desktop application. The ROG hotkey agent starts it with
# `gtk-launch asus-linux-control`, which only works when the desktop entry is installed.
echo ""
echo -e "${YELLOW}Installing desktop application...${NC}"
install_binary_atomically "$UI_BINARY" /usr/bin/asus-linux-control
install -D -m 0644 "$PACKAGING_LINUX_DIR/asus-linux-control.desktop" \
    /usr/share/applications/asus-linux-control.desktop
install -D -m 0644 "$PACKAGING_LINUX_DIR/asus-linux-control.png" \
    /usr/share/icons/hicolor/256x256/apps/asus-linux-control.png
install -D -m 0644 "$PACKAGING_LINUX_DIR/asus-linux-control.metainfo.xml" \
    /usr/share/metainfo/asus-linux-control.metainfo.xml
if command -v update-desktop-database > /dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache > /dev/null 2>&1; then
    gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
echo -e "${GREEN}✓ Desktop application installed to /usr/bin/asus-linux-control${NC}"

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
ROG_HOTKEY_DROPIN_DIR="/etc/systemd/system/asus-control-daemon.service.d"
if [ -f "$SYSTEMD_DIR/asus-control-daemon.service.d/rog-hotkey.conf" ]; then
    install -d -m 0755 "$ROG_HOTKEY_DROPIN_DIR"
    install -m 0644 "$SYSTEMD_DIR/asus-control-daemon.service.d/rog-hotkey.conf" \
        "$ROG_HOTKEY_DROPIN_DIR/rog-hotkey.conf"
    echo -e "${GREEN}✓ ROG hotkey daemon opt-in installed${NC}"
fi

if [ -f "$SYSTEMD_DIR/asus-control-daemon.service" ]; then
    install -m 0644 "$SYSTEMD_DIR/asus-control-daemon.service" /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable asus-control-daemon.service
    # Restart rather than `enable --now`: if D-Bus activation already started the daemon during
    # the install, it must still be replaced by a process running the new binary and policy.
    systemctl restart asus-control-daemon.service
    echo -e "${GREEN}✓ Systemd service enabled and restarted${NC}"
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
