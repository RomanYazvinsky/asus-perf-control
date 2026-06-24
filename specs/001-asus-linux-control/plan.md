# Implementation Plan: ASUS Linux Control Tool

**Branch**: `001-asus-linux-control` | **Date**: 2026-04-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-asus-linux-control/spec.md`

## Summary

ASUS Linux Control Tool is a Flutter + Rust desktop application for managing ASUS laptop hardware on Linux.

**✅ CURRENT STATUS**: **3 User Stories FULLY IMPLEMENTED**
- **US1 (Performance Modes)**: ✅ Complete - via asusd D-Bus `PlatformProfile` property
- **US3 (Battery Charge Limit)**: ✅ Complete - via sysfs with udev rules
- **US7 (CPU TDP & Temperature Limits)**: ✅ Complete - TDP via asusd `AsusArmoury` properties; temp limit via our daemon (persistent, ryzenadj)

**🎯 NEXT PRIORITY**: **US2: System Health Monitoring** (FR-013–FR-020) - Real-time CPU/GPU temperatures, fan speeds, and battery status dashboard with fl_chart integration.

**Technical Architecture** (updated 2026-04-13):
- **asusd** (`xyz.ljones.Asusd`) — primary hardware backend for:
  - Performance modes (`PlatformProfile` property, group-level D-Bus auth)
  - TDP PL1/PL2 (`AsusArmoury` properties with real hardware min/max, persistent)
  - Charge mode, boot sound, dGPU/eGPU control — available for future US
- **Our daemon** (`com.asuslinuxcontrol.Daemon`) — temp-limit only:
  - `GetTempLimit` / `SetTempLimit` — persistent via file + applied via ryzenadj
  - Polkit authorization for writes, auto-applied on boot
- **Direct sysfs** (with udev rules) — for battery charge limit:
  - `/sys/class/power_supply/BAT0/charge_control_end_threshold` — writable via FFI
  - Used for US3 (Battery Charge Limit)
- **Flutter** → `AsusdClient` (D-Bus Properties) + `DbusClient` (our daemon) + Direct FFI (sysfs) → `BridgeService` → BLoC → UI

## Technical Context

**Language/Version**: Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend)
**Primary Dependencies**:
- Rust: `flutter_rust_bridge` 2.12.0 (already in use), `std::fs` (no new crates needed for TDP)
- Flutter: `flutter_bloc` ^8.1.0, `flutter_rust_bridge` ^2.0.0 (already in use)

**Storage**: JSON config at `~/.config/asus-linux-control/config.json` — TDP settings added as `tdp: TdpConfig` field
**Testing**: `cargo test` (Rust unit), `flutter test` (widget tests)
**Target Platform**: Linux x86_64, kernel 5.17+ for `ppt_pl1_spl` / `ppt_pl2_sppt` / `ppt_cpu_limit` sysfs attributes
**Project Type**: Desktop application (native Linux GUI), existing codebase extension
**Performance Goals**: TDP change applied within 1 second; feature gracefully disabled if sysfs attributes absent
**Constraints**:
- Offline-first, no network
- Requires udev rules (already installed via `install/install.sh`)
- Kernel 5.17+ for PPT attributes; `is_tdp_supported()` gates the feature for older kernels
- TDP settings are volatile (reset on reboot) — re-apply via config on startup is in scope

**Scale/Scope**:
- 1 new Rust service module (`rust_core/src/services/tdp.rs`)
- 3 new FFI functions in `rust_core/src/api.rs`
- 1 new Flutter screen (`lib/screens/tdp_screen.dart`)
- 1 new BLoC (`lib/blocs/tdp/`)
- NavigationRail updated: 7 → 8 screens
- Estimated ~300 LOC Rust, ~400 LOC Dart

## Constitution Check

*GATE: Must pass before implementation. Re-check after design.*

**Status**: ✅ **No violations** — all 5 principles satisfied:

- ✅ **I. Hardware Safety First**: TDP validation enforces range 5–54W; CPU temp limit range 60–100°C; `validate_tdp()` rejects out-of-range values before any sysfs write; `is_tdp_supported()` check prevents writes on unsupported hardware
- ✅ **II. Modular Service Architecture**: New `rust_core/src/services/tdp.rs` module is independent, testable in isolation, registered in `services/mod.rs`
- ✅ **III. Type-Safe FFI Boundary**: New functions use `#[flutter_rust_bridge::frb(sync)]` with `Result<T, String>`; `TdpConfig` struct derives `Serialize`; codegen re-run after API change
- ✅ **IV. Test-Before-Hardware**: Unit tests for validation (valid/invalid ranges) before hardware integration; integration test calling FFI `get_tdp_config` from Dart
- ✅ **V. Least Privilege**: Writes to `/sys/devices/platform/asus-nb-wmi/ppt_*` granted via existing udev rules (same `asus-nb-wmi` subsystem, no new rules needed); no root required at runtime

