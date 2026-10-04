# Plan: Replace the Flutter UI with a native Rust (egui) UI

Date: 2026-10-03
Branch: `rust-egui-ui`

## Goal

Replace the Flutter desktop app (`lib/`, ~12k lines of Dart, BLoC + `flutter_rust_bridge`)
with a single native Rust binary built on `egui`/`eframe`, keeping every user-visible
feature and every persisted file format unchanged. The privileged daemon
(`rust_daemon`) and its D-Bus contract are not touched.

## Decisions

| Topic | Decision | Why |
|---|---|---|
| UI toolkit | `eframe`/`egui` (glow backend, Wayland + X11) | Immediate mode keeps porting simple, custom painting for the fan curve is trivial, one static binary. |
| Crate layout | New crate `rust_ui/` producing the binary `asus-linux-control` | Same executable name, so `.desktop`, hotkey agent (`gtk-launch asus-linux-control`) and packages keep working. |
| Hardware reads | `rust_ui` depends on `rust_core` as an `rlib` | Reuses sysfs discovery (fans, temps) without FFI. |
| D-Bus | `zbus` 4 (already used by the daemon) on a background tokio runtime | Same library as the daemon, async, no `dbus-send` subprocesses. |
| State management | One controller per screen (`features/*`): plain state enum + async operations behind service traits | Mirrors the BLoC events/states 1:1, unit-testable with mock services. |
| UI ↔ backend | UI spawns async jobs on tokio, results come back over a channel and trigger `request_repaint()` | UI thread never blocks on D-Bus or polkit dialogs. |
| Settings | Same JSON files (`$XDG_CONFIG_HOME/asus-linux-control/settings.json`, `profiles/*.json`), same lenient parsing + v1→v2 migration | Existing user settings and profiles keep working. |
| Single instance | Unix socket in `$XDG_RUNTIME_DIR`; a second launch asks the first to raise its window and exits | Replaces GtkApplication uniqueness used by the ROG hotkey agent. |
| Errors | Modal "copyable alert" with selectable details and a Copy button | Same UX as `CopyableAlert`. |
| Gamepad | Keep `UiInputCommand` plumbing: Steam Input poller (still a stub in `rust_core`) is mapped to egui focus/keys; the curve editor handles the curve commands and the same keyboard shortcuts | Behaviour parity with the Flutter build; the native adapter can be plugged in later. |

## Architecture

```
rust_ui (bin: asus-linux-control)
├── main.rs            single-instance guard, eframe bootstrap
├── app.rs             shell: navigation rail, screens, alerts, job pump
├── runtime.rs         tokio runtime + job channel (Job<T> -> UI message)
├── backend/           zbus clients
│   ├── asusd.rs       xyz.ljones.Asusd: platform profile, armoury PPT, charge limit, fan curves
│   ├── daemon.rs      com.asuslinuxcontrol.Daemon: temp limit, CPU boost, fan engine, package updates
│   └── supergfx.rs    org.supergfxctl.Daemon
├── settings/          AppSettings model, file store, profile store, profiled store, migration
├── features/          one module per screen: state + async operations (ported BLoC logic)
│   performance, cpu_boost, tdp, battery, gpu, fan, updates, profiles, xg_diag
├── ui/                widgets: cards, mode buttons, slider card, fan curve editor, alert, theme
└── input.rs           UiInputCommand -> egui focus / curve editor
```

Flutter file → Rust module mapping:

| Flutter | Rust |
|---|---|
| `services/dbus_client.dart`, `bridge_service.dart` | `backend/asusd.rs`, `backend/daemon.rs`, `features/*` services |
| `services/asusd_fan_client.dart`, `services/fan/*` | `backend/asusd.rs` (fan curves), `backend/daemon.rs` (fan engine), `features/fan/model.rs` |
| `services/supergfx_client.dart`, `gpu_service.dart`, `models/gpu_models.dart` | `backend/supergfx.rs`, `features/gpu.rs` |
| `services/package_update_service.dart`, `blocs/package_updates/*` | `backend/daemon.rs`, `features/updates.rs` |
| `services/settings/*` | `settings/*` |
| `services/xg_mobile_fan_diagnostic_service.dart` | `features/xg_diag.rs` |
| `blocs/*` | `features/*` |
| `screens/*`, `widgets/*` | `ui/screens/*`, `ui/widgets/*` |
| `widgets/copyable_alert.dart` | `ui/alert.rs` |
| `widgets/input/*`, `services/input/*` | `input.rs` |
| `linux/runner` (GtkApplication activation) | `main.rs` single-instance socket |

