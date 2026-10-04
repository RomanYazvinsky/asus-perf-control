# Installation Files

This directory contains installation scripts and permission configuration files for the ASUS Linux Control Tool.

## Architecture

The tool uses a **D-Bus daemon with polkit authorization** for secure access to hardware:

```text
asus-linux-control (Rust UI) → D-Bus → asus-control-daemon (root) → sysfs
                            ↑
                      Polkit authorization
```

## Files

- `99-asus-control.rules` - udev rules for USB HID devices and hwmon (read-only access)
- `dbus/` - D-Bus service activation, bus policy and polkit actions
- `systemd/` - daemon unit, ROG hotkey drop-in and the user hotkey agent unit
- `install.sh` - Installation script to deploy daemon, rules, and permissions
- `rebuild-daemon.sh` - Rebuild and reinstall only the daemon during development
- `DBUS_INSTALLATION.md` - Detailed D-Bus setup and security documentation

The daemon itself is built from `rust_daemon/` (`make install` builds it first).

## Usage

Run the installation script with sudo:

```bash
sudo ./install.sh
```

This will:

1. Install D-Bus daemon to `/usr/lib/asus-linux-control/`
2. Configure polkit authorization policies
3. Install udev rules for USB HID and hwmon access
4. Add user to `plugdev` group (if exists)
5. Reload D-Bus and udev

## Security Model

**Critical operations** (thermal policy, battery limit) are protected by:

- **D-Bus daemon** running as root with controlled access
- **Polkit authorization** - active users allowed, remote users require admin password
- **No direct sysfs access** - prevents privilege escalation bypasses

**Non-critical operations** (keyboard backlight, RGB) use udev rules with group restrictions.

## Manual Installation

See `DBUS_INSTALLATION.md` for detailed manual setup instructions.

## Troubleshooting

- **Permission denied**: Check user is in `plugdev` group: `groups $USER`
- **D-Bus errors**: Check daemon installation and logs: `journalctl -f | grep asus-control`
- **Polkit authorization failed**: Verify polkit policy is installed in `/usr/share/polkit-1/actions/`
