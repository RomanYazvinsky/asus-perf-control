# ASUS Linux Control Tool

A native Rust desktop application for controlling ASUS laptop hardware on Linux
(a G-Helper–style tool). It consists of an egui UI running as your user and a
privileged D-Bus daemon that performs root-only writes behind Polkit.

The project is **Rust only**. The former Flutter UI and `flutter_rust_bridge` FFI
layer have been removed; there is no Dart/Flutter toolchain involved anywhere.

## Features

- Performance mode (Silent / Balanced / Turbo)
- CPU TDP limits (Sustained + Boost) and CPU temperature ceiling — current values are read from the system on start
- Battery charge limit (20–100%)
- Fan curves and the daemon-driven dynamic fan engine
- GPU mode switching (via supergfxd)
- Settings profiles (`settings.json` + `profiles/*.json`)
- XG Mobile fan diagnostic
- Package update status/apply through the system package manager
- Gamepad / Steam Input navigation and ROG/M4 hotkey launch
- Single instance: launching again raises the existing window

## Architecture

```text
asus-linux-control  (rust_ui, egui, runs as user)
    │ D-Bus (zbus)                              │ sysfs reads (rust_core)
    ├── xyz.ljones.Asusd ─────── profile, TDP, charge limit, fan curves
    ├── org.supergfxctl.Daemon ─ GPU mode
    └── com.asuslinuxcontrol.Daemon (rust_daemon, root) ◄── Polkit
            temp limit, CPU boost, dynamic fan engine, package updates, ROG hotkey
```

| Crate | Role |
|---|---|
| `rust_ui/` | Desktop application, binary `asus-linux-control`. Screens in `ui/`, testable screen logic in `features/`, D-Bus clients in `backend/`. |
| `rust_core/` | Small library of read-only sysfs helpers (fan RPMs, temperatures, Steam Input), linked into `rust_ui`. |
| `rust_daemon/` | Privileged D-Bus daemon `asus-control-daemon` plus the user-session `asus-control-hotkey-agent`. |

Detailed per-feature call routes: [`docs/app-function-interactions.md`](docs/app-function-interactions.md).

## Requirements

**Runtime**

