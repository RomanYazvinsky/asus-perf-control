# Ryzen Boost Modes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Ryzen CPU boost presets similar to G-Helper, including a full boost-off mode.

**Architecture:** Keep ASUS platform controls on the existing `asusd` path and put Linux CPU boost/EPP writes behind the privileged `rust_daemon`. The Flutter UI talks only to service interfaces and BLoCs; the daemon owns boost/EPP sysfs writes, validation, persistence, startup re-apply, polkit authorization, and resolving the active system power-plan EPP baseline. Governor selection stays owned by the system power-plan stack and is never written by the boost feature.

**Tech Stack:** Flutter, flutter_bloc, Dart `dbus`, Rust daemon with `zbus`, Linux CPUFreq/amd-pstate sysfs, asusd D-Bus.

---

## Key Mapping

G-Helper exposes Windows `ProcessorPerformanceBoostMode` values such as `Disabled`, `Enabled`, `Aggressive`, `Efficient Enabled`, and `Efficient Aggressive`. On Linux there is no one-to-one Windows policy value, so implement compatible behavior as explicit presets over:

- boost switch: `/sys/devices/system/cpu/cpufreq/boost`, or per-policy/per-CPU `boost` when only that exists
- EPP hint: `/sys/devices/system/cpu/cpufreq/policy*/energy_performance_preference`
- optional max frequency cap later as a separate explicit mode, if testing shows boost-off is not enough on a specific driver

Proposed presets:

| UI mode | Boost | EPP target | Availability rule | Intent |
| --- | --- | --- | --- | --- |
| Disabled | `0` | active system power-plan baseline, when EPP exists | boost writable | Never boost above base where the driver honors the boost knob; restore the active system power plan's EPP baseline when EPP is available. |
| Efficient | `1` | `power` | `power` EPP available and writable | Allow boost, bias toward lowest power. |
| Efficient Enabled | `1` | `balance_power` | `balance_power` EPP available and writable | Normal boost with efficiency bias. |
| Enabled | `1` | active system power-plan baseline, when EPP exists | boost writable | Default boost for the active system power plan. |
| Efficient Aggressive | `1` | `balance_performance` | `balance_performance` EPP available and writable | Prefer responsive boost without forcing full performance EPP. |
| Aggressive | `1` | `performance` | `performance` EPP available and writable | Max responsiveness and higher power/noise. |

The active system power-plan baseline is not an app constant and is not "leave whatever is currently set". It is the EPP state produced by the current system power-plan owner. In this app's current stack that owner is `asusd` `PlatformProfile` (`LowPower=3`, `Balanced=0`, `Performance=1`). To resolve baseline EPP values, re-apply the active system power plan through its owner, then read `energy_performance_preference` from each cpufreq policy. If a later stack uses `power-profiles-daemon` instead, add a system-power-plan adapter rather than hardcoding Linux EPP values.

Do not write `/sys/devices/system/cpu/cpufreq/policy*/scaling_governor` from this feature. Governor changes are broader power-plan behavior and belong to `asusd`, `power-profiles-daemon`, or the user's system configuration.

If a required EPP value is not listed by the kernel for every writable policy, do not offer that EPP-dependent mode in `availableModes`. Do not choose a nearby EPP value, and do not partially offer a preset whose EPP write would be skipped. If `dynamic_epp` is enabled and blocks manual EPP writes, surface `eppWritable=false` and remove EPP-dependent modes from `availableModes`. `Disabled` and `Enabled` remain available on boost-only systems when the boost knob is writable; in that case they only write boost.

## Control Flow Scheme

The shared app interaction scheme lives in `docs/app-function-interactions.md`.
The planned Ryzen boost route is:

`ui -> CpuBoostBloc -> BridgeService -> CpuBoostDaemonClient -> own dbus / rust_daemon -> cpufreq sysfs`

## File Structure

- Create `lib/services/cpu_boost_service.dart`
  - Define `CpuBoostMode`, `CpuBoostStatus`, and `ICpuBoostService`.
- Create `lib/services/cpu_boost_daemon_client.dart`
  - D-Bus wrapper for `GetCpuBoostStatus()` and `SetCpuBoostMode(String mode)`.
- Modify `lib/services/bridge_service.dart`
  - Implement `ICpuBoostService` and route boost calls to the daemon client.
- Create `lib/blocs/cpu_boost/cpu_boost_event.dart`
- Create `lib/blocs/cpu_boost/cpu_boost_state.dart`
- Create `lib/blocs/cpu_boost/cpu_boost_bloc.dart`
  - Load status, set mode, expose unsupported/error states.
