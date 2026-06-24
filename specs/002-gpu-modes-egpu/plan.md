# Implementation Plan: Extended GPU Modes Support

**Branch**: `002-gpu-modes-egpu` | **Date**: 2026-04-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-gpu-modes-egpu/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extended GPU modes support for ASUS Linux Control Tool to include not only basic MUX switching but also eGPU support (AsusEgpu, XG Mobile mode) based on supergfxctl implementation. The feature adds 6 GPU modes: Integrated, Hybrid, Ultimate, AsusMuxDgpu (MUX switch with reboot), AsusEgpu (external GPU), and Vfio (GPU passthrough for VMs). System must detect hardware capabilities and only show supported modes.

**CRITICAL UPDATE (2026-04-13)**: Research identified critical bug in eGPU detection algorithm. Recommended approach: use supergfxctl daemon via D-Bus instead of reimplementing PCI hotplug detection. See research.md section 3.1 for details.

## Technical Context

**Language/Version**: Rust 1.75+ (backend), Dart 3.0+ / Flutter 3.16+ (frontend)
**Primary Dependencies**: Rust: `flutter_rust_bridge` 2.12.0, `nvml-wrapper` 0.10+ (Nvidia GPU), sysfs interfaces (AMD GPU); Flutter: `flutter_bloc` ^8.1.0, `flutter_rust_bridge` ^2.0.0
**Storage**: JSON config at `~/.config/asus-linux-control/config.json` - GPU mode preferences added
**Testing**: `cargo test` (Rust unit), `flutter test` (widget tests), manual hardware validation
**Target Platform**: Linux (ASUS laptops with hybrid graphics, kernel 6.1+)
**Project Type**: Desktop application (Flutter + Rust FFI)
**Performance Goals**: GPU mode switch within 10 seconds (excluding reboot requirements), hardware detection on startup < 1 second
**Constraints**: No root access at runtime (udev rules), GPU mode changes may require logout/reboot, hardware-specific mode availability
**Scale/Scope**: ~2,500+ LOC existing, adding ~800 lines Rust + ~400 lines Dart for GPU service and UI

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Evaluation (Before Phase 0)

### Principle I: Hardware Safety First (NON-NEGOTIABLE)
- **PASS**: dGPU toggle operations will warn user of potential display disruption (FR-008)
- **PASS**: GPU mode changes will require explicit user confirmation for destructive actions (reboot required modes)
- **PASS**: Hardware detection before mode switch (FR-001, FR-002)

### Principle II: Modular Service Architecture
- **PASS**: GPU service will be in `rust_core/src/services/gpu.rs` (independent module)
- **PASS**: Service will expose controller struct with `new()` constructor and feature methods
- **PASS**: Service will not directly call other services (composition in api.rs)
- **PASS**: Unit tests in `rust_core/tests/` for GPU service

### Principle III: Type-Safe FFI Boundary
- **PASS**: All GPU control functions will use `flutter_rust_bridge` with `#[frb(sync)]` or async annotations
- **PASS**: GPU data structures will derive `Serialize` and be documented in `contracts/ffi-api.md`
- **PASS**: FFI functions returning fallible operations will use `Result<T, String>` with descriptive errors

### Principle IV: Test-Before-Hardware
- **PASS**: GPU service will have >= 80% line coverage with mocked hardware interfaces
- **PASS**: Every GPU FFI function will have integration test calling from Dart
- **PASS**: Every GPU mode switch validation will be tested with invalid inputs
- **PASS**: Manual hardware validation documented in PR

### Principle V: Least Privilege Security Model
- **PASS**: Application will not run as root
- **PASS**: GPU sysfs writes will use udev rules (already in install/99-asus-control.rules)
- **PASS**: User configuration stored in `~/.config/asus-linux-control/` (XDG spec)
- **PASS**: GPU mode switching will validate paths and numeric inputs

