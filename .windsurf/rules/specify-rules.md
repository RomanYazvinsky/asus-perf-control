# flutter_application_1 Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-13

## Active Technologies
- Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend) (001-asus-linux-control)
- JSON config at `~/.config/asus-linux-control/config.json` — TDP settings added as `tdp: TdpConfig` field (001-asus-linux-control)
- Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend) + Rust: `flutter_rust_bridge` 2.12.0, `nvml-wrapper` 0.10+ (Nvidia GPU), sysfs interfaces (AMD GPU); Flutter: `flutter_bloc` ^8.1.0, `flutter_rust_bridge` ^2.0.0 (002-gpu-modes-egpu)
- JSON config at `~/.config/asus-linux-control/config.json` - GPU mode preferences added (002-gpu-modes-egpu)

- Rust 1.75+ (backend), Dart 3.0+/Flutter 3.16+ (frontend) (001-asus-linux-control)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+ (backend), Dart 3.0+/Flutter 3.16+ (frontend): Follow standard conventions

## Recent Changes
- 002-gpu-modes-egpu: Added Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend) + Rust: `flutter_rust_bridge` 2.12.0, `nvml-wrapper` 0.10+ (Nvidia GPU), sysfs interfaces (AMD GPU); Flutter: `flutter_bloc` ^8.1.0, `flutter_rust_bridge` ^2.0.0
- 001-asus-linux-control: Added Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend)

- 001-asus-linux-control: Added Rust 1.75+ (backend), Dart 3.0+/Flutter 3.16+ (frontend)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