## Project Structure

### Documentation (this feature)

```text
specs/001-asus-linux-control/
├── plan.md              # This file (updated 2026-04-09)
├── research.md          # Phase 0 — complete (2026-04-08)
├── data-model.md        # Phase 1 — needs TdpConfig entity added
├── quickstart.md        # Phase 1 — complete (2026-04-08)
├── contracts/
│   └── ffi-api.md       # Needs 3 new TDP functions documented
└── tasks.md             # Needs Phase 12 (TDP) + Phase 2 stubs added
```

### Source Code (new/modified files)

```text
rust_core/
├── src/
│   ├── api.rs                          # ADD: get_tdp_config, set_tdp_config, is_tdp_supported
│   ├── lib.rs                          # (unchanged)
│   └── services/
│       ├── mod.rs                      # ADD: pub mod tdp;
│       └── tdp.rs                      # NEW: TdpController

lib/
├── main.dart                           # UPDATE: add TDP screen to NavigationRail
├── bridge_generated.dart/
│   ├── api.dart                        # REGEN: new TDP FFI functions
│   └── frb_generated.dart              # REGEN: new bindings
├── screens/
│   └── tdp_screen.dart                 # NEW: TDP control screen
└── blocs/
    └── tdp/
        ├── tdp_bloc.dart               # NEW
        ├── tdp_event.dart              # NEW
        └── tdp_state.dart              # NEW
```

**Structure Decision**: Flutter + Rust hybrid desktop application. Follows identical pattern already established for US1 (performance bloc, performance screen, performance service). No new architectural patterns required.

## Implementation Status Overview

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: Setup | ✅ Complete | Project structure, dependencies, FFI bridge |
| Phase 2: Foundational | ✅ Complete | Data structures, service stubs, NavigationRail |
| Phase 3: US7 TDP | ✅ Complete | Full implementation with D-Bus daemon |
| Phase 4: US1 Performance | ✅ Complete | Direct FFI, sysfs control, BLoC pattern |
| Phase 5: US2 Monitoring | 🎯 Next | Real-time sensors with fl_chart |
| Phase 6: US3 Battery | ✅ Complete | Charge limit control via sysfs |
| Phase 7-10: Other US | ⏳ Pending | GPU, Fans, RGB, Display |
| Phase 11: Automation | ⏳ Pending | AC/battery profiles, auto-switching |
| Phase 12: Polish | 🔄 In Progress | README, docs, testing |

---

## Implementation Phases (Original)

*Note: Phases A-D below document the TDP implementation which is now ✅ COMPLETE*

### Phase A — Rust Backend (✅ COMPLETE)

**Status**: Fully implemented and tested

#### A1. TDP Service Module

Create `rust_core/src/services/tdp.rs`:

```rust
const PPT_PL1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl";
const PPT_PL2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt";
const PPT_CPU_TEMP_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit";

pub struct TdpController;

impl TdpController {
    pub fn is_supported() -> bool { Path::new(PPT_PL1_PATH).exists() }
    pub fn get_sustained_tdp() -> Result<u32, String>  // reads mW, returns W
    pub fn get_boost_tdp() -> Result<u32, String>
    pub fn get_cpu_temp_limit() -> Result<u32, String>
    pub fn set_sustained_tdp(watts: u32) -> Result<(), String>  // validates 5–54W
    pub fn set_boost_tdp(watts: u32) -> Result<(), String>
    pub fn set_cpu_temp_limit(celsius: u32) -> Result<(), String>  // validates 60–100°C
}
```