### Principle VI: Single Responsibility
- **PASS**: GPU service file will not exceed 200 lines
- **PASS**: GPU screen will not exceed 150 lines
- **PASS**: GPU widgets will be in separate files under `widgets/`
- **PASS**: Each GPU function will do one thing (read/write/validate)

### Principle VII: Plan-Review-Implement Discipline
- **PASS**: This plan.md exists before implementation begins
- **PASS**: Review phase will be executed before code changes
- **PASS**: Bug loop will return to plan phase before fixes

**PRE-DESIGN GATE STATUS**: ✅ PASS

---

### Post-Design Re-evaluation (After Phase 1)

### Principle I: Hardware Safety First (NON-NEGOTIABLE)
- **PASS**: research.md documents safety checks for dGPU toggle warnings
- **PASS**: data-model.md includes validation rules for temperature (0-120°C) and utilization (0-100%)
- **PASS**: contracts/ffi-api.md documents user-facing error messages for safety failures

### Principle II: Modular Service Architecture
- **PASS**: data-model.md defines independent GPU entities (GpuMode, GpuConfiguration, GpuStatus)
- **PASS**: quickstart.md shows GpuController as independent service module
- **PASS**: Project structure shows gpu.rs as separate file in services/

### Principle III: Type-Safe FFI Boundary
- **PASS**: contracts/ffi-api.md documents all FFI functions with `#[frb(sync)]` annotations
- **PASS**: data-model.md shows Rust structs with `serde::Serialize` derive
- **PASS**: contracts/ffi-api.md defines error messages as user-facing `Result<T, String>`

### Principle IV: Test-Before-Hardware
- **PASS**: quickstart.md includes Rust unit tests example
- **PASS**: quickstart.md includes Flutter widget tests example
- **PASS**: quickstart.md documents integration tests for FFI functions

### Principle V: Least Privilege Security Model
- **PASS**: quickstart.md references existing udev rules
- **PASS**: data-model.md shows config stored in XDG-compliant location
- **PASS**: research.md documents path validation for sysfs writes

### Principle VI: Single Responsibility
- **PASS**: quickstart.md shows GPU widgets in separate files (gpu_mode_selector.dart, gpu_status_card.dart)
- **PASS**: quickstart.md shows GPU service as single responsibility file
- **PASS**: Project structure separates concerns (services/, blocs/, widgets/)

### Principle VII: Plan-Review-Implement Discipline
- **PASS**: This plan.md exists and is complete before implementation
- **PASS**: All design artifacts (research.md, data-model.md, contracts/, quickstart.md) generated
- **PASS**: Agent context updated with new technology information

**POST-DESIGN GATE STATUS**: ✅ PASS - All constitution principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/002-gpu-modes-egpu/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
rust_core/
├── src/
│   ├── services/
│   │   └── gpu.rs       # NEW: GPU service module
│   ├── api.rs            # MODIFY: Add GPU FFI functions
│   └── lib.rs            # MODIFY: Export GPU service
└── tests/
    └── gpu_tests.rs      # NEW: GPU service unit tests

lib/
├── blocs/
│   └── gpu/              # NEW: GPU BLoC
│       ├── gpu_bloc.dart
│       ├── gpu_event.dart
│       └── gpu_state.dart
├── screens/
│   └── gpu_screen.dart   # MODIFY/NEW: GPU mode selector screen
├── widgets/
│   └── gpu/              # NEW: GPU-specific widgets
│       ├── gpu_mode_selector.dart
│       └── gpu_status_card.dart
└── services/
    └── gpu_service.dart  # NEW: GPU service interface

test/
└── blocs/
    └── gpu/              # NEW: GPU BLoC tests
        └── gpu_bloc_test.dart
```

**Structure Decision**: Flutter + Rust FFI desktop application (existing structure). GPU feature follows established pattern: Rust service in `rust_core/src/services/gpu.rs`, Flutter BLoC in `lib/blocs/gpu/`, UI in `lib/screens/gpu_screen.dart` with widgets in `lib/widgets/gpu/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations - this section not applicable.

