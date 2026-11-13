# Installation

## Requirements

| Component | Needed for |
|---|---|
| ASUS laptop with the `asus-nb-wmi` kernel module, Linux 5.4+ (5.17+ for PPT) | everything |
| D-Bus, Polkit, systemd | daemon |
| [asusd](https://gitlab.com/asus-linux/asusctl) | performance mode, PPT, charge limit, fan curves |
| [supergfxd](https://gitlab.com/asus-linux/supergfxctl) | GPU modes (optional) |
| [ryzenadj](https://github.com/FlyGoat/RyzenAdj) at `/usr/local/bin/ryzenadj` | CPU temperature limit (AMD only, optional) |
| `nvidia-smi` | GPU temperature/clock with the proprietary NVIDIA driver (optional) |
| Wayland or X11 with OpenGL (EGL) | UI |

## From packages (recommended)

The `asus-linux-control` meta package pulls in `asus-linux-control-gui` and
`asus-linux-control-daemon`. A public signed repository is not published yet, so build the
packages locally ([packaging guide](packaging.md)) and install the files:

```bash
make package-rpm      # or: make package-arch

# Fedora
sudo dnf install dist/rpmbuild/RPMS/*/asus-linux-control*.rpm
# Bazzite
rpm-ostree install dist/rpmbuild/RPMS/*/asus-linux-control*.rpm && systemctl reboot
# Arch Linux / SteamOS
sudo pacman -U dist/arch/asus-linux-control*.pkg.tar.zst
```

Once a repository is configured, `dnf install` / `rpm-ostree install` / `pacman -S
asus-linux-control` work the same way.

Then start **ASUS Linux Control** from the application menu, press the ROG key, or run
`asus-linux-control`. The daemon is enabled by the package and is also D-Bus activated.

With a repository configured, updates arrive through the normal system update or the app's
**Updates** screen.

## From source

```bash
git clone https://github.com/RomanYazvinsky/asus-perf-control.git
cd asus-perf-control
make install        # builds daemon + UI, installs them with all system files (uses sudo)
```

`make install` (via `install/install.sh`):

1. installs `asus-control-daemon` and `asus-control-hotkey-agent` to
   `/usr/lib/asus-linux-control/`, the app to `/usr/bin/asus-linux-control`, plus menu entry,
   icon and metainfo;
2. installs the D-Bus activation file, bus policy and Polkit actions;
3. installs and enables the daemon unit with the ROG hotkey drop-in, and the user hotkey agent;
4. loads the SELinux module when SELinux is present;
5. installs the udev rules and adds you to `plugdev`;
6. reloads D-Bus and udev.

After the very first install, **log out and back in** (group membership, user agent).

## Verify

```bash
systemctl status asus-control-daemon
systemctl --user status asus-control-hotkey-agent
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetRogHotkeyStatus
ps -eZ | grep asus-control-da          # SELinux systems: expect asus_control_t
```

## Uninstall (source install)

```bash
sudo systemctl disable --now asus-control-daemon
systemctl --user disable --now asus-control-hotkey-agent
sudo rm -rf /usr/lib/asus-linux-control /usr/bin/asus-linux-control \
  /usr/share/applications/asus-linux-control.desktop \
  /usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service \
  /etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf \
  /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy \
  /etc/systemd/system/asus-control-daemon.service \
  /etc/systemd/system/asus-control-daemon.service.d \
  /etc/udev/rules.d/99-asus-control.rules
sudo semodule -r asus_control 2>/dev/null || true
# GNOME: remove the ROG key shortcut in Settings → Keyboard → Custom Shortcuts
```

Saved daemon state lives in `/var/lib/asus-linux-control/`, user settings in
`~/.config/asus-linux-control/`.
