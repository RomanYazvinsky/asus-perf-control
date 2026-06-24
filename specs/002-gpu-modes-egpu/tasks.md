# Tasks: Extended GPU Modes Support

**Input**: Design documents from `/specs/002-gpu-modes-egpu/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are OPTIONAL - only include them if explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume Flutter + Rust FFI structure per plan.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Dependency setup and project configuration

- [x] T001 Add nvml-wrapper 0.10+ dependency to rust_core/Cargo.toml
- [x] T002 [P] Verify flutter_rust_bridge ^2.0.0 is in pubspec.yaml
- [x] T003 [P] Verify flutter_bloc ^8.1.0 is in pubspec.yaml

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core GPU entities and FFI API that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Rust Backend - GPU Entities

- [x] T004 Create GpuMode enum in rust_core/src/services/gpu.rs with 6 variants (Integrated, Hybrid, Ultimate, AsusMuxDgpu, AsusEgpu, Vfio)
- [x] T005 [P] Create GpuVendor enum in rust_core/src/services/gpu.rs (Nvidia, Amd)
- [x] T006 [P] Create GpuPowerState enum in rust_core/src/services/gpu.rs (Active, Idle, Suspended, Unknown)
- [x] T007 Create GpuConfiguration struct in rust_core/src/services/gpu.rs (has_dgpu, dgpu_vendor, has_mux_switch, supports_egpu, external_gpu_connected, vfio_available)
- [x] T008 [P] Create GpuStatus struct in rust_core/src/services/gpu.rs (active_mode, dgpu_enabled, temperature, utilization, power_state)
- [x] T009 Create GpuController struct in rust_core/src/services/gpu.rs with new() constructor
- [x] T010 Implement detect_dgpu() method in rust_core/src/services/gpu.rs (PCI enumeration)
- [x] T011 [P] Implement detect_dgpu_vendor() method in rust_core/src/services/gpu.rs (PCI vendor ID check)
- [x] T012 [P] Implement has_mux_support() method in rust_core/src/services/gpu.rs (sysfs presence check)
- [x] T013 [P] Implement has_egpu_support() method in rust_core/src/services/gpu.rs (sysfs presence check)
- [x] T014 [P] Implement is_egpu_connected() method in rust_core/src/services/gpu.rs (PCI/Thunderbolt check)
- [x] T015 [P] Implement has_vfio_support() method in rust_core/src/services/gpu.rs (module availability check)
- [x] T016 Implement get_capabilities() method in rust_core/src/services/gpu.rs (returns GpuConfiguration)
- [x] T017 Implement read_dgpu_state() method in rust_core/src/services/gpu.rs (sysfs read from dgpu_disable)
- [x] T018 [P] Implement write_dgpu_state() method in rust_core/src/services/gpu.rs (sysfs write to dgpu_disable)
- [x] T019 Implement get_status() method in rust_core/src/services/gpu.rs (returns GpuStatus)

### Rust Backend - FFI API

- [x] T020 Add get_gpu_capabilities() FFI function to rust_core/src/api.rs with #[frb(sync)] annotation
- [x] T021 [P] Add get_gpu_status() FFI function to rust_core/src/api.rs with #[frb(sync)] annotation
- [x] T022 [P] Add set_gpu_mode() FFI function to rust_core/src/api.rs with #[frb(sync)] annotation returning Result<(), String>
- [x] T023 [P] Add get_gpu_mode() FFI function to rust_core/src/api.rs with #[frb(sync)] annotation
- [x] T024 [P] Add get_supported_gpu_modes() FFI function to rust_core/src/api.rs with #[frb(sync)] annotation
- [x] T025 Export gpu module in rust_core/src/lib.rs

### Flutter Frontend - GPU Entities

- [x] T026 Create GpuMode enum in lib/services/gpu_service.dart (6 variants matching Rust)
- [x] T027 [P] Create GpuVendor enum in lib/services/gpu_service.dart (Nvidia, Amd)
- [x] T028 [P] Create GpuPowerState enum in lib/services/gpu_service.dart (Active, Idle, Suspended, Unknown)
- [x] T029 Create GpuConfiguration data class in lib/services/gpu_service.dart
- [x] T030 [P] Create GpuStatus data class in lib/services/gpu_service.dart
- [x] T031 Create IGpuService interface in lib/services/gpu_service.dart with getCapabilities(), getStatus(), setGpuMode(), getSupportedModes() methods

### Flutter Frontend - BLoC Foundation

- [x] T032 Create GpuState classes in lib/blocs/gpu/gpu_state.dart (GpuInitial, GpuLoading, GpuLoaded, GpuUnsupported, GpuError, GpuRequiresReboot)
- [x] T033 [P] Create GpuEvent classes in lib/blocs/gpu/gpu_event.dart (LoadCapabilitiesEvent, LoadStatusEvent, SetModeRequestedEvent, ConfirmRebootEvent)
- [x] T034 Create GpuBloc in lib/blocs/gpu/gpu_bloc.dart with event handlers

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic GPU Mode Switching (Priority: P1) 🎯 MVP

**Goal**: Enable switching between Integrated, Hybrid, and Ultimate modes for battery optimization and performance tuning

**Independent Test**: Can be fully tested by switching GPU modes and verifying that discrete GPU is enabled/disabled via system power consumption and GPU detection tools

### Rust Backend - US1 Implementation

- [x] T035 [US1] Implement set_mode() method for Integrated mode in rust_core/src/services/gpu.rs (dgpu_disable=1)
- [x] T036 [US1] Implement set_mode() method for Hybrid mode in rust_core/src/services/gpu.rs (dgpu_disable=0, MUX off)
- [x] T037 [US1] Implement set_mode() method for Ultimate mode in rust_core/src/services/gpu.rs (dgpu_disable=0, dGPU primary)
- [x] T038 [US1] Add error handling for mode switch failures in rust_core/src/services/gpu.rs with user-facing messages
- [x] T039 [US1] Add config persistence for GPU mode in rust_core/src/services/gpu_config.rs (save to ~/.config/asus-linux-control/gpu_mode.json)

### Flutter Frontend - US1 Implementation

- [x] T040 [US1] Create GpuService implementation in lib/services/gpu_service.dart (BridgeService wrapper)
- [x] T041 [P] [US1] Implement _onLoadCapabilities() in lib/blocs/gpu/gpu_bloc.dart (calls IGpuService)
- [x] T042 [P] [US1] Implement _onSetModeRequested() in lib/blocs/gpu/gpu_bloc.dart with mode validation
- [x] T043 [US1] Create gpu_screen.dart in lib/screens/ with BlocProvider and BlocBuilder setup
- [x] T044 [US1] Add GPU mode warning dialog in lib/screens/gpu_screen.dart for session restart requirements (FR-008)
- [x] T045 [P] [US1] Create gpu_status_card.dart in lib/widgets/gpu/ (displays active mode, dgpu_enabled, temperature)
- [x] T046 [P] [US1] Create gpu_mode_selector.dart in lib/widgets/gpu/ (radio button list for mode selection)
- [x] T047 [US1] Add GPU screen to NavigationRail in lib/main.dart

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - ASUS MUX Switching (Priority: P2)

**Goal**: Enable MUX switch to use dGPU as primary graphics for maximum performance (requires reboot)

**Independent Test**: Can be fully tested by toggling MUX switch, rebooting, and verifying display is connected directly to dGPU via GPU detection tools and performance benchmarks

### Rust Backend - US2 Implementation

- [x] T048 [US2] Implement toggle_mux() method in rust_core/src/services/gpu.rs (writes to /sys/class/platform/asus-nb-wmi/mux)
- [x] T049 [US2] Add MUX mode support to set_mode() method in rust_core/src/services/gpu.rs (AsusMuxDgpu variant)
- [x] T050 [US2] Add MUX mode detection to get_capabilities() in rust_core/src/services/gpu.rs

### Flutter Frontend - US2 Implementation

- [x] T051 [US2] Add AsusMuxDgpu mode to GpuMode enum in lib/services/gpu_service.dart
- [x] T052 [US2] Add reboot confirmation dialog in lib/screens/gpu_screen.dart for AsusMuxDgpu mode
- [x] T053 [US2] Add MUX mode to gpu_mode_selector.dart in lib/widgets/gpu/ (hidden if has_mux_switch=false)
- [x] T054 [US2] Add reboot initiation in lib/screens/gpu_screen.dart after MUX confirmation

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - External GPU Support (Priority: P3)

**Goal**: Enable switching to external GPU mode for ASUS Flow series and XG Mobile devices

**Independent Test**: Can be fully tested by connecting an external GPU (e.g., ASUS XG Mobile), switching to "AsusEgpu" mode, and verifying external GPU is active and rendering graphics

### Rust Backend - US3 Implementation

- [x] T055 [US3] Implement enable_egpu_mode() method in rust_core/src/services/gpu.rs (writes to egpu_enable sysfs)
- [x] T056 [US3] Add eGPU connection check to enable_egpu_mode() in rust_core/src/services/gpu.rs (returns error if disconnected)
- [x] T057 [US3] Add AsusEgpu mode support to set_mode() method in rust_core/src/services/gpu.rs

### Flutter Frontend - US3 Implementation

- [x] T058 [US3] Add AsusEgpu mode to GpuMode enum in lib/services/gpu_service.dart
- [x] T059 [US3] Add eGPU disconnected error dialog in lib/screens/gpu_screen.dart
- [x] T060 [US3] Add AsusEgpu mode to gpu_mode_selector.dart in lib/widgets/gpu/ (hidden if supports_egpu=false)

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: User Story 4 - VFIO GPU Passthrough (Priority: P4)

**Goal**: Enable binding dGPU to VFIO for GPU passthrough to VMs

**Independent Test**: Can be fully tested by switching to VFIO mode, verifying dGPU is bound to vfio-pci driver, and confirming GPU is available for VM passthrough

### Rust Backend - US4 Implementation

- [x] T061 [US4] Implement has_vfio_support() check in rust_core/src/services/gpu.rs (module availability)
- [x] T062 [US4] Implement bind_dgpu_to_vfio() method in rust_core/src/services/gpu.rs (unbind from graphics, bind to vfio-pci)
- [x] T063 [US4] Implement rebind_dgpu_to_graphics() method in rust_core/src/services/gpu.rs (bind back to nvidia/amdgpu)
- [x] T064 [US4] Add Vfio mode support to set_mode() method in rust_core/src/services/gpu.rs
- [x] T065 [US4] Add vfio availability check to set_mode() in rust_core/src/services/gpu.rs (returns error if unavailable)

### Flutter Frontend - US4 Implementation

- [x] T066 [US4] Add Vfio mode to GpuMode enum in lib/services/gpu_service.dart
- [x] T067 [US4] Add vfio not available error dialog in lib/screens/gpu_screen.dart
- [x] T068 [US4] Add Vfio mode to gpu_mode_selector.dart in lib/widgets/gpu/ (hidden if vfio_available=false)

**Checkpoint**: All four user stories should now be independently functional

---

## Phase 7: GPU Monitoring (Cross-Cutting)

**Purpose**: Add GPU temperature and utilization monitoring for all modes

- [x] T069 [P] Implement NvidiaGpuMonitor in rust_core/src/services/gpu_monitor.rs using nvml-wrapper (temperature, utilization)
- [x] T070 [P] Implement AmdGpuMonitor in rust_core/src/services/gpu_monitor.rs using sysfs drm interfaces (temperature, utilization)
- [x] T071 Add GPU monitoring integration to get_status() in rust_core/src/services/gpu.rs (vendor-specific monitor selection)
- [x] T072 Add temperature display to gpu_status_card.dart in lib/widgets/gpu/
- [x] T073 Add utilization display to gpu_status_card.dart in lib/widgets/gpu/

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T074 [P] Add Rust unit tests in rust_core/tests/gpu_tests.rs (>=80% coverage with mocked hardware)
- [x] T075 [P] Add Flutter BLoC tests in test/blocs/gpu_bloc_test.dart
- [ ] T076 [P] Add FFI integration tests in test/ (call each GPU function from Dart)
- [x] T077 Run flutter_rust_bridge_codegen to update bridge_generated_bridge.dart in lib/
- [x] T078 [P] Add configuration schema validation in rust_core/src/services/gpu_config.rs (gpu_mode field validation)
- [ ] T079 [P] Add auto GPU mode switching based on AC/battery power state in rust_core/src/services/gpu.rs
- [ ] T080 Add kernel parameter override support (supergfxd.mode=) documentation in README.md
- [ ] T081 Add troubleshooting guide to README.md for common GPU switching issues
- [ ] T082 Run quickstart.md validation (build Rust, run Flutter app, test GPU mode switching)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4)
- **GPU Monitoring (Phase 7)**: Depends on US1 completion (can add to existing monitoring)
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Extends US1 modes but independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Independent hardware feature
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Independent hardware feature

### Within Each User Story

- Rust backend methods before Flutter frontend
- FFI API before Flutter service
- BLoC before UI widgets
- Core implementation before error handling
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T002, T003)
- All Foundational entity tasks marked [P] can run in parallel (T005, T006, T008, T011, T012, T013, T014, T015)
- All Foundational FFI tasks marked [P] can run in parallel (T021, T022, T023, T024)
- All Foundational Flutter entity tasks marked [P] can run in parallel (T027, T028, T030)
- US1 Flutter tasks marked [P] can run in parallel (T041, T042, T045, T046)
- GPU monitoring vendor implementations marked [P] can run in parallel (T069, T070)
- Polish test tasks marked [P] can run in parallel (T074, T075, T076)
- Polish config tasks marked [P] can run in parallel (T078, T079)

---

## Parallel Example: User Story 1

```bash
# Launch all Flutter BLoC tasks for User Story 1 together:
Task: "Implement _onLoadCapabilities() in lib/blocs/gpu/gpu_bloc.dart"
Task: "Implement _onSetModeRequested() in lib/blocs/gpu/gpu_bloc.dart"

# Launch all GPU widget tasks for User Story 1 together:
Task: "Create gpu_status_card.dart in lib/widgets/gpu/"
Task: "Create gpu_mode_selector.dart in lib/widgets/gpu/"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently (switch Integrated/Hybrid/Ultimate modes)
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add GPU Monitoring → Test across all modes
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (P1)
   - Developer B: User Story 2 (P2)
   - Developer C: User Story 3 (P3)
   - Developer D: User Story 4 (P4)
3. Stories complete and integrate independently
4. Team converges on GPU Monitoring and Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Constitution compliance: All Rust files must stay under 200 lines, all Dart files under 150 lines
- Hardware validation required per Constitution Principle IV
