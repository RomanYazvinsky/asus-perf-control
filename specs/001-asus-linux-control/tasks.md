# Tasks: ASUS Linux Control Tool

**Input**: Design documents from `/specs/001-asus-linux-control/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/ffi-api.md, research.md

**Tests**: Not explicitly requested in specification - focusing on implementation tasks only

**Organization**: Tasks are grouped by user story (P1-P7) to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Project uses hybrid Rust + Flutter structure:
- **Rust backend**: `rust_core/src/`
- **Flutter frontend**: `lib/`
- **Installation scripts**: `install/`
- **Contracts**: `specs/001-asus-linux-control/contracts/`

---

## Phase 1: Setup (Shared Infrastructure) ✅ COMPLETE

**Purpose**: Project initialization and basic structure

- [x] T001 Create Rust workspace at rust_core/ with Cargo.toml configuration
- [x] T002 [P] Create Flutter project structure at lib/ with pubspec.yaml
- [x] T003 [P] Configure Rust dependencies in rust_core/Cargo.toml (libmedium, nvml-wrapper, upower_dbus, hidapi, serde, tokio, flutter_rust_bridge)
- [x] T004 [P] Configure Flutter dependencies in pubspec.yaml (flutter_bloc, bloc, fl_chart, flutter_rust_bridge)
- [x] T005 [P] Create project directory structure per plan.md (services/, blocs/, screens/, widgets/)
- [x] T006 [P] Setup Material Design 3 theme in lib/main.dart with dark theme only
- [x] T007 [P] Create installation directory at install/ with placeholder files

---

## Phase 2: Foundational (Blocking Prerequisites) ✅ COMPLETE

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**✅ STATUS**: Foundation complete - FFI bridge, data structures, and service architecture ready

### FFI Bridge Setup

- [x] T008 Create FFI API entry point in rust_core/src/api.rs with flutter_rust_bridge annotations
- [x] T009 Create Rust library root in rust_core/src/lib.rs with module exports
- [x] T010 Generate Flutter bindings by running flutter_rust_bridge_codegen (creates lib/bridge_generated.dart/)
- [x] T011 Create bridge service wrapper in lib/services/bridge_service.dart

### Core Data Structures (Rust)

- [x] T012 [P] Define SensorReading struct in rust_core/src/api.rs with serde derive
- [x] T013 [P] Define PerformanceProfile struct in rust_core/src/api.rs
- [x] T014 [P] Define FanCurve struct with validation in rust_core/src/api.rs
- [x] T015 [P] Define BatteryStatus struct in rust_core/src/api.rs
- [x] T169 [P] Define TdpConfig struct in rust_core/src/api.rs with Serialize derive

### Service Module Structure (Rust)

- [x] T016 [P] Create service module root at rust_core/src/services/mod.rs
- [x] T017 [P] Create performance service at rust_core/src/services/performance.rs (FULL IMPLEMENTATION)
- [x] T018 [P] Create sensors service stub at rust_core/src/services/sensors.rs
- [x] T019 [P] Create battery service stub at rust_core/src/services/battery.rs
- [x] T020 [P] Create gpu service stub at rust_core/src/services/gpu.rs
- [x] T021 [P] Create fans service stub at rust_core/src/services/fans.rs
- [x] T022 [P] Create rgb service stub at rust_core/src/services/rgb.rs
- [x] T022b [P] Create tdp service at rust_core/src/services/tdp.rs (FULL IMPLEMENTATION)

### Configuration & Error Handling

- [x] T023 [P] Implement configuration loader in rust_core/src/config.rs with JSON serde
- [x] T024 [P] Create error types in rust_core/src/error.rs with user-friendly messages
- [x] T025 [P] Implement XDG config directory resolution in rust_core/src/config.rs (~/.config/asus-linux-control/)

### Flutter Navigation & Layout

- [x] T026 Create main app scaffold in lib/main.dart with NavigationRail (Performance + TDP screens)
- [x] T027 [P] Create screen: lib/screens/home_screen.dart (Performance dashboard with BLoC)
- [x] T028 [P] Create screen stub: lib/screens/monitoring_screen.dart
- [x] T029 [P] Create screen stub: lib/screens/battery_screen.dart
- [x] T030 [P] Create screen stub: lib/screens/gpu_screen.dart
- [x] T031 [P] Create screen stub: lib/screens/fans_screen.dart
- [x] T032 [P] Create screen stub: lib/screens/rgb_screen.dart
- [x] T033 [P] Create screen stub: lib/screens/settings_screen.dart
- [x] T033b [P] Create screen: lib/screens/tdp_screen.dart (FULL IMPLEMENTATION with BLoC)

**Checkpoint**: ✅ Foundation ready - FFI bridge operational, 2 user stories fully implemented

---

## Phase 3: User Story 7 - Control CPU TDP and Temperature Limits (Priority: P7) ✅ COMPLETE

**Goal**: Enable users to set Sustained TDP, Boost TDP, and CPU thermal throttle temperature ceiling via ASUS WMI sysfs PPT attributes

**✅ STATUS**: FULLY IMPLEMENTED - Uses D-Bus daemon with polkit authorization for privileged sysfs access

**Independent Test**: Set TDP values, run `stress-ng --cpu 4`, verify CPU package power via `turbostat` stays within configured limits

### Rust Backend Implementation

- [x] T162 [US7] Implement TdpController struct in rust_core/src/services/tdp.rs with is_supported() check
- [x] T163 [US7] Implement get_sustained_tdp() reading ppt_pl1_spl in milliwatts, returning watts
- [x] T164 [US7] Implement get_boost_tdp() reading ppt_pl2_sppt in milliwatts, returning watts
- [x] T165 [US7] Implement get_cpu_temp_limit() reading ppt_cpu_limit in degrees Celsius
- [x] T166 [US7] Implement set_sustained_tdp() with validation (5-54W) and milliwatt conversion
- [x] T167 [US7] Implement set_boost_tdp() with validation (5-54W, >= sustained) and milliwatt conversion
- [x] T168 [US7] Implement set_cpu_temp_limit() with validation (60-100°C)
- [x] T169 [US7] Add TdpConfig struct to rust_core/src/api.rs with Serialize derive
- [x] T170 [US7] Implement get_tdp_config() FFI function in rust_core/src/api.rs
- [x] T171 [US7] Implement set_tdp_config() FFI function in rust_core/src/api.rs
- [x] T172 [US7] Implement is_tdp_supported() FFI function in rust_core/src/api.rs
- [x] T173 [US7] Add tdp: Option<TdpConfig> field to AppConfig in rust_core/src/config.rs for persistence
- [x] T174 [US7] Write unit tests in rust_core/tests/tdp_tests.rs (valid/invalid range validation)

### Flutter Frontend Implementation

- [x] T175 [P] [US7] Create TdpBloc in lib/blocs/tdp/tdp_bloc.dart
- [x] T176 [P] [US7] Define TdpEvent (LoadTdpConfig, SetTdpRequested, ResetTdpToDefaults) in lib/blocs/tdp/tdp_event.dart
- [x] T177 [P] [US7] Define TdpState (TdpInitial, TdpLoading, TdpLoaded, TdpUnsupported, TdpError) in lib/blocs/tdp/tdp_state.dart
- [x] T178 [US7] Implement TDP screen in lib/screens/tdp_screen.dart with three Slider widgets (Sustained, Boost, Temp Limit)
- [x] T179 [US7] Add TdpUnsupported state UI (informational card shown when hardware not supported)
- [x] T180 [US7] Add Apply and Reset buttons dispatching TdpEvent to TdpBloc
- [x] T181 [US7] Integrate TdpBloc into NavigationRail as 2nd destination in lib/main.dart with BlocProvider
- [x] T182 [US7] Add widget tests in test/screens/tdp_screen_test.dart (unsupported state, loaded state, Apply button)

### D-Bus Daemon (System Service with Polkit)

- [x] T183 [US7] Create D-Bus daemon rust_daemon/ for privileged TDP control
- [x] T184 [US7] Implement TdpController in rust_daemon/src/hardware/tdp.rs
- [x] T185 [US7] Create polkit policy for authorization (install/dbus/com.asuslinuxcontrol.Daemon.policy)
- [x] T186 [US7] Create D-Bus service configuration (install/dbus/com.asuslinuxcontrol.Daemon.conf)
- [x] T187 [US7] Create DbusClient in lib/services/dbus_client.dart for Flutter → Daemon communication
- [x] T188 [US7] Update BridgeService to use D-Bus for TDP operations
- [x] T189 [US7] Create install scripts (install/install.sh, Makefile targets)

**Checkpoint**: ✅ User Story 7 complete - TDP control functional end-to-end via D-Bus daemon

---

## Phase 4: User Story 1 - Control Performance Modes (Priority: P1) ✅ COMPLETE

**Goal**: Enable users to switch between Silent, Balanced, and Turbo performance modes via GUI

**✅ STATUS**: FULLY IMPLEMENTED - Direct FFI calls to Rust backend, works via sysfs with udev rules

**Independent Test**: Launch app, select different performance modes, verify system thermal policy changes and persists across restarts

### Rust Backend Implementation

- [x] T034 [US1] Implement PerformanceController::new() in rust_core/src/services/performance.rs
- [x] T035 [US1] Implement read_thermal_policy() function to read /sys/class/platform/asus-nb-wmi/throttle_thermal_policy
- [x] T036 [US1] Implement write_thermal_policy() with validation (0-2) and permission error handling
- [x] T037 [US1] Implement set_performance_mode() FFI function in rust_core/src/api.rs
- [x] T038 [US1] Implement get_performance_mode() FFI function in rust_core/src/api.rs
- [x] T039 [US1] Add configuration persistence for selected mode in rust_core/src/config.rs

### Flutter Frontend Implementation

- [x] T040 [P] [US1] Create PerformanceBloc in lib/blocs/performance/performance_bloc.dart
- [x] T041 [P] [US1] Define PerformanceEvent (SetModeRequested, LoadCurrentMode) in lib/blocs/performance/performance_event.dart
- [x] T042 [P] [US1] Define PerformanceState (Loading, Loaded, Error) in lib/blocs/performance/performance_state.dart
- [x] T043 [US1] Implement mode selector widget in lib/widgets/mode_selector.dart with 3 buttons (Silent/Balanced/Turbo)
- [x] T044 [US1] Integrate PerformanceBloc into lib/screens/home_screen.dart with BlocBuilder
- [x] T045 [US1] Add permission error handling UI with actionable error messages (CopyableAlert widget)
- [x] T046 [US1] Implement mode persistence check on app startup in lib/main.dart

### Installation & Permissions

- [x] T047 [P] [US1] Create udev rules in install/99-asus-control.rules for /sys/class/platform/asus-nb-wmi access
- [x] T048 [P] [US1] Create installation script in install/install.sh for udev rules deployment
- [x] T049 [US1] Add permission validation function in rust_core/src/services/performance.rs

**Checkpoint**: ✅ User Story 1 complete - performance mode switching functional end-to-end

---

## Phase 5: User Story 2 - Monitor System Health (Priority: P2)

**Goal**: Display real-time CPU/GPU temperatures, fan speeds, and battery status in dashboard

**Independent Test**: Open dashboard, verify sensor readings update every 1 second with accurate values

### Rust Backend Implementation

- [ ] T050 [US2] Implement SensorMonitor::new() in rust_core/src/services/sensors.rs with libmedium initialization
- [ ] T051 [US2] Implement read_cpu_temperature() using hwmon sysfs interfaces
- [ ] T052 [US2] Implement read_gpu_temperature() with NVML for Nvidia and sysfs for AMD
- [ ] T053 [US2] Implement read_fan_speeds() to detect and read all hwmon fan inputs
- [ ] T054 [US2] Implement read_battery_status() using UPower D-Bus
- [ ] T055 [US2] Implement get_sensor_readings() FFI function in rust_core/src/api.rs returning SensorReading struct
- [ ] T056 [US2] Add sensor availability detection with graceful fallbacks for missing hardware

### Flutter Frontend Implementation

- [ ] T057 [P] [US2] Create MonitoringBloc in lib/blocs/monitoring/monitoring_bloc.dart with periodic refresh
- [ ] T058 [P] [US2] Define MonitoringEvent (StartMonitoring, StopMonitoring, RefreshSensors) in lib/blocs/monitoring/monitoring_event.dart
- [ ] T059 [P] [US2] Define MonitoringState (Monitoring, SensorData, Error) in lib/blocs/monitoring/monitoring_state.dart
- [ ] T060 [US2] Implement sensor card widget in lib/widgets/sensor_card.dart with temperature/RPM display
- [ ] T061 [US2] Create real-time line chart widget in lib/widgets/real_time_chart.dart using fl_chart
- [ ] T062 [US2] Implement monitoring dashboard in lib/screens/monitoring_screen.dart with multiple sensor cards
- [ ] T063 [US2] Add periodic timer (1Hz) to MonitoringBloc for sensor refresh
- [ ] T064 [US2] Implement error states for unavailable sensors (display "N/A")

**Checkpoint**: User Story 2 complete - real-time monitoring dashboard functional

---

## Phase 6: User Story 3 - Set Battery Charge Limit (Priority: P3) ✅ COMPLETE

**Goal**: Allow users to set battery charge threshold (0-100%) to extend battery lifespan

**✅ STATUS**: FULLY IMPLEMENTED - Direct FFI calls to Rust backend, works via sysfs with udev rules

**Independent Test**: Set charge limit to 80%, plug in laptop, verify battery stops charging at threshold

### Rust Backend Implementation

- [x] T065 [US3] Implement BatteryController::new() in rust_core/src/services/battery.rs
- [x] T066 [US3] Implement read_charge_limit() from /sys/class/power_supply/BAT0/charge_control_end_threshold
- [x] T067 [US3] Implement write_charge_limit() with validation (0-100) and safety checks
- [x] T068 [US3] Implement set_battery_charge_limit() FFI function in rust_core/src/api.rs
- [x] T069 [US3] Implement get_battery_charge_limit() FFI function in rust_core/src/api.rs
- [x] T070 [US3] Add charge limit persistence to configuration in rust_core/src/config.rs

### Flutter Frontend Implementation

- [x] T071 [P] [US3] Create BatteryBloc in lib/blocs/battery/battery_bloc.dart
- [x] T072 [P] [US3] Define BatteryEvent (SetLimitRequested, LoadCurrentLimit) in lib/blocs/battery/battery_event.dart
- [x] T073 [P] [US3] Define BatteryState (Loading, Loaded, Error) in lib/blocs/battery/battery_state.dart
- [x] T074 [US3] Create charge limit slider widget in lib/screens/battery_screen.dart (0-100%)
- [x] T075 [US3] Add input validation UI with error messages for invalid ranges
- [x] T076 [US3] Display current battery status and charge percentage in battery screen
- [x] T077 [US3] Add confirmation dialog for charge limit changes

**Checkpoint**: ✅ User Story 3 complete - battery charge limiting functional

---

## Phase 7: User Story 4 - Manage GPU Power Modes (Priority: P4)

**Goal**: Switch between Eco (iGPU only), Standard (hybrid), and Ultimate (dGPU primary) GPU modes

**Independent Test**: Switch GPU modes, verify discrete GPU enabled/disabled state via power consumption

### Rust Backend Implementation

- [ ] T078 [US4] Implement GpuController::new() in rust_core/src/services/gpu.rs with hardware detection
- [ ] T079 [US4] Implement read_gpu_mode() from /sys/class/platform/asus-nb-wmi/dgpu_disable
- [ ] T080 [US4] Implement write_gpu_mode() with session restart warning logic
- [ ] T081 [US4] Implement detect_nvidia_gpu() using nvml-wrapper initialization
- [ ] T082 [US4] Implement detect_amd_gpu() via sysfs drm interfaces
- [ ] T083 [US4] Implement set_gpu_mode() FFI function in rust_core/src/api.rs
- [ ] T084 [US4] Implement get_gpu_mode() FFI function in rust_core/src/api.rs
- [ ] T085 [US4] Add GPU power statistics reading for Nvidia via NVML

### Flutter Frontend Implementation

- [ ] T086 [P] [US4] Create GpuBloc in lib/blocs/gpu/gpu_bloc.dart
- [ ] T087 [P] [US4] Define GpuEvent (SetModeRequested, LoadCurrentMode) in lib/blocs/gpu/gpu_event.dart
- [ ] T088 [P] [US4] Define GpuState (Loading, Loaded, RequiresRestart, Error) in lib/blocs/gpu/gpu_state.dart
- [ ] T089 [US4] Create GPU mode selector in lib/screens/gpu_screen.dart (Eco/Standard/Ultimate)
- [ ] T090 [US4] Add session restart warning dialog when mode change requires restart
- [ ] T091 [US4] Display GPU detection status (Nvidia/AMD/None) in GPU screen
- [ ] T092 [US4] Show GPU utilization and temperature if available

**Checkpoint**: User Story 4 complete - GPU power mode management functional

---

## Phase 8: User Story 5 - Customize Fan Curves (Priority: P5)

**Goal**: Create custom fan curves with temperature-to-PWM mappings via interactive editor

**Independent Test**: Create custom curve, apply it, measure actual fan speeds match configured curve

### Rust Backend Implementation

- [ ] T093 [US5] Implement FanController::new() in rust_core/src/services/fans.rs with hwmon PWM detection
- [ ] T094 [US5] Implement read_fan_curve() from hwmon PWM auto-point interfaces
- [ ] T095 [US5] Implement write_fan_curve() with safety validation (>= 30% PWM at 80°C+)
- [ ] T096 [US5] Implement validate_fan_curve() with all safety checks from constitution
- [ ] T097 [US5] Create preset fan curves (Silent, Balanced, Performance) in rust_core/src/services/fans.rs
- [ ] T098 [US5] Implement set_fan_curve() FFI function in rust_core/src/api.rs
- [ ] T099 [US5] Implement get_fan_curve() FFI function in rust_core/src/api.rs
- [ ] T100 [US5] Add fan curve persistence to configuration storage

### Flutter Frontend Implementation

- [ ] T101 [P] [US5] Create FansBloc in lib/blocs/fans/fans_bloc.dart
- [ ] T102 [P] [US5] Define FansEvent (SetCurveRequested, LoadCurve, SelectPreset) in lib/blocs/fans/fans_event.dart
- [ ] T103 [P] [US5] Define FansState (Loading, Loaded, ValidationError) in lib/blocs/fans/fans_state.dart
- [ ] T104 [US5] Create interactive scatter plot editor in lib/widgets/fan_curve_editor.dart using fl_chart
- [ ] T105 [US5] Implement drag-and-drop point manipulation in fan curve editor
- [ ] T106 [US5] Add point addition/removal functionality to curve editor
- [ ] T107 [US5] Implement fan curve screen in lib/screens/fans_screen.dart with editor and presets
- [ ] T108 [US5] Add client-side validation with safety warnings before applying curve
- [ ] T109 [US5] Display current fan RPM vs target RPM in fans screen

**Checkpoint**: User Story 5 complete - custom fan curve editor functional

---

## Phase 9: User Story 6 - Control RGB Lighting (Priority: P6)

**Goal**: Control keyboard backlight, Anime Matrix display, and peripheral lighting

**Independent Test**: Adjust backlight brightness, upload image to Anime Matrix, verify visual changes

### Rust Backend Implementation

- [ ] T110 [US6] Implement RgbController::new() in rust_core/src/services/rgb.rs with hidapi initialization
- [ ] T111 [US6] Implement keyboard_backlight_control() via sysfs LED interface
- [ ] T112 [US6] Implement anime_matrix_detection() for USB HID device (vendor ID 0x0b05)
- [ ] T113 [US6] Implement anime_matrix_send_image() using Starlight protocol reference
- [ ] T114 [US6] Implement set_keyboard_brightness() FFI function in rust_core/src/api.rs
- [ ] T115 [US6] Implement upload_anime_matrix_image() FFI function in rust_core/src/api.rs
- [ ] T116 [US6] Add USB HID error handling for disconnected devices

### Flutter Frontend Implementation

- [ ] T117 [P] [US6] Create RGB screen in lib/screens/rgb_screen.dart with tabbed interface
- [ ] T118 [US6] Implement keyboard brightness slider (0-3 levels) in RGB screen
- [ ] T119 [US6] Create Anime Matrix image upload widget with file picker
- [ ] T120 [US6] Add Anime Matrix preview widget in lib/widgets/anime_matrix_preview.dart
- [ ] T121 [US6] Implement brightness control for Anime Matrix display
- [ ] T122 [US6] Add error handling for missing RGB hardware

### Installation & Permissions

- [ ] T123 [P] [US6] Add USB HID udev rules to install/99-asus-control.rules for ASUS devices (vendor 0x0b05)
- [ ] T124 [US6] Update install.sh to include USB HID permission setup

**Checkpoint**: User Story 6 complete - RGB lighting control functional

---

## Phase 10: User Story 8 - Manage Display Settings (Priority: P8)

**Goal**: Control screen refresh rate and display overdrive for battery optimization

**Independent Test**: Change refresh rate, verify actual display refresh via system tools

### Rust Backend Implementation

- [ ] T125 [US7] Implement display_controller in rust_core/src/services/display.rs
- [ ] T126 [US7] Implement detect_supported_refresh_rates() via X11/Wayland detection
- [ ] T127 [US7] Implement set_refresh_rate() with xrandr or wlr-randr integration
- [ ] T128 [US7] Implement read_display_overdrive() from /sys/class/platform/asus-nb-wmi/panel_od
- [ ] T129 [US7] Implement write_display_overdrive() with toggle functionality
- [ ] T130 [US7] Add display control FFI functions to rust_core/src/api.rs

### Flutter Frontend Implementation

- [ ] T131 [US7] Create display settings section in lib/screens/settings_screen.dart
- [ ] T132 [US7] Implement refresh rate selector with detected available rates
- [ ] T133 [US7] Add display overdrive toggle switch
- [ ] T134 [US7] Show current active refresh rate in settings

**Checkpoint**: User Story 7 complete - display settings control functional

---

## Phase 11: Automation & Cross-Cutting Features

**Purpose**: Features that affect multiple user stories (FR-044 to FR-047)

### Automation Rules

- [ ] T135 [P] Create ConfigBloc in lib/blocs/config/config_bloc.dart for global settings
- [ ] T136 [P] Define automation rules in rust_core/src/config.rs (AC/battery triggers)
- [ ] T137 Implement auto_performance_mode_switch() on AC/battery state change in rust_core/src/services/performance.rs
- [ ] T138 [P] Implement auto_gpu_mode_switch() on power state change in rust_core/src/services/gpu.rs
- [ ] T139 [P] Implement auto_refresh_rate_adjust() on power state change in rust_core/src/services/display.rs
- [ ] T140 Add automation settings UI in lib/screens/settings_screen.dart with toggles

### Configuration Profiles

- [ ] T141 [P] Implement profile save/load in rust_core/src/config.rs
- [ ] T142 [P] Implement export_profile() FFI function in rust_core/src/api.rs
- [ ] T143 [P] Implement import_profile() FFI function in rust_core/src/api.rs
- [ ] T144 Add profile management UI in lib/screens/settings_screen.dart

---

## Phase 12: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Documentation & Installation

- [ ] T145 [P] Create comprehensive README.md with installation instructions
- [ ] T146 [P] Document hardware compatibility in docs/compatibility.md
- [ ] T147 [P] Create troubleshooting guide in docs/troubleshooting.md
- [ ] T148 Verify install.sh works on fresh system with no prior permissions

### Error Handling & Logging

- [ ] T149 [P] Add structured logging throughout Rust services with log crate
- [ ] T150 [P] Implement error logging to ~/.local/state/asus-linux-control/logs/
- [ ] T151 Add "View Logs" button in settings screen for debugging

### Performance & Optimization

- [ ] T152 Optimize sensor reading performance to meet <100ms latency requirement
- [ ] T153 Reduce memory footprint to <150MB target per performance goals
- [ ] T154 Profile app startup time and optimize to <2 seconds target

### Code Quality

- [ ] T155 [P] Run cargo clippy on Rust code and fix warnings
- [ ] T156 [P] Run flutter analyze and fix linting warnings
- [ ] T157 [P] Add Rust unit tests for safety validations in rust_core/tests/unit/
- [ ] T158 [P] Add Flutter widget tests for critical UI components in test/

### Final Validation

- [ ] T159 Run quickstart.md validation on clean system
- [ ] T160 Test all 8 user stories independently for acceptance scenarios
- [ ] T161 Verify constitution compliance (hardware safety, permissions, error handling)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-10)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if team capacity allows)
  - Or sequentially in priority order: US7(TDP) → US1 → US2 → US3 → US4 → US5 → US6 → US8
- **Automation (Phase 11)**: Depends on US1, US4, US8 being complete
- **Polish (Phase 12)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Independent - only needs Foundational phase
- **User Story 2 (P2)**: Independent - only needs Foundational phase
- **User Story 3 (P3)**: Independent - only needs Foundational phase
- **User Story 4 (P4)**: Independent - only needs Foundational phase
- **User Story 5 (P5)**: Independent - only needs Foundational phase, uses monitoring from US2 for validation
- **User Story 6 (P6)**: Independent - only needs Foundational phase
- **User Story 7 (P7)**: Independent - only needs Foundational phase (TDP control)
- **User Story 8 (P8)**: Independent - only needs Foundational phase (Display settings)

### Within Each User Story

- Rust backend implementation before Flutter frontend (FFI functions must exist)
- BLoC event/state definitions before BLoC implementation
- Widgets before screen integration
- Permission setup can proceed in parallel with implementation

### Parallel Opportunities

- **Setup phase**: All tasks T002-T007 can run in parallel
- **Foundational phase**: 
  - T012-T015 (data structures) in parallel
  - T016-T022 (service stubs) in parallel
  - T027-T033 (screen stubs) in parallel
- **Within each user story**: Model/event/state definitions can run in parallel
- **User stories**: After Foundational complete, US1-US8 can all start in parallel (with sufficient team)

---

## Parallel Example: User Story 1

```bash
# These tasks can run simultaneously:
T040: Create PerformanceBloc 
T041: Define PerformanceEvent
T042: Define PerformanceState
T047: Create udev rules file
T048: Create installation script

