# ASUS Linux Control Tool

A native Rust application (egui UI + privileged D-Bus daemon) for controlling ASUS laptop hardware on Linux.

## Features

- ✅ Performance mode control (Silent / Balanced / Turbo)
- ✅ CPU TDP limits (Sustained + Boost) — загружает текущие значения из системы при старте
- ✅ CPU temperature ceiling
- ✅ Battery charge limit (20–100%)
- ✅ D-Bus daemon с Polkit-авторизацией для всех write-операций
- ✅ Material Design 3 dark theme
- ✅ Native Rust UI (egui), один бинарник без FFI
- ✅ Type-safe Rust backend

## Architecture

```
asus-linux-control (egui, user space)
    │ D-Bus (zbus)                         │ sysfs read (rust_core)
    ├── xyz.ljones.Asusd ── profile, TDP, charge limit, fan curves
    ├── org.supergfxctl.Daemon ── GPU mode
    └── com.asuslinuxcontrol.Daemon (root) ←── Polkit authorization
            temp limit, CPU boost, dynamic fan engine, package updates
```

`rust_ui` — desktop application: navigation shell, screens and per-screen logic
(`features/*`, former BLoC). `rust_core` — sysfs readers (fans, temperatures) linked
directly as an `rlib`. `rust_daemon` — privileged daemon, unchanged.

## Requirements

**System:**
- Linux kernel 5.4+ (5.17+ для TDP control)
- ASUS ноутбук с модулем `asus-nb-wmi`
- D-Bus + Polkit

**Dev:**
- Rust 1.95+ / Cargo (требование egui 0.36)
- Wayland or X11 session, OpenGL/Vulkan driver

## Quick Start

### Native Packages And Updates

The recommended user install target is the native `asus-linux-control` package.
It is a meta package that installs both `asus-linux-control-gui` and
`asus-linux-control-daemon`.

Updates use the operating system package manager and signed repositories:

- Fedora uses RPM/DNF.
- Bazzite uses RPM layered with rpm-ostree and generally requires a reboot after
  updates are staged.
- Arch Linux uses pacman.
- SteamOS is an advanced target and may report unsupported when host pacman
  updates are not available.

This feature does not support Flatpak packaging, a custom binary downloader, or
GUI-provided package-manager commands. The GUI talks to the installed daemon
over D-Bus; the daemon selects a fixed backend and a fixed command allowlist.

### 1. Установить daemon (один раз, после каждого обновления daemon)

```bash
make install
```

Собирает `rust_daemon`, копирует в `/usr/lib/asus-linux-control/`, устанавливает polkit и udev rules.

> После первой установки перелогиньтесь (для группы `plugdev`).

### 2. Запустить приложение

```bash
make run
```

Собирает `rust_ui` (вместе с `rust_core`) и запускает приложение.

**Вручную:**

```bash
cd rust_ui && cargo run --release
```

Повторный запуск (например, ROG-клавишей через `gtk-launch asus-linux-control`) не
открывает второе окно, а поднимает уже запущенное.

### Все команды make

| Команда | Что делает |
|---|---|
| `make run` | Собрать и запустить UI |
| `make debug` | Debug-сборка UI с трассировкой D-Bus вызовов (`RUST_LOG=debug`) |
| `make test` | Тесты `rust_core`, `rust_ui`, `rust_daemon` |
| `make install` | Собрать rust_daemon + установить в систему |
| `make build-ui` | Только UI (`rust_ui/target/release/asus-linux-control`) |
| `make build-daemon` | Только rust_daemon |
| `make clean` | Очистить артефакты |

## Build Instructions

These commands build local binaries and native package artifacts for the
supported package ecosystems.

## Versioning

Project releases use semantic versioning. The current release version is
`0.0.1`.

Release tags use the `v<semver>` format, for example `v0.0.1`. Keep
`rust_ui/Cargo.toml`, RPM, pacman, and platform fallback metadata synchronized
when bumping a release.
See `docs/release-versioning.md` for the full rules.

### Debug build and run

```bash
make debug
```

### Release UI

```bash
make build-ui
```

The binary is written to:

```text
rust_ui/target/release/asus-linux-control
```

### Release daemon

```bash
make build-daemon
```

The daemon binary is written to:

```text
rust_daemon/target/release/asus-control-daemon
```

### Native package artifacts

RPM packaging requires `rpmbuild`. Pacman packaging requires `makepkg`.

Build Fedora/Bazzite RPM packages:

```bash
make package-rpm
```

Build Arch/SteamOS pacman packages:

```bash
make package-arch
```

Build every native package target:

```bash
make package-all
```

Package outputs are written under:

```text
dist/rpmbuild/RPMS/
dist/rpmbuild/SRPMS/
dist/arch/
```

### Full local verification

```bash
make test
cd rust_ui && cargo clippy --all-targets -- -D warnings
```

Run these before packaging or installing a new build.

### ROG/M4 hotkey verification

Hardware verification for launch, same-process window raising, debounce,
unrelated M-key behavior, firmware restoration, and unsupported-device fallback
is documented in [ROG/M4 hotkey manual verification](docs/research/rog-m4-hotkey-manual-verification.md).

