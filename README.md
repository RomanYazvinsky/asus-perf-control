# ASUS Linux Control

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![Rust 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange.svg)](https://www.rust-lang.org)
![Platform: Linux](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

A native control center for ASUS ROG / TUF laptops on Linux, in the spirit of
[G-Helper](https://github.com/seerge/g-helper). It manages performance modes, power and
temperature limits, fans, GPU modes and battery charging from one window. The ROG key opens it.

Written in Rust: an [egui](https://github.com/emilk/egui) desktop app running as your user,
plus a small privileged D-Bus daemon behind Polkit. It builds on
[asusd](https://gitlab.com/asus-linux/asusctl) and
[supergfxd](https://gitlab.com/asus-linux/supergfxctl) instead of replacing them.

## Contents

- [Features](#features)
- [Compatibility](#compatibility)
- [Installation](#installation)
- [Usage](#usage)
- [Documentation](#documentation)
- [Development](#development)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Features

| | |
|---|---|
| **Performance mode** | Silent / Balanced / Turbo, plus CPU boost presets (Off, Efficient, Balanced, Aggressive) |
| **Power limits** | Sustained and boost PPT per platform profile, CPU temperature ceiling (AMD, via `ryzenadj`) |
| **Fans** | 8-point curves, constant speed, or a daemon-driven dynamic engine with a high-temperature safety floor; max-speed test |
| **GPU modes** | Integrated, Hybrid, Ultimate, MUX dGPU, XG Mobile, VFIO through supergfxd |
| **Battery** | Charge limit 20–100 % |
| **Status toolbar** | Live CPU / iGPU / dGPU name, temperature, clock and fan RPM. A sleeping dGPU is never woken to read it |
| **Profiles** | Named sets of hardware settings, applied in one click |
| **ROG key** | Opens or raises the app; native GNOME shortcut with proper Wayland focus |
| **Updates** | Package updates through dnf, rpm-ostree or pacman, from inside the app |
| **XG Mobile diagnostic** | Fan PWM sweep with hwmon sampling for the XG Mobile dock |

Every feature is described in [`docs/features/`](docs/features/README.md).

## Compatibility

- **Hardware:** ASUS laptops supported by the `asus-nb-wmi` kernel driver and asusd.
  Developed and tested on a ROG Flow X13 (GV301QH) with an XG Mobile dock; features the
  hardware doesn't expose are reported as unsupported.
- **Distributions:** Fedora, Bazzite, Arch Linux; SteamOS as an advanced target.
- **Desktops:** any Wayland or X11 session. GNOME gets a native ROG key shortcut.

| Dependency | Required for |
|---|---|
| Linux 5.4+ (5.17+ for PPT), `asus-nb-wmi` | everything |
| D-Bus, Polkit, systemd | daemon |
| asusd | performance mode, PPT, charge limit, fan curves |
| supergfxd | GPU modes (optional) |
| ryzenadj | CPU temperature limit, AMD only (optional) |

## Installation

### Native packages (recommended)

```bash
make package-rpm     # Fedora / Bazzite → dist/rpmbuild/RPMS/
make package-arch    # Arch / SteamOS   → dist/arch/
```

Then install the `asus-linux-control` package with `dnf`, `rpm-ostree` or `pacman`. It pulls
in the GUI and the daemon.

### From source

```bash
git clone https://github.com/RomanYazvinsky/asus-perf-control.git
cd asus-perf-control
make install         # builds and installs the daemon, the app and all system files (sudo)
```

Log out and back in after the first install.

Full guide, verification and uninstall: [`docs/guides/installation.md`](docs/guides/installation.md).

## Usage

Start **ASUS Linux Control** from the application menu, press the **ROG key**, or run:

```bash
asus-linux-control
```

Launching it again raises the existing window. Settings are stored in
`~/.config/asus-linux-control/` (`settings.json` + `profiles/*.json`).

Something not working? See [troubleshooting](docs/guides/troubleshooting.md), or run with
`RUST_LOG=debug` to trace every D-Bus call.

## Documentation

| | |
|---|---|
| [Architecture](docs/architecture.md) | Processes, ownership of each hardware setting, persistence, security model |
| [Features](docs/features/README.md) | What each feature does and how it works |
| [Modules](docs/modules/README.md) | `rust_ui`, `rust_core`, `rust_daemon` (incl. D-Bus API), install & packaging files |
| [Guides](docs/README.md) | Installation, development, troubleshooting, packaging, releases |
| [Research](docs/research/) | Hardware investigations and manual test plans |

```text
asus-linux-control (rust_ui, user)
    │ D-Bus                                      │ sysfs reads (rust_core)
    ├── asusd ──────────── profile, PPT, charge limit, fan curves
    ├── supergfxd ──────── GPU mode
    └── asus-control-daemon (rust_daemon, root) ◄── Polkit
            temperature limit, CPU boost, dynamic fans, ROG key, package updates
```

## Development

```bash
make run             # build and run the UI (release)
make debug           # debug build with RUST_LOG=debug
make test            # tests of all three crates
make install         # reinstall after daemon changes
```

| Directory | Contents |
|---|---|
| `rust_ui/` | Desktop app (egui): screens, screen logic, D-Bus clients, settings |
| `rust_core/` | Read-only hardware readers linked into the app |
| `rust_daemon/` | Privileged D-Bus daemon and the ROG hotkey agent |
| `install/` | Install script, D-Bus, Polkit, systemd, udev, SELinux files |
| `packaging/` | RPM spec, PKGBUILD, desktop entry, AppStream metadata, icon |
| `docs/` | Documentation |

Conventions, debugging and commit style: [`docs/guides/development.md`](docs/guides/development.md).
Release process: [`docs/guides/release.md`](docs/guides/release.md).

## Acknowledgements

- [G-Helper](https://github.com/seerge/g-helper) by seerge. The ASUS M-key firmware report
  sequence is adapted from [PR #5691](https://github.com/seerge/g-helper/pull/5691).
- [asusctl / asusd and supergfxctl](https://gitlab.com/asus-linux) by the asus-linux project.
- [RyzenAdj](https://github.com/FlyGoat/RyzenAdj).

## License

MIT, as declared in the crate and package metadata.