**Safety validations (constitution §I)**:
- `watts < 5 || watts > 54` → `Err("TDP must be 5–54W")`
- `celsius < 60 || celsius > 100` → `Err("Temp limit must be 60–100°C")`
- sysfs writes use `mw = watts * 1000` (interface expects milliwatts)

#### A2. Register Module

In `rust_core/src/services/mod.rs`:
```rust
pub mod tdp;
```

In `rust_core/src/api.rs` — add `TdpConfig` struct + 3 FFI functions:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TdpConfig {
    pub sustained_tdp_w: Option<i32>,
    pub boost_tdp_w: Option<i32>,
    pub cpu_temp_limit_c: Option<i32>,
    pub is_supported: bool,
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_tdp_config() -> TdpConfig { ... }

#[flutter_rust_bridge::frb(sync)]
pub fn set_tdp_config(sustained_w: i32, boost_w: i32, temp_limit_c: i32) -> Result<(), String> { ... }

#[flutter_rust_bridge::frb(sync)]
pub fn is_tdp_supported() -> bool { TdpController::is_supported() }
```

#### A3. Unit Tests

In `rust_core/tests/unit/tdp_tests.rs` (per constitution §IV):
- `test_validate_tdp_too_low()` — expects Err for watts < 5
- `test_validate_tdp_too_high()` — expects Err for watts > 54
- `test_validate_temp_too_low()` — expects Err for celsius < 60
- `test_validate_temp_too_high()` — expects Err for celsius > 100
- `test_validate_tdp_valid_range()` — expects Ok for watts in 5–54
- `test_is_supported_returns_bool()` — no panic

#### A4. Regenerate FFI Bindings

```bash
# After api.rs changes:
flutter_rust_bridge_codegen \
  --rust-input rust_core/src/api.rs \
  --dart-output lib/bridge_generated.dart
cargo build --release
```

---

### Phase B — Flutter Frontend (✅ COMPLETE)

**Status**: TDP screen fully implemented with BLoC pattern, sliders, and error handling

#### B1. TdpBloc

`lib/blocs/tdp/tdp_event.dart`:
```dart
abstract class TdpEvent {}
class LoadTdpConfig extends TdpEvent {}
class SetTdpRequested extends TdpEvent {
  final int sustainedW, boostW, tempLimitC;
}
class ResetTdpToDefaults extends TdpEvent {}
```

`lib/blocs/tdp/tdp_state.dart`:
```dart
abstract class TdpState {}
class TdpInitial extends TdpState {}
class TdpLoading extends TdpState {}
class TdpLoaded extends TdpState { final TdpConfig config; }
class TdpUnsupported extends TdpState {}
class TdpError extends TdpState { final String message; }
```

`lib/blocs/tdp/tdp_bloc.dart` — handles LoadTdpConfig → calls `ffi.getTdpConfig()`, SetTdpRequested → calls `ffi.setTdpConfig(...)`.

#### B2. TDP Screen

`lib/screens/tdp_screen.dart`:
- Show `TdpUnsupported` message if `!config.isSupported`
- Three `Slider` widgets: Sustained TDP (5–54W), Boost TDP (5–54W), CPU Temp Limit (60–100°C)
- Constraint: Boost TDP >= Sustained TDP (UI validation)
- "Apply" button → dispatches `SetTdpRequested`
- "Reset to Mode Defaults" button → dispatches `ResetTdpToDefaults`
- `BlocBuilder<TdpBloc, TdpState>` for reactive updates
- Error card (same style as PerformanceBloc errors in home_screen.dart)

#### B3. NavigationRail Integration

In `lib/main.dart` (or `home_screen.dart` where NavigationRail lives):
- Add 8th destination: `NavigationRailDestination(icon: Icon(Icons.power_settings_new), label: Text('TDP'))`
- Add `TdpScreen()` to screens list
- Wrap with `BlocProvider<TdpBloc>`

#### B4. Widget Tests

`test/screens/tdp_screen_test.dart`:
- Renders `TdpUnsupported` when state is `TdpUnsupported`
- Renders sliders when state is `TdpLoaded`
- Apply button dispatches `SetTdpRequested`
- Uses `RustLib.initMock(api: MockRustLibApi())`

---

### Phase C — Documentation Fixes (✅ COMPLETE)

**Status**: tasks.md and data-model.md updated with TDP entities and task tracking

Fix issues C1, C2, I1, I3, I6 identified in analysis:

1. **tasks.md Phase 2**: Add `T022b` (tdp service stub) and `T033b` (tdp_screen.dart stub)
2. **tasks.md**: Add **Phase 12** with TDP tasks (T162–T174)
3. **tasks.md**: Fix FR-number references (Phase 10 header, Notes section)
4. **data-model.md**: Add `TdpConfig` entity
5. **contracts/ffi-api.md**: Document 3 new TDP FFI functions
6. **tasks.md T160**: Update "7 user stories" → "8 user stories"

---

### Phase D — Config Persistence (✅ COMPLETE)

**Status**: TDP config persistence integrated with AppConfig

- Add `tdp: Option<TdpConfig>` field to `AppConfig` struct in `rust_core/src/config.rs`
- On app startup in `api.rs`: if `tdp` config present and hardware supported → re-apply (volatile TDP resets on reboot)
- Add `save_tdp_config()` call after successful `set_tdp_config()`

---

## Dependency Order

```
Phase A (Rust backend)
  └─→ Phase B (Flutter frontend)  [parallel with Phase D after A1+A2 done]
  └─→ Phase D (Config persistence)

Phase C (tasks.md / docs fixes) — independent, can run at any time
```

## Complexity Tracking

> No constitution violations — no entries required.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| — | — | — |

---

## 🎯 Next Steps & Roadmap

### Immediate (Ready to Implement)

#### US2: System Health Monitoring (🎯 Next Priority)
Real-time CPU/GPU temperatures, fan speeds, battery status dashboard.

**Key Components**:
- Rust: `libmedium` integration for hwmon sensor reading
- FFI: `get_sensor_readings()` streaming API
- Flutter: `MonitoringBloc` + fl_chart real-time charts
- Screen: `lib/screens/monitoring_screen.dart` with sensor cards

**Files to Create/Modify**:
- `rust_core/src/services/sensors.rs` (full implementation)
- `lib/blocs/monitoring/` (new BLoC)
- `lib/screens/monitoring_screen.dart` (full implementation)
- `lib/widgets/sensor_card.dart`, `real_time_chart.dart` (new widgets)

**Estimated**: 8-12 hours

### Short-term (Backlog)

| Priority | User Story | Description | Est. Hours |
|----------|------------|-------------|------------|
| P3 | US3 | Battery charge limiting (via asusd `ChargeControlEndThreshold`) | 3-4 |
| P4 | US4 | GPU mode control (Eco/Standard/Ultimate) | 6-8 |
| P5 | US5 | Fan curve editor with fl_chart | 12-16 |
| P6 | US6 | RGB lighting control | 8-10 |
| P8 | US8 | Display settings (refresh rate, overdrive) | 4-6 |

### Architecture Decisions (Updated 2026-04-13)

1. **asusd as primary backend**: All hardware with kernel armoury class support goes through asusd D-Bus Properties interface (`xyz.ljones.Asusd`). Group-level D-Bus auth (wheel/users), no polkit needed. Persistent across reboots.
2. **Our daemon for temp-limit only**: `com.asuslinuxcontrol.Daemon` handles persistent CPU temperature limits via file + ryzenadj. This is the only function asusd doesn't provide.
3. **No direct sysfs from Flutter**: All privileged writes go through D-Bus (asusd or our daemon). Direct FFI sysfs calls removed.
4. **asusd capabilities available on this hardware**:
   - `PlatformProfile` — performance modes (✅ in use)
   - `ppt_pl1_spl`, `ppt_pl2_sppt` — TDP with real min/max (✅ in use)
   - `ChargeControlEndThreshold` — battery charge limit (🎯 planned for US3)
   - `charge_mode` — charge mode 0/1/2 (available)
   - `boot_sound` — boot sound toggle (available)
   - `dgpu_disable`, `egpu_enable`, `egpu_connected` — GPU control (available for US4)

### Current Project Statistics

- **Lines of Code**: ~2,500+ (Rust: ~800, Dart: ~1,700)
- **Test Coverage**: Unit tests for TDP validation, widget tests for screens
- **Documentation**: README, FFI guide, DBus setup guide, MVP summary
- **CI/CD**: Makefile with build, install, clean targets
