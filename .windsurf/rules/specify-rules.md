# asus-linux-control Development Guidelines

Last updated: 2026-10-03

## Active Technologies
- Rust 1.95+ (all crates); desktop UI with `eframe`/`egui` 0.36, D-Bus via `zbus` 4, async via `tokio`
- JSON settings at `~/.config/asus-linux-control/settings.json`, hardware profiles in `profiles/*.json` next to it
- Privileged daemon `rust_daemon` (`com.asuslinuxcontrol.Daemon`) with polkit; asusd and supergfxd as system services

## Project Structure

```text
rust_ui/      # desktop app: backend/ (D-Bus clients), features/ (logic), settings/, ui/
rust_core/    # sysfs readers (fans, temperatures)
rust_daemon/  # privileged D-Bus daemon
packaging/    # RPM spec, PKGBUILD, desktop/metainfo
```

## Commands

make test
cd rust_ui && cargo clippy --all-targets -- -D warnings
make run

## Code Style

Rust 2021: rustfmt defaults, clippy clean with `-D warnings`.

## Recent Changes
- 2026-10-03: Flutter UI replaced by native Rust (egui) app `rust_ui`; `flutter_rust_bridge` removed
- 002-gpu-modes-egpu: GPU modes via supergfxd
- 001-asus-linux-control: initial performance/TDP/battery controls

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