- Create `lib/widgets/cpu_boost/boost_mode_selector.dart`
  - Reusable selector for the boost presets.
- Modify `lib/screens/home_screen.dart` or create a dedicated CPU screen
  - Add the selector near performance/TDP controls.
- Create `rust_daemon/src/hardware/cpu_boost.rs`
  - Discover cpufreq policies, available EPP values, boost knob location, dynamic EPP lock, active system power-plan EPP baseline, and apply presets.
- Modify `rust_daemon/src/hardware/mod.rs`
  - Export `cpu_boost`.
- Modify `rust_daemon/src/main.rs`
  - Add D-Bus methods and startup re-apply.
- Modify `install/dbus/com.asuslinuxcontrol.Daemon.policy`
  - Add `com.asuslinuxcontrol.set-cpu-boost-mode`.
- Modify tests:
  - `test/blocs/cpu_boost/cpu_boost_bloc_test.dart`
  - `test/services/cpu_boost_daemon_client_test.dart`
  - `test/screens/home_screen_test.dart`
  - `rust_daemon/tests/cpu_boost_test.rs`

## Task 1: Model The Boost Presets

**Files:**
- Create: `lib/services/cpu_boost_service.dart`
- Test: `test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

- [ ] **Step 1: Write a failing enum/status test**

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/cpu_boost_service.dart';

void main() {
  test('CpuBoostMode exposes stable daemon ids', () {
    expect(CpuBoostMode.disabled.daemonId, 'disabled');
    expect(CpuBoostMode.efficient.daemonId, 'efficient');
    expect(CpuBoostMode.efficientEnabled.daemonId, 'efficient_enabled');
    expect(CpuBoostMode.enabled.daemonId, 'enabled');
    expect(CpuBoostMode.efficientAggressive.daemonId, 'efficient_aggressive');
    expect(CpuBoostMode.aggressive.daemonId, 'aggressive');
  });
}
```

- [ ] **Step 2: Run the test**

Run: `flutter test test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

Expected: FAIL because `cpu_boost_service.dart` does not exist.

- [ ] **Step 3: Add the model and service contract**

```dart
enum CpuBoostMode {
  disabled('disabled', 'Disabled'),
  efficient('efficient', 'Efficient'),
  efficientEnabled('efficient_enabled', 'Efficient Enabled'),
  enabled('enabled', 'Enabled'),
  efficientAggressive('efficient_aggressive', 'Efficient Aggressive'),
  aggressive('aggressive', 'Aggressive');

  const CpuBoostMode(this.daemonId, this.label);

  final String daemonId;
  final String label;

  static CpuBoostMode fromDaemonId(String id) {
    return CpuBoostMode.values.firstWhere(
      (mode) => mode.daemonId == id,
      orElse: () => CpuBoostMode.enabled,
    );
  }
}

class CpuBoostStatus {
  final bool isSupported;
  final CpuBoostMode currentMode;
  final bool boostEnabled;
  final String? epp;
  final bool eppWritable;
  final List<CpuBoostMode> availableModes;

  const CpuBoostStatus({
    required this.isSupported,
    required this.currentMode,
    required this.boostEnabled,
    required this.availableModes,
    this.epp,
    this.eppWritable = true,
  });
}

abstract class ICpuBoostService {
  Future<CpuBoostStatus> getCpuBoostStatus();
  Future<void> setCpuBoostMode(CpuBoostMode mode);
}
```

- [ ] **Step 4: Run the test**

Run: `flutter test test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

Expected: PASS for the model test.

## Task 2: Add The Flutter Service And BLoC

