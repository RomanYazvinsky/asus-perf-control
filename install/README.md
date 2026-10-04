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
- `selinux/asus_control.cil` - SELinux policy module (Fedora/Bazzite), see below
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

1. Install D-Bus daemon and the ROG hotkey agent to `/usr/lib/asus-linux-control/`, and the
   desktop app to `/usr/bin/asus-linux-control` with its menu entry (the ROG/M4 key starts it
   through `gtk-launch asus-linux-control`)
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

## SELinux (Fedora, Bazzite)

Without a policy of its own, systemd runs the daemon in the generic `init_t` domain, which is
not allowed to open device nodes. The ROG/M4 key binding then fails with an
"SELinux is preventing asus-control-da from 'read, write' accesses on the chr_file hidrawN" alert.

`selinux/asus_control.cil` gives the daemon its own confined domain `asus_control_t` that may
open the ROG keyboard (`/dev/hidraw*`, `usb_device_t`) and the ASUS input event device, write
the hardware sysfs attributes, own its D-Bus name, keep state in `/var/lib/asus-linux-control`
and hand package updates to the `rpm_t` domain. Other services gain no new access.

`install.sh`, `rebuild-daemon.sh` and the RPM load it automatically. Manually:

```bash
sudo semodule -i selinux/asus_control.cil
sudo restorecon -RF /usr/lib/asus-linux-control /var/lib/asus-linux-control
sudo systemctl restart asus-control-daemon
ps -eZ | grep asus-control-da        # expect ...:system_r:asus_control_t:s0
```

If a feature is denied, list the denials and report them:

```bash
sudo ausearch -m avc -ts recent -c asus-control-da
```

Remove with `sudo semodule -r asus_control` (the daemon then runs in `init_t` again).

## Manual Installation

See `DBUS_INSTALLATION.md` for detailed manual setup instructions.

## Troubleshooting

- **Permission denied**: Check user is in `plugdev` group: `groups $USER`
- **D-Bus errors**: Check daemon installation and logs: `journalctl -f | grep asus-control`
- **Polkit authorization failed**: Verify polkit policy is installed in `/usr/share/polkit-1/actions/`