## Steps

1. **rust_core cleanup** – drop `flutter_rust_bridge`, `frb_generated.rs`, `#[frb]` attributes and the
   unused dependencies (`libmedium`, `nvml-wrapper`, `upower_dbus`, `hidapi`, `tokio`, `zbus`, `dirs`);
   crate becomes a plain `rlib`. Existing rust_core tests must still pass.
2. **rust_ui skeleton** – Cargo crate, eframe app, dark theme, navigation rail with the 8 destinations
   (Performance, TDP, GPU, Fans, Battery, XG Diag, Updates, Profiles), tokio job runtime,
   single-instance socket, copyable alert.
3. **Settings** – port `AppSettings`/`HardwareProfile` with identical JSON shape, issue logging,
   migration, profile id sanitising, profiled store semantics. Unit tests ported from Dart tests.
4. **Backends** – zbus proxies for asusd, our daemon and supergfxd with identical method/property
   names, signatures and value mappings (PlatformProfile ↔ mode, byte charge limit, `(s(y×8)(y×8)b)` curves).
5. **Features + screens** – port each BLoC with its "restore saved desired setting on load" logic:
   performance + CPU boost (home), TDP, GPU, battery, fans (asusd curve / dynamic engine / max-speed
   test / reset, 2 s RPM refresh, curve editor with drag + keyboard), package updates (polling while
   running), profiles (create/apply/delete, reload hardware screens), XG Mobile diagnostics
   (sweeps, hypothesis tests, saved session, diagnosis rules).
6. **Build + packaging** – Makefile targets (`run`, `build-ui`, `build-artifacts`, packages),
   RPM spec and PKGBUILD install the single binary `rust_ui/target/release/asus-linux-control`;
   descriptions no longer mention Flutter. VS Code tasks updated.
7. **Remove Flutter** – `lib/`, `test/`, `linux/`, `android/`, `ios/`, `macos/`, `web/`, `windows/`,
   `pubspec.*`, `analysis_options.yaml`, `.metadata`, `*.iml`, Flutter-specific scripts/docs sections.
   The pending uncommitted Flutter edits are stashed (not lost) before removal.
8. **Verify** – `cargo build --release`, `cargo test` and `cargo clippy` for `rust_core` and `rust_ui`;
   launch the binary to check it starts and renders (hardware calls degrade to error/unsupported
   states when asusd/daemon are absent).

## Out of scope

- Changes to `rust_daemon` or the D-Bus/polkit contract.
- A real Steam Input adapter (still a stub, as in the Flutter build).
- New features or redesigned screens beyond what egui needs.

## Status (2026-10-03)

All steps 1–8 are implemented on branch `rust-egui-ui` (not committed yet).

- Toolchain: egui/eframe 0.36 need rustc ≥ 1.95 (`rust-version = "1.95"` in `rust_ui/Cargo.toml`).
- Tests: `rust_ui` 90 unit tests + 18 packaging tests, `rust_core` and `rust_daemon` suites pass;
  `cargo clippy --all-targets -D warnings` clean for `rust_ui`.
- Smoke run: release binary starts, a second launch raises the running window.
- Docs and agent rules updated; constitution amended to 2.0.0 (principle III now covers D-Bus
  process boundaries instead of FFI).
- Deviation from Flutter: screens without loaded data show a Retry placeholder instead of an
  endless spinner; the updates screen polls a running update every 2 s.
- Open follow-ups: files in `rust_ui` above the 200-line limit (constitution VI, see its
  Sync Impact Report); unused `rust_core::services::performance` still calls `dbus-send`;
  pre-existing clippy lints in `rust_core`.
- Incident: uncommitted Flutter edits were deleted during step 7; what could be recovered is in
  `recovered-flutter-edits/` (see its README).