# Then sequentially:
T034-T039: Rust implementation (backend must exist before FFI calls)
T043-T046: Flutter UI integration (needs BLoC + FFI)
```

---

## Implementation Strategy

### Next Up: TDP (Phase 3) — Highest Priority

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T033b) - CRITICAL blocking phase
3. Complete **Phase 3: User Story 7 TDP** (T162-T182) ← **IMPLEMENT THIS NEXT**
4. **STOP and VALIDATE**: Test TDP control on hardware
5. Continue with Phase 4 (US1 Performance) and beyond

**TDP Deliverable**: Working GUI screen to set CPU power limits and thermal ceiling

### MVP First (User Story 1)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T033b) - CRITICAL blocking phase
3. Complete Phase 4: User Story 1 (T034-T049)
4. **STOP and VALIDATE**: Test performance mode switching independently
5. Deploy/demo MVP

**MVP Deliverable**: Working GUI app that switches performance modes on ASUS laptops

### Incremental Delivery

1. Setup + Foundational → Foundation ready (~25% complete)
2. Add US7 TDP → Test independently → **TDP Release** (~35% complete)
3. Add US1 → Test independently → **MVP Release** (~45% complete)
4. Add US2 → Test independently → **v0.2 Release** with monitoring (~55% complete)
5. Add US3-US8 → Progressive releases
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With 3 developers after Foundational phase completes:

- **Developer A**: US7 TDP (Phase 3) → US1 (Performance) → US4 (GPU)
- **Developer B**: US2 (Monitoring) → US5 (Fans)
- **Developer C**: US3 (Battery) → US6 (RGB) → US8 (Display)

All stories integrate through the common FFI bridge and don't conflict.

---

## Technical References by User Story

### US1: Performance Mode Control (✅ Complete)

**Kernel Interface:**
- sysfs: `/sys/class/platform/asus-nb-wmi/throttle_thermal_policy`
- Values: 0=Silent, 1=Balanced, 2=Turbo
- Kernel: 5.6+ (Dec 2019)
- ABI doc: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi

**Reference Implementations:**
- g-helper: https://github.com/seerge/g-helper/blob/main/app/Mode/CPUBoost.cs
- asusctl: https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-platform/src/platform.rs
- Kernel patch: https://patchwork.kernel.org/project/platform-driver-x86/patch/20191215142634.13888-1-leonmaxx@gmail.com/

---

### US2: System Health Monitoring (🎯 Next)

**Libraries:**
- Rust: `libmedium` v0.13+ - https://crates.io/crates/libmedium
- hwmon sysfs: `/sys/class/hwmon/hwmonX/`

**Kernel Interfaces:**
- Temperature: `tempX_input` (millidegrees C)
- Fan RPM: `fanX_input`
- PWM: `pwmX` (0-255)
- Docs: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface

**Reference:**
- lm-sensors: https://hwmon.wiki.kernel.org/
- libmedium examples: https://github.com/saronic-technologies/libmedium-rs

---

### US3: Battery Charge Limit

**Sysfs Interface:**
- Path: `/sys/class/power_supply/BAT0/charge_control_end_threshold`
- Range: 0-100 (percentage)
- Alternative: D-Bus UPower service

**D-Bus API:**
- Service: `org.freedesktop.UPower`
- Crate: `upower_dbus` v0.3+ - https://docs.rs/upower_dbus/
- Docs: https://upower.freedesktop.org/docs/UPower.html

**Reference:**
- asusctl battery: https://gitlab.com/asus-linux/asusctl/-/blob/main/daemon/src/ctrl_charge.rs

---

### US4: GPU Power Modes

**ASUS WMI:**
- dGPU toggle: `/sys/class/platform/asus-nb-wmi/dgpu_disable`
  - 0 = Enable dGPU, 1 = Disable dGPU
  - Kernel: 5.17+

**Nvidia:**
- NVML library: `nvml-wrapper` v0.10+ - https://docs.rs/nvml-wrapper/
- C Library: `libnvidia-ml.so` (included with drivers)
- Docs: https://docs.nvidia.com/deploy/nvml-api/

**AMD:**
- sysfs: `/sys/class/drm/card0/device/pp_od_clk_voltage`
- Power profile: `/sys/class/drm/card0/device/pp_power_profile_mode`
- Docs: https://docs.kernel.org/gpu/amdgpu/thermal.html

**Reference:**
- g-helper GPU: https://github.com/seerge/g-helper/blob/main/app/Mode/GPUMode.cs
- asusctl GPU: https://gitlab.com/asus-linux/asusctl/-/blob/main/daemon/src/ctrl_gpu.rs

---

### US5: Fan Curve Control

**hwmon PWM Interface:**
- PWM Value: `pwmX` (0-255)
- PWM Enable: `pwmX_enable` (0=full, 1=manual, 2=auto)
- Fan RPM: `fanX_input`

**Fan Curves (ASUS-specific):**
- Per-profile: Silent/Balanced/Turbo each has own curve
- Points: `pwm1_auto_pointN_temp` + `pwm1_auto_pointN_pwm` (N=1-8)
- ACPI: WMI method SFUN 0x07

**Kernel Support:**
- Patch: "asus-wmi: Add support for custom fan curves" v15
- Merged: kernel 6.0+
- Discussion: https://www.spinics.net/lists/kernel/msg4096977.html

**Constants (from kernel):**
```c
#define FAN_CURVE_DEV_MID      0x02
#define FAN_CURVE_PWM_MASK     0x04
```

**Reference:**
- asusctl fan: https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-dbus/src/ctrl_fan.rs
- Arch Wiki: https://wiki.archlinux.org/title/Fan_speed_control#ASUS_laptops

---

### US6: RGB Lighting & Anime Matrix

**Keyboard Backlight:**
- sysfs LED interface: `/sys/class/leds/`
- ASUS-specific: `asus::kbd_backlight`

**Anime Matrix:**
- USB HID protocol (reverse-engineered)
- Device ID: USB 0b05:1866 (ASUS N-KEY)
- **⚠️ Requires reverse-engineering from existing projects**

**Reference Implementations:**
- **Starlight** (C#): https://github.com/vddCore/Starlight
  - Full Anime Matrix driver for GA402 (G14 2022)
- **asusctl rog-anime** (Rust): https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-anime/src/usb.rs
  - USB HID communication module
- **SignalRGB** (C++): https://forum.signalrgb.com/t/asus-anime-matrix-support/356
- **Zephyrus Control Center** (C#): https://github.com/vddcore/Zephyrus-Control-Center

**USB HID:**
- Crate: `hidapi` v2.0+ - https://docs.rs/hidapi/
- libusb docs: https://libusb.sourceforge.io/api-1.0/

---

### US7: TDP & Temperature Limits (✅ Complete)

**Kernel sysfs (PPT - Package Power Tracking):**
- PL1/SPL: `/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl` (milliwatts)
- PL2/SPPT: `/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt` (milliwatts)
- FPPT: `/sys/devices/platform/asus-nb-wmi/ppt_fppt` (AMD only)
- APU SPPT: `/sys/devices/platform/asus-nb-wmi/ppt_apu_sppt`
- Platform SPPT: `/sys/devices/platform/asus-nb-wmi/ppt_platform_sppt`
- CPU Temp: `/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit` (°C)

**Kernel Version:**
- 5.17+: Basic PPT attributes
- 6.5+: Full PPT suite
- 6.10+: ⚠️ sysfs deprecated, use `asus-armoury` driver

**WMI Device IDs:**
```c
#define ASUS_WMI_DEVID_PPT_PL2_SPPT     0x001200A0
#define ASUS_WMI_DEVID_PPT_PL1_SPL      0x001200A3
#define ASUS_WMI_DEVID_PPT_APU_SPPT     0x001200B0
#define ASUS_WMI_DEVID_PPT_PLAT_SPPT    0x001200B1
#define ASUS_WMI_DEVID_PPT_PL3_FPPT     0x001200C1
```

**Reference Implementations:**
- **g-helper**: https://github.com/seerge/g-helper/blob/main/app/Mode/CPUBoost.cs
- **asusctl**: https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-platform/src/platform.rs
  - D-Bus API: https://asus-linux.org/wiki/asusctl-api/
- Kernel ABI: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi
- Driver source: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/asus-wmi.c

**⚠️ Important Note:**
Current implementation uses D-Bus daemon with polkit for privileged TDP operations. Direct sysfs writes require root or udev rules + polkit policy.

---

### US8: Display Settings

**Refresh Rate:**
- X11: `xrandr` via FFI or command
- Wayland: wlr-randr protocol or compositor-specific
- KDE: D-Bus `org.kde.KScreen`
- GNOME: `org.gnome.Mutter.DisplayConfig`

**Overdrive:**
- ASUS-specific: `/sys/class/platform/asus-nb-wmi/panel_od`
- Kernel: 5.17+
- Values: 0=disable, 1=enable

**Reference:**
- asusctl display: https://gitlab.com/asus-linux/asusctl/-/blob/main/daemon/src/ctrl_display.rs

---

## Notes

- [P] tasks = different files, no blocking dependencies, safe to parallelize
- [Story] label (US1-US7) maps task to specific user story for traceability
- Each user story independently completable per spec.md requirements
- Constitution safety checks (Principle I) enforced in T096, T161, T174
- BLoC pattern per FR-048 ensures clean state management separation
- fl_chart integration per FR-052 in T061, T104
- Material Design 3 dark theme per FR-049, FR-051 in T006
- All FFI functions follow type-safe contract per Principle III
- Total tasks: ~180 (7 Setup, 28 Foundational, 145 User Stories)
