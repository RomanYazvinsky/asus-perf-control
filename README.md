# ASUS Linux Control Tool

A modern Flutter + Rust application for controlling ASUS laptop hardware on Linux.

## Features

- ✅ Performance mode control (Silent / Balanced / Turbo)
- ✅ CPU TDP limits (Sustained + Boost) — загружает текущие значения из системы при старте
- ✅ CPU temperature ceiling
- ✅ Battery charge limit (20–100%)
- ✅ D-Bus daemon с Polkit-авторизацией для всех write-операций
- ✅ Material Design 3 dark theme
- ✅ BLoC state management
- ✅ Type-safe Rust backend

## Architecture

```
Flutter App (user space)
    │ FFI (flutter_rust_bridge)
    ▼
rust_core  ──────────────────────────── performance mode (direct sysfs)
    │ D-Bus call
    ▼
asus-control-daemon (root) ←── Polkit authorization
    │ sysfs write
    ▼
/sys/devices/platform/asus-nb-wmi/
    ppt_pl1_spl          — Sustained TDP (W)
    ppt_pl2_sppt         — Boost TDP (W)
    throttle_thermal_policy — Performance mode
/sys/class/power_supply/BAT0/
    charge_control_end_threshold — Battery limit
```

TDP и battery limit требуют daemon (привилегированный доступ через Polkit).
Performance mode читается напрямую через FFI.

## Requirements

**System:**
- Linux kernel 5.4+ (5.17+ для TDP control)
- ASUS ноутбук с модулем `asus-nb-wmi`
- D-Bus + Polkit

**Dev:**
- Flutter SDK 3.16+ / Dart 3.0+
- Rust 1.75+ / Cargo

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

Собирает `rust_core` и запускает Flutter автоматически.

**Через IDE (VS Code / Windsurf):** F5 → `Flutter: Run (Auto-build Rust)`

**Вручную:**

```bash
cd rust_core && cargo build --release && cd ..
flutter run -d linux
```

> `RustLib.init()` в `main.dart` загружает `rust_core/target/release/libasus_linux_control_core.so`.
> Без сборки Rust приложение зависнет при старте.

### Все команды make

| Команда | Что делает |
|---|---|
| `make run` | Собрать rust_core + запустить Flutter |
| `make install` | Собрать rust_daemon + установить в систему |
| `make build-rust` | Только rust_core |
| `make build-daemon` | Только rust_daemon |
| `make build-flutter` | Release сборка Flutter |
| `make clean` | Очистить артефакты |

## Build Instructions

These commands build local binaries, the Flutter Linux bundle, and native
package artifacts for the supported package ecosystems.

## Versioning

Project releases use semantic versioning. The current release version is
`0.0.1`; Flutter build metadata is tracked as `0.0.1+1`.

Release tags use the `v<semver>` format, for example `v0.0.1`. Keep Flutter,
RPM, pacman, and platform fallback metadata synchronized when bumping a release.
See `docs/release-versioning.md` for the full rules.

### Debug build and run

```bash
make run
```

This builds `rust_core` in release mode and starts the Flutter Linux app.

### Release Flutter app

```bash
make build-rust
flutter build linux --release
```

The Linux bundle is written to:

```text
build/linux/x64/release/bundle/
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
flutter test
cd rust_daemon && cargo test
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

Читается/пишется напрямую через FFI (без daemon).

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
├── rust_core/              # Rust FFI library (загружается Flutter)
│   └── src/
│       ├── api.rs          # FFI entry point
│       └── services/       # performance.rs и др.
│
├── rust_daemon/            # D-Bus daemon (запускается как root)
│   └── src/
│       ├── main.rs         # D-Bus interface + zbus
│       ├── polkit.rs       # Polkit authorization
│       └── hardware/
│           ├── tdp.rs      # ppt_pl1_spl, ppt_pl2_sppt
│           ├── battery.rs  # charge_control_end_threshold
│           └── thermal.rs  # throttle_thermal_policy
│
├── lib/                    # Flutter frontend
│   ├── main.dart
│   ├── blocs/
│   │   ├── performance/    # PerformanceBloc
│   │   └── tdp/            # TdpBloc
│   ├── screens/
│   │   ├── home_screen.dart
│   │   └── tdp_screen.dart
│   └── services/
│       ├── bridge_service.dart  # FFI + D-Bus facade
│       └── dbus_client.dart     # D-Bus client
│
├── install/                # Установка daemon
│   ├── install.sh
│   ├── dbus/               # D-Bus/Polkit конфигурация
│   └── DBUS_INSTALLATION.md
│
└── Makefile                # Основные команды сборки
```

## Development

### После изменений в rust_core (FFI)

```bash
cd rust_core && cargo build --release
flutter run -d linux  # hot restart не подхватывает .so
```

Если изменился публичный API в `api.rs` — перегенерировать биндинги:

```bash
flutter_rust_bridge_codegen generate \
  --rust-input=crate::api \
  --rust-root=rust_core \
  --dart-output=lib/bridge_generated.dart
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

### App Won't Start / Зависает на старте

Если приложение зависает при запуске — скорее всего не собрана Rust библиотека.
`RustLib.init()` пытается загрузить `libasus_linux_control_core.so` и блокируется,
если файл отсутствует.

```bash
# 1. Проверьте, что .so файл существует
ls -la rust_core/target/release/libasus_linux_control_core.so

# 2. Если файла нет — соберите Rust backend
cd rust_core
cargo build --release
cd ..

# 3. Очистите Flutter кеш и перезапустите
flutter clean
flutter pub get
flutter run -d linux
```

### FFI Error: flutter_rust_bridge has not been initialized

Означает, что `await RustLib.init()` не был вызван до использования FFI функций.
Проверьте, что `main.dart` содержит:

```dart
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const AsusLinuxControlApp());
}
```

## 📋 Roadmap

### ✅ Phase 1: MVP (Current)
- Performance mode control with parameter reading
- Basic UI with Material Design 3
- BLoC state management

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
- Flutter + Rust integration via FFI
- BLoC pattern for state management
- Material Design 3 implementation
- Linux sysfs hardware control
- Type-safe hardware abstraction

## 📄 License

Educational/demonstration project.

## 🙏 Acknowledgments

- **ASUS WMI Documentation**: Linux kernel documentation
- **flutter_rust_bridge**: FFI bridge framework
- **Reference Projects**: G-Helper, AsusCtl, Starlight