- Linux kernel 5.4+ (5.17+ for TDP control), ASUS laptop with the `asus-nb-wmi` module
- D-Bus and Polkit
- [`asusd`](https://gitlab.com/asus-linux/asusctl) for profiles / TDP / charge limit / fan curves; `supergfxd` for GPU modes (without it the GPU screen reports an error)
- Wayland or X11 session with an OpenGL (EGL) or Vulkan driver

**Development**

- Rust 1.95+ / Cargo (required by egui 0.36)
- `make`, `sudo` (for installing the daemon)
- Optional: `rpmbuild` (RPM packages), `makepkg` (pacman packages)

## Running the application

### From source (development)

```bash
# 1. Build and install the privileged daemon, D-Bus/Polkit/udev/SELinux files and the
#    desktop app (/usr/bin/asus-linux-control + menu entry, which the ROG/M4 key launches).
#    Needed once, and again after every change in rust_daemon/.
make install
#    After the very first install, log out and back in (plugdev group membership).

# 2. Build and start the UI.
make run            # release build: cargo run --release in rust_ui/
make debug          # debug build with RUST_LOG=debug tracing of every D-Bus call
```

Equivalent without `make`:

```bash
cd rust_ui && cargo run --release
```

The UI also starts without the daemon installed, but every privileged action
(temperature limit, CPU boost, fan engine, updates) will report an error.

### Installed from packages

Install the `asus-linux-control` meta package (pulls in `asus-linux-control-gui`
and `asus-linux-control-daemon`), then start **ASUS Linux Control** from the
application menu, or from a terminal:

```bash
asus-linux-control
```

The daemon is D-Bus activated, so it does not need to be started manually.

## Make targets

| Command | What it does |
|---|---|
| `make run` | Build (release) and run the UI |
| `make debug` | Debug build of the UI with `RUST_LOG=debug` |
| `make test` | Run tests of `rust_core`, `rust_ui` and `rust_daemon` |
| `make install` | Build daemon + UI and install them with D-Bus/Polkit/udev/systemd/SELinux files and the desktop entry |
| `make build-ui` | Build only the UI → `rust_ui/target/release/asus-linux-control` |
| `make build-daemon` | Build only the daemon → `rust_daemon/target/release/asus-control-daemon` |
| `make package-rpm` | Fedora/Bazzite RPMs → `dist/rpmbuild/RPMS/`, `dist/rpmbuild/SRPMS/` |
| `make package-arch` | Arch/SteamOS pacman packages → `dist/arch/` |
| `make package-all` | Both package formats |
| `make clean` | Remove all `target/` directories and `dist/` |

## Development workflow

After changes in `rust_ui/` or `rust_core/`:

```bash
make test
cd rust_ui && cargo clippy --all-targets -- -D warnings
make run
```

After changes in `rust_daemon/`:

```bash
make install                              # rebuild + reinstall
pkill -f asus-control-daemon || true      # stop the old process;
                                          # D-Bus re-activates it on the next call
```

See `install/DBUS_INSTALLATION.md` for the manual daemon setup and update procedure.

## Packages and updates

The recommended install target is the native `asus-linux-control` package.
Updates go through the operating system package manager and signed repositories:

- Fedora — RPM/DNF
- Bazzite — RPM layered with rpm-ostree (a reboot is usually required after updates are staged)
- Arch Linux — pacman
- SteamOS — advanced target; may report unsupported when host pacman updates are not available

Flatpak, a custom binary downloader and GUI-provided package-manager commands are
not supported. The GUI talks to the installed daemon over D-Bus; the daemon
selects a fixed backend and a fixed command allowlist.

## Versioning

Releases use semantic versioning; the current version is `0.0.1`, tagged as
`v<semver>` (e.g. `v0.0.1`). Keep `rust_ui/Cargo.toml`, `Makefile`, the RPM spec,
the PKGBUILD and the metainfo in sync when bumping. Full rules:
[`docs/release-versioning.md`](docs/release-versioning.md).

## Hardware reference

| Setting | Sysfs attribute | Range | Path |
|---|---|---|---|
| Performance mode | `throttle_thermal_policy` | 0 Silent / 1 Balanced / 2 Turbo | asusd `xyz.ljones.Platform.PlatformProfile` |
| Sustained TDP | `ppt_pl1_spl` | 5–54 W | asusd |
| Boost TDP | `ppt_pl2_sppt` | 5–54 W | asusd |
| CPU temperature limit | `ppt_cpu_limit` | 60–100 °C | own daemon |
| Battery charge limit | `charge_control_end_threshold` | 20–100 % | asusd |

TDP and temperature values are volatile and reset on reboot; the app reads the
current values from the system on start.

## Configuration files

- Settings: `$XDG_CONFIG_HOME/asus-linux-control/settings.json`
- Profiles: `$XDG_CONFIG_HOME/asus-linux-control/profiles/*.json`
- XG Mobile diagnostic session: `$XDG_STATE_HOME/asus-linux-control/xg_mobile_fan_diagnostic.json`

The file formats are unchanged from the former Flutter version, so existing
settings keep working.

## Project structure

```text
.
├── rust_ui/        # Desktop application (egui)
│   └── src/
│       ├── main.rs         # single instance + eframe bootstrap
│       ├── app.rs          # navigation shell, alerts
│       ├── backend/        # zbus clients: asusd, supergfxd, own daemon
│       ├── features/       # per-screen logic behind service traits (unit-tested)
│       ├── settings/       # settings.json + profiles
│       └── ui/             # screens, widgets, fan curve editor
├── rust_core/      # read-only sysfs helpers used by the UI
├── rust_daemon/    # privileged D-Bus daemon + hotkey agent
├── install/        # install.sh, D-Bus/Polkit/udev/systemd files, SELinux module
├── packaging/      # RPM spec, PKGBUILD, .desktop, metainfo, icon
├── docs/           # architecture notes, versioning, hardware research
└── Makefile
```

## Documentation

- [`docs/app-function-interactions.md`](docs/app-function-interactions.md) — UI → feature → D-Bus/sysfs routes for every function
- [`docs/release-versioning.md`](docs/release-versioning.md) — release and version-bump rules
- [`install/README.md`](install/README.md), [`install/DBUS_INSTALLATION.md`](install/DBUS_INSTALLATION.md) — daemon installation and security model
- [`packaging/README.md`](packaging/README.md) — native packaging
- [`rust_daemon/README.md`](rust_daemon/README.md) — daemon internals
- [`docs/research/`](docs/research/) — hardware research and manual verification guides (fan control, package updates, [ROG/M4 hotkey](docs/research/rog-m4-hotkey-manual-verification.md))

The ASUS M-key firmware report sequence is adapted from G-Helper by seerge:
https://github.com/seerge/g-helper/pull/5691

## Troubleshooting

**App won't start / no window**

```bash
RUST_LOG=debug asus-linux-control      # or: make debug
```

A second launch only raises the existing window. A stale single-instance socket
is removed automatically: `$XDG_RUNTIME_DIR/asus-linux-control.sock`.

**Permission denied**

```bash
ls -l /etc/udev/rules.d/99-asus-control.rules
groups $USER                           # must include plugdev
sudo udevadm control --reload-rules && sudo udevadm trigger
```

**ASUS WMI interface not found**

```bash
lsmod | grep asus
ls -la /sys/class/platform/asus-nb-wmi/
sudo modprobe asus-nb-wmi
```

**SELinux alert: "SELinux is preventing asus-control-da …"**

The daemon needs the `asus_control` SELinux module (loaded by `make install` and the RPM).
Check that it runs in its own domain:

```bash
ps -eZ | grep asus-control-da          # expect asus_control_t, not init_t
```

If it shows `init_t`, rerun `make install`. Details: [`install/README.md`](install/README.md#selinux-fedora-bazzite).

**Daemon errors**

```bash
journalctl -f | grep asus-control
```

## License

MIT, as declared in `rust_ui/Cargo.toml` and the package metadata. Reference projects: G-Helper, asusctl, Starlight.
