# Troubleshooting

Start with logs:

```bash
RUST_LOG=debug asus-linux-control          # UI: every D-Bus call with timing and errors
journalctl -u asus-control-daemon -f       # daemon
journalctl --user -u asus-control-hotkey-agent -f
```

Every error dialog in the app has a copy button for its technical details. Include it in bug
reports.

## The app doesn't open / no window

- A second launch only raises the existing window. A stale socket from a crashed instance
  (`$XDG_RUNTIME_DIR/asus-linux-control.sock`) is removed automatically.
- No OpenGL/EGL driver → eframe can't create a window; check the terminal output.

## The window opens behind other windows (GNOME)

GNOME only raises windows that present an activation token. Start the app from the menu, the
ROG key shortcut, or a terminal, not from a background script. Check that the shortcut exists:

```bash
gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings
```

## "Not authorized" / Polkit errors

```bash
ls /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy
pkcheck --action-id com.asuslinuxcontrol.set-temp-limit --process $$ -u
```

Writes are allowed without a password only for the active local session.

## A screen says the service is unavailable

| Screen | Needs |
|---|---|
| Performance, TDP, Battery, Fans | `asusd` running: `systemctl status asusd` |
| GPU | `supergfxd` running: `systemctl status supergfxd` |
| Temperature limit, CPU boost, dynamic fans, Updates | `asus-control-daemon`: `systemctl status asus-control-daemon` |

The CPU temperature limit also needs `ryzenadj` at `/usr/local/bin/ryzenadj` (AMD only).

## ASUS WMI interface not found

```bash
lsmod | grep asus
ls /sys/class/platform/asus-nb-wmi/ 2>/dev/null || ls /sys/devices/platform/asus-nb-wmi/
sudo modprobe asus-nb-wmi
```

## Permission denied on HID / hwmon

```bash
ls -l /etc/udev/rules.d/99-asus-control.rules
groups "$USER"                              # must include plugdev
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## The ROG key does nothing

```bash
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetRogHotkeyStatus | tr ',' '\n'
```

- `hardwareSupported: false`: read `reason`. Try `ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1` in a
  drop-in for `asus-control-daemon.service`.
- `lastError` mentions permission denied on `hidraw` → SELinux, see below.
- Agent not running → `systemctl --user enable --now asus-control-hotkey-agent`.

Full test plan: [`rog-m4-hotkey-manual-verification.md`](../research/rog-m4-hotkey-manual-verification.md).

## SELinux: "SELinux is preventing asus-control-da …"

```bash
ps -eZ | grep asus-control-da               # expect asus_control_t, not init_t
sudo semodule -i install/selinux/asus_control.cil
sudo restorecon -RF /usr/lib/asus-linux-control /var/lib/asus-linux-control
sudo systemctl restart asus-control-daemon
sudo ausearch -m avc -ts recent -c asus-control-da    # remaining denials, please report
```

## Updates screen says "unsupported"

The daemon supports Fedora (dnf/dnf5), Bazzite (rpm-ostree), Arch and SteamOS (pacman, if the
host is writable). Automatic updates also need the signed project repository; locally
installed package files are install-only.