The ASUS M-key firmware report sequence is adapted from G-Helper by seerge:
https://github.com/seerge/g-helper/pull/5691

## Hardware Control

### Performance Mode

| Mode | Value | Sysfs |
|---|---|---|
| Silent | 0 | `throttle_thermal_policy` |
| Balanced | 1 | `throttle_thermal_policy` |
| Turbo | 2 | `throttle_thermal_policy` |

Читается/пишется через asusd (`xyz.ljones.Platform.PlatformProfile`).

### TDP Control (через D-Bus daemon)

| Параметр | Sysfs | Диапазон |
|---|---|---|
| Sustained TDP | `ppt_pl1_spl` | 5–54 W |
| Boost TDP | `ppt_pl2_sppt` | 5–54 W |
| CPU Temp limit | `ppt_cpu_limit` | 60–100 °C |

Значения volatile — сбрасываются при перезагрузке. Приложение читает текущие значения из системы при старте.

### Battery Charge Limit (через D-Bus daemon)

Диапазон: 20–100%. Sysfs: `charge_control_end_threshold`.

## Project Structure

```
.
├── rust_ui/                # Desktop application (egui), binary `asus-linux-control`
│   └── src/
│       ├── main.rs         # single instance + eframe bootstrap
│       ├── app.rs          # navigation shell, alerts
│       ├── backend/        # zbus clients: asusd, supergfxd, our daemon
│       ├── features/       # per-screen logic and service traits (unit-tested)
│       ├── settings/       # settings.json + profiles/*.json
│       └── ui/             # screens, widgets, fan curve editor
│
├── rust_core/              # sysfs readers shared with the UI (fans, temperatures)
│
├── rust_daemon/            # D-Bus daemon (запускается как root)
│   └── src/
│       ├── main.rs         # D-Bus interface + zbus
│       ├── polkit.rs       # Polkit authorization
│       └── hardware/       # tdp, battery, thermal, fan, cpu_boost...
│
├── install/                # Установка daemon
│   ├── install.sh
│   ├── dbus/               # D-Bus/Polkit конфигурация
│   └── DBUS_INSTALLATION.md
│
├── packaging/              # RPM spec, PKGBUILD, desktop/metainfo
└── Makefile                # Основные команды сборки
```

Настройки хранятся в `$XDG_CONFIG_HOME/asus-linux-control/settings.json` (профили —
в `profiles/` рядом), сессия XG-диагностики — в
`$XDG_STATE_HOME/asus-linux-control/xg_mobile_fan_diagnostic.json`. Формат файлов
совпадает с прежней Flutter-версией.

## Development

### После изменений в rust_ui / rust_core

```bash
make test
make run
```

### После изменений в rust_daemon

```bash
make install          # пересобрать + переустановить
pkill -f asus-control-daemon || true   # убить старый процесс
# daemon поднимется автоматически при следующем D-Bus вызове
```

Подробнее: `install/DBUS_INSTALLATION.md` → раздел «Обновление daemon».

## 🐛 Troubleshooting

### Permission Denied Error

```bash
# Verify udev rules are installed
ls -l /etc/udev/rules.d/99-asus-control.rules

# Check if user is in plugdev group
groups $USER

# Reload udev rules manually
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### ASUS WMI Interface Not Found

```bash
# Check if asus-nb-wmi module is loaded
lsmod | grep asus

# Check if interface exists
ls -la /sys/class/platform/asus-nb-wmi/

# Try loading module manually
sudo modprobe asus-nb-wmi
```

### App Won't Start

```bash
# Запуск с подробным логом всех D-Bus вызовов
RUST_LOG=debug asus-linux-control

# Окно не появляется, но процесс уже запущен — второй запуск только поднимает
# существующее окно. Устаревший сокет удаляется автоматически:
ls -la "$XDG_RUNTIME_DIR/asus-linux-control.sock"
```

## 📋 Roadmap

### ✅ Phase 1: MVP (Current)
- Performance mode control with parameter reading
- Native egui UI with a Material-style dark theme

### 🚧 Phase 2: Monitoring (Next)
- Real-time CPU/GPU temperature monitoring
- Fan speed display
- Battery status dashboard

### 📅 Phase 3: Advanced Features
- Custom fan curve editor
- Battery charge limiting
- GPU mode control (Eco/Standard/Ultimate)
- RGB lighting control
- Anime Matrix support

## 📚 Documentation

- **Specification**: `specs/001-asus-linux-control/spec.md`
- **Implementation Plan**: `specs/001-asus-linux-control/plan.md`
- **Task Breakdown**: `specs/001-asus-linux-control/tasks.md`
- **API Contracts**: `specs/001-asus-linux-control/contracts/`
- **Quick Start Guide**: `specs/001-asus-linux-control/quickstart.md`

## 🤝 Contributing

This is a learning/demonstration project showcasing:
- Native Rust desktop UI with egui
- Testable screen logic behind service traits
- Linux sysfs hardware control
- Type-safe hardware abstraction

## 📄 License

Educational/demonstration project.

## 🙏 Acknowledgments

- **ASUS WMI Documentation**: Linux kernel documentation
- **egui / eframe**: immediate-mode UI framework
- **Reference Projects**: G-Helper, AsusCtl, Starlight