**Files:**
- Create: `lib/services/cpu_boost_daemon_client.dart`
- Modify: `lib/services/bridge_service.dart`
- Create: `lib/blocs/cpu_boost/cpu_boost_event.dart`
- Create: `lib/blocs/cpu_boost/cpu_boost_state.dart`
- Create: `lib/blocs/cpu_boost/cpu_boost_bloc.dart`
- Test: `test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

- [ ] **Step 1: Test load and set flows**

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_bloc.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_event.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_state.dart';
import 'package:flutter_application_1/services/cpu_boost_service.dart';

class MockCpuBoostService extends Mock implements ICpuBoostService {}

void main() {
  late MockCpuBoostService service;

  setUp(() {
    service = MockCpuBoostService();
  });

  test('loads current boost mode', () async {
    when(() => service.getCpuBoostStatus()).thenAnswer(
      (_) async => const CpuBoostStatus(
        isSupported: true,
        currentMode: CpuBoostMode.efficientAggressive,
        boostEnabled: true,
        epp: 'balance_performance',
        availableModes: CpuBoostMode.values,
      ),
    );

    final bloc = CpuBoostBloc(service: service)..add(LoadCpuBoostStatus());

    await expectLater(
      bloc.stream,
      emitsInOrder([
        isA<CpuBoostLoading>(),
        isA<CpuBoostLoaded>().having(
          (s) => s.status.currentMode,
          'mode',
          CpuBoostMode.efficientAggressive,
        ),
      ]),
    );
  });

  test('sets boost disabled and reloads status', () async {
    when(() => service.setCpuBoostMode(CpuBoostMode.disabled))
        .thenAnswer((_) async {});
    when(() => service.getCpuBoostStatus()).thenAnswer(
      (_) async => const CpuBoostStatus(
        isSupported: true,
        currentMode: CpuBoostMode.disabled,
        boostEnabled: false,
        availableModes: CpuBoostMode.values,
      ),
    );

    final bloc = CpuBoostBloc(service: service)
      ..add(SetCpuBoostModeRequested(CpuBoostMode.disabled));

    await expectLater(
      bloc.stream,
      emitsInOrder([
        isA<CpuBoostLoading>(),
        isA<CpuBoostLoaded>().having(
          (s) => s.status.boostEnabled,
          'boostEnabled',
          false,
        ),
      ]),
    );
  });
}
```

- [ ] **Step 2: Run the test**

