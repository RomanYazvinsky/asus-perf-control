# `install/` and `packaging/` — system integration

## `install/`

| File | Installed to | Purpose |
|---|---|---|
| `install.sh` | — | Installs everything below from a source build (`make install`) |
| `rebuild-daemon.sh` | — | Rebuild and reinstall only the daemon |
| `dbus/com.asuslinuxcontrol.Daemon.service` | `/usr/share/dbus-1/system-services/` | D-Bus activation → systemd unit |
| `dbus/com.asuslinuxcontrol.Daemon.conf` | `/etc/dbus-1/system.d/` | Bus policy: only root owns the name |
| `dbus/com.asuslinuxcontrol.Daemon.policy` | `/usr/share/polkit-1/actions/` | Polkit actions |
| `systemd/asus-control-daemon.service` | `/etc/systemd/system/` (`/usr/lib/systemd/system/` in packages) | Daemon unit, `WantedBy=multi-user.target` |
| `systemd/asus-control-daemon.service.d/rog-hotkey.conf` | unit drop-in dir | `ASUS_ROG_HOTKEY_ENABLED=1` |
| `systemd/asus-control-hotkey-agent.service` | user unit dir | Hotkey agent, `WantedBy=graphical-session.target` |
| `99-asus-control.rules` | `/etc/udev/rules.d/` | `plugdev` access to ASUS USB HID and hwmon |
| `selinux/asus_control.cil` | SELinux module store | Confined `asus_control_t` domain |

### Polkit actions

| Action | Used by |
|---|---|
| `com.asuslinuxcontrol.set-temp-limit` | `SetTempLimit`, `SetProfileTempLimit` |
| `com.asuslinuxcontrol.set-cpu-boost-mode` | `SetCpuBoostMode` |
| `com.asuslinuxcontrol.manage-fan-engine` | `StartFanEngine`, `UpdateFanEngineCurve`, `StopFanEngine` |
| `com.asuslinuxcontrol.manage-updates` | `StartUpdate`, `CancelUpdate` |

Inactive and remote sessions always need admin authentication. The active local user needs no
password, except for `manage-updates`, which always asks for admin authentication.

### SELinux

Without its own policy systemd runs the daemon in `init_t`, which may not open device nodes;
the ROG key then fails with an AVC denial on `hidrawN`. The module gives the daemon a confined
`asus_control_t` domain that may:

- open the ROG keyboard (`/dev/hidraw*`, `usb_device_t`) and the ASUS input event device,
- write the hardware sysfs attributes,
- own its D-Bus name,
- keep state in `/var/lib/asus-linux-control`,
- hand package updates over to the `rpm_t` domain.

Other services get no new access.

## `packaging/`

| Path | Purpose |
|---|---|
| `rpm/asus-linux-control.spec` | Fedora / Bazzite RPMs |
| `arch/PKGBUILD`, `arch/*.install` | Arch / SteamOS pacman packages |
| `linux/asus-linux-control.desktop` | Menu entry (also used by `gtk-launch`) |
| `linux/asus-linux-control.metainfo.xml` | AppStream metadata |
| `linux/asus-linux-control.png` | Icon |

Both formats build three packages:

| Package | Contents |
|---|---|
| `asus-linux-control` | meta package, depends on the two below |
| `asus-linux-control-gui` | `/usr/bin/asus-linux-control`, desktop entry, icon, metainfo, hotkey agent binary + user unit, ROG hotkey drop-in |
| `asus-linux-control-daemon` | daemon, its systemd unit, D-Bus / Polkit / udev files, SELinux module |

How to build them: [packaging guide](../guides/packaging.md).