## Implementation Strategy Update

### Critical Findings

**eGPU Detection Bug**: Initial implementation in `rust_core/src/services/gpu.rs:111-145` contains critical error:
- **Wrong**: Uses PCI device address (`0000:01:00.0`) as hotplug slot directory name
- **Correct**: Must iterate `/sys/bus/pci/slots/`, read each slot's `address` file, match with device
- **Impact**: XG Mobile (ASUS eGPU) will never be detected with current algorithm
- **Source**: supergfxctl reference implementation `find_slot_power()` function

### Recommended Approach

**Option A: Use supergfxctl via D-Bus (RECOMMENDED)**

**Advantages**:
- ✅ Proven XG Mobile support on ASUS Flow series
- ✅ No complex PCI enumeration to maintain
- ✅ Handles driver conflicts, logout/reboot requirements
- ✅ D-Bus signals for real-time notifications
- ✅ Active maintenance by asus-linux community
- ✅ Already uses D-Bus for asusd (TDP, battery)

**Implementation**:
1. Create `lib/services/supergfx_client.dart` with D-Bus interface
2. Expose 5 methods: `getMode()`, `getSupportedModes()`, `setMode()`, `getVendor()`, `watchModeChanges()`
3. Integrate with existing BridgeService pattern
4. Add dependency check: `systemctl status supergfxd`

**D-Bus Interface**:
- Bus: `org.supergfxctl.Daemon`
- Path: `/org/supergfxctl/Gfx`
- Methods: `Mode()`, `Supported()`, `SetMode(y)`, `Vendor()`, `Power()`

**Option B: Fix Rust implementation (LEARNING EXERCISE)**

**Required Changes**:
1. Add `find_slot_power()` helper function (see research.md:135-154)
2. Fix `is_egpu_connected()` to use proper hotplug detection (research.md:156-194)
3. Iterate slots, read `address` files, match PCI device addresses
4. Add comprehensive error handling for missing `/sys/bus/pci/slots/`

**Recommendation**: **Use Option A (supergfxctl D-Bus)** for production. Keep Option B as learning exercise/fallback.

### Modified Project Structure

**New Files** (if using supergfxctl D-Bus):
```
lib/services/
├── supergfx_client.dart    # NEW: D-Bus client for supergfxctl
└── gpu_service.dart         # MODIFY: Wrapper around supergfx_client

lib/blocs/gpu/
├── gpu_bloc.dart            # Uses SupergfxClient instead of FFI
├── gpu_event.dart
└── gpu_state.dart
```

**Modified Files**:
```
lib/services/bridge_service.dart  # Add supergfx_client integration
pubspec.yaml                      # Already has dbus: ^0.7.0
```

**Removed/Deprecated** (if using D-Bus):
```
rust_core/src/services/gpu.rs     # Keep for reference, not used in production
rust_core/tests/gpu_tests.rs      # Not needed if using supergfxctl
```

### Testing Strategy

**If using supergfxctl D-Bus**:
1. Mock D-Bus responses in Flutter tests
2. Integration test: call real supergfxd on test system
3. Manual validation on ASUS Flow with XG Mobile

**If using Rust implementation**:
1. Unit tests with mocked sysfs (complex)
2. Integration tests on real hardware only
3. Cannot test XG Mobile detection without physical device

### Migration Path

**Phase 1**: Implement supergfxctl D-Bus integration (1-2 days)
- Dart client, BLoC integration, UI
- Test on system with supergfxd installed

**Phase 2**: Fallback to Rust implementation if supergfxd unavailable (optional)
- Check `systemctl status supergfxd` on startup
- Show warning if supergfxctl not installed
- Use Rust FFI as fallback (with known XG Mobile bug)

**Phase 3**: Fix Rust implementation (learning exercise, low priority)
- Apply fixes from research.md
- Test on real hardware
- Keep as alternative backend