Run: `flutter test test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

Expected: FAIL because the BLoC files do not exist.

- [ ] **Step 3: Implement events, states, and BLoC**

`CpuBoostBloc` should mirror existing `PerformanceBloc`: emit loading, call service, emit loaded or error. After `SetCpuBoostModeRequested`, call `setCpuBoostMode(mode)` and then `getCpuBoostStatus()` so the UI shows the actual applied state.

- [ ] **Step 4: Implement daemon client and bridge wiring**

`CpuBoostDaemonClient` should call:

- `GetCpuBoostStatus() -> (bssbsas)` or JSON string if tuple maintenance becomes brittle
- `SetCpuBoostMode(s) -> b`

Prefer JSON for forward compatibility:

```json
{
  "isSupported": true,
  "currentMode": "efficient_aggressive",
  "boostEnabled": true,
  "epp": "balance_performance",
  "eppWritable": true,
  "availableModes": ["disabled", "efficient", "efficient_enabled", "enabled", "efficient_aggressive", "aggressive"]
}
```

- [ ] **Step 5: Run the tests**

Run: `flutter test test/blocs/cpu_boost/cpu_boost_bloc_test.dart`

Expected: PASS.

## Task 3: Add Rust Daemon Sysfs Support

**Files:**
- Create: `rust_daemon/src/hardware/cpu_boost.rs`
- Modify: `rust_daemon/src/hardware/mod.rs`
- Modify: `rust_daemon/src/main.rs`
- Modify: `install/dbus/com.asuslinuxcontrol.Daemon.policy`
- Test: `rust_daemon/tests/cpu_boost_test.rs`

- [ ] **Step 1: Test preset resolution without real sysfs**

Use temp directories in Rust tests and inject a fake cpufreq root. Cover:

- `disabled` writes `boost=0`, restores active system power-plan EPP when EPP exists, and never writes `scaling_governor`
- `enabled` writes `boost=1`, restores active system power-plan EPP when EPP exists, and never writes `scaling_governor`
- `aggressive` writes `boost=1` and `energy_performance_preference=performance`, and never writes `scaling_governor`
- modes with unavailable required EPP strings are omitted from `availableModes`
- `SetCpuBoostMode` rejects a mode that is not currently in `availableModes` before writing any sysfs value
- dynamic EPP lock reports `eppWritable=false` and removes EPP-dependent modes from `availableModes`
- boost-only systems expose only `Disabled` and `Enabled` when the boost knob is writable

- [ ] **Step 2: Implement discovery**

Discovery order:

1. global boost: `/sys/devices/system/cpu/cpufreq/boost`
2. policy boost: `/sys/devices/system/cpu/cpufreq/policy*/boost`
3. per-CPU boost: `/sys/devices/system/cpu/cpu*/cpufreq/boost`
4. policies: `/sys/devices/system/cpu/cpufreq/policy*`

If none exist, return `isSupported=false`.

- [ ] **Step 3: Implement apply**

For each requested mode:

- before writing, validate that the requested mode is present in the current `availableModes`
- reject unavailable modes with a readable error and do not write partial state
- write boost first after validation succeeds
- never write `scaling_governor`
- write the exact EPP target from the preset when the preset has a concrete EPP target
- for `disabled` and `enabled`, restore active system power-plan EPP when EPP exists; otherwise only write boost
- collect failed writes into status warnings for logs

- [ ] **Step 4: Persist selected mode**

Save the selected daemon id to `/var/lib/asus-linux-control/cpu-boost-mode.json`. On daemon startup, after temp limit re-apply, read and re-apply the saved boost mode. If apply fails, log and keep daemon running.

- [ ] **Step 5: Add D-Bus methods**

In `rust_daemon/src/main.rs`:

- `GetCpuBoostStatus() -> s` JSON status
- `SetCpuBoostMode(mode: String) -> bool`

Use new polkit action `com.asuslinuxcontrol.set-cpu-boost-mode` for writes. Read status does not require authorization.

- [ ] **Step 6: Run daemon tests**

Run: `cd rust_daemon && cargo test cpu_boost`

Expected: PASS.

## Task 4: Add UI Selector

**Files:**
- Create: `lib/widgets/cpu_boost/boost_mode_selector.dart`
- Modify: `lib/screens/home_screen.dart`
- Test: `test/screens/home_screen_test.dart`

- [ ] **Step 1: Add widget tests**

Cover:

- all supported modes render
- current mode is selected
- tapping `Disabled` dispatches `SetCpuBoostModeRequested(CpuBoostMode.disabled)`
- unsupported status shows a compact unavailable message

- [ ] **Step 2: Implement selector**

Use the same visual density as existing `ModeSelector`, but keep this as a separate component. Labels should be mode names only; descriptions can be short:

- Disabled: "No CPU boost"
- Efficient: "Lowest boost bias"
- Efficient Enabled: "Efficient normal boost"
- Enabled: "Balanced boost"
- Efficient Aggressive: "Responsive efficient boost"
- Aggressive: "Maximum boost response"

- [ ] **Step 3: Wire into screen**

Add `CpuBoostBloc` provider next to `PerformanceBloc`. Load on screen open. Show errors through `CopyableAlert.showError`.

- [ ] **Step 4: Run widget tests**

Run: `flutter test test/screens/home_screen_test.dart`

Expected: PASS.

## Task 5: Integration Verification

**Files:**
- No production files unless tests expose a bug.

- [ ] **Step 1: Static analysis**

Run: `flutter analyze`

Expected: no new analyzer errors.

- [ ] **Step 2: Flutter tests**

Run: `flutter test`

Expected: PASS.

- [ ] **Step 3: Rust daemon tests**

Run: `cd rust_daemon && cargo test`

Expected: PASS.

- [ ] **Step 4: Manual Linux verification**

On a supported AMD laptop:

```bash
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetCpuBoostStatus
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon SetCpuBoostMode s disabled
cat /sys/devices/system/cpu/cpufreq/boost
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon SetCpuBoostMode s efficient_aggressive
cat /sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference
```

Expected: disabled mode writes boost `0` and restores active system power-plan EPP when EPP exists; efficient/aggressive modes restore boost `1` and apply only their exact required EPP values. No CPU boost mode writes `scaling_governor`.

## Risks And Decisions

- G-Helper's names come from Windows boost policy. Linux implementation must be behavior-compatible, not value-compatible.
- `amd_pstate` active mode may reject EPP writes when `dynamic_epp` is enabled; this should be visible in status, not treated as total failure.
- Some systems expose only global boost, some per-policy/per-CPU boost, and some none. The UI must show unsupported when no writable boost or EPP control is available.
- Boost-off should be implemented through the kernel boost knob first. Frequency caps are a separate later mechanism because they are more invasive and can fight user power profiles.

## References

- G-Helper project notes that it provides "turbo boost selection for every performance mode": https://github.com/seerge/g-helper
- Microsoft documents Windows `ProcessorPerformanceBoostMode` values including disabled/efficient/aggressive variants: https://learn.microsoft.com/en-us/dotnet/api/microsoft.windows.eventtracing.power.processorperformanceboostmode
- Linux amd-pstate docs describe the AMD p-state driver, EPP, and boost sysfs controls: https://www.kernel.org/doc/html/latest/admin-guide/pm/amd-pstate.html
- Linux CPUFreq docs describe CPUFreq policies and boost sysfs behavior: https://kernel.org/doc/html/next/admin-guide/pm/cpufreq.html
