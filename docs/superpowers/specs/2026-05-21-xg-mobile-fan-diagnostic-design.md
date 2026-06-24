# XG Mobile Fan Diagnostic Screen Design

Date: 2026-05-21

## Goal

Add a separate diagnostic screen that helps test whether XG Mobile and platform/EC policy change fan routing on Flow X13/GV301QH. The screen must guide the user through reboot-safe tests and explain whether fan behavior is controlled by direct fan curves, profile floors/caps, CPU/GPU coupling, or XG Mobile state.

The diagnostic should explain what happened in hardware terms, not just show raw numbers.

## Scope

In scope:
- Add a dedicated app screen reachable from the main navigation.
- Run hypothesis tests against balanced and selected platform profiles.
- Test whether GPU fan speed depends on XG Mobile state.
- Test whether GPU fan speed depends on CPU fan requests.
- Test whether profiles impose minimum/maximum RPM boundaries.
- Test whether EC/asusd applies curve points at runtime for the current temperature, while also recording whether it transforms stored curve points.
- Read fan RPMs, temperatures, curve data, and relevant hwmon values during the sweep.
- Restore balanced profile defaults after every run, including failures.
- Persist diagnostic runs to disk so the first run survives a reboot.
- Present a result summary and interpretation.
- Keep raw samples in memory for copy/export-friendly UI text.

Out of scope:
- Automatic XG Mobile connect/disconnect.
- Kernel module changes.
- Firmware/EC writes beyond existing asusd fan curve writes.
- Claiming GPU fan control works unless RPM changes are observed.

## User Flow

1. The user opens `XG Mobile Diagnostic`.
2. The screen shows safety state and current baseline:
   - AC power state if available.
   - CPU/GPU RPM readings.
   - current fan curve values for balanced profile.
   - current temperatures.
3. The user chooses a hypothesis test:
   - `XG Mobile Dependency`
   - `CPU/GPU Coupling`
   - `Profile Bounds`
   - `Curve Rule Discovery`
4. Each test explains exactly what it will write before it starts.
5. Every test restores profile defaults in `finally`.
6. For XG Mobile dependency, the user runs the first pass, changes XG Mobile state outside the app, reboots, and runs the second pass.
7. After reboot, the screen loads the saved first run.
8. The screen compares both runs and explains the likely result.

## Architecture

Add a small feature slice:

- `lib/services/xg_mobile_fan_diagnostic_service.dart`
  - Owns the hardware diagnostic sequence.
  - Uses `AsusdFanClient` for curve writes and restore.
  - Uses existing fan service/bridge reads where possible.
  - Reads additional hwmon files through a narrow `HwmonProbe` helper.
  - Serializes run data for reboot-safe A/B comparison.

- `lib/screens/xg_mobile_fan_diagnostic_screen.dart`
  - Stateful screen with the wizard and result summary.
  - Does not use the main `FanBloc`; this avoids mixing diagnostics with normal fan control state.

- `lib/main.dart`
  - Add a navigation rail destination for the diagnostic screen.

The service returns immutable result objects:
- `XgFanDiagnosticRun`
- `XgFanSweepStep`
- `XgFanSample`
- `XgFanDiagnosis`

Additional scenario results:
- `FanHypothesisResult`
- `FanHypothesisStep`
- `FanHypothesisDiagnosis`

## Hardware Writes

The sweep writes only the GPU curve for balanced profile:

- `180`
- `220`
- `255`

Each value is written as an eight-point flat curve using temperatures `30 40 50 60 70 80 90 100`, then enabled for `GPU`.

After the run, the service calls `SetCurvesToDefaults u 0`.

If an exception occurs, restore still runs in `finally`.

Additional writes:
- CPU/GPU coupling writes CPU-only and GPU-only flat curves for balanced profile.
- Profile bounds writes CPU flat curves per selected profile.
- Curve rule discovery writes a matrix of curves derived from the profile default curve, with mandatory below-default cases. It compares requested, readback, sysfs/runtime samples, current-temperature expected PWM, and RPM response to infer whether EC stores, transforms, applies, clamps, or ignores each curve shape.
- No test writes invalid sysfs `pwm_enable=0`.

## Interpretation Rules

The screen should classify results conservatively:

- `Responded`: `fan2_input` increases by at least 500 RPM during a step.
- `AcceptedNoResponse`: `pwm2_enable=1` and point8 matches requested PWM, but `fan2_input` changes by less than 500 RPM.
- `RouteChanged`: one run responds and the other does not.
- `NewHwmonLikely`: hwmon labels or fan IDs differ between runs.
- `Inconclusive`: missing RPM or missing hwmon data.

Text should make clear that accepted curve writes are not proof of physical fan control.

Additional hypothesis classifications:

- `GpuDependsOnXgMobile`: fan2 baseline or response changes between reboot-separated XG Mobile states.
- `GpuDependsOnCpuFan`: fan2 changes during CPU-only sweep but not GPU-only sweep.
- `ProfileFloorOrCap`: low PWM cannot lower RPM below a profile floor, or 255 differs by profile.
- `DownwardApplied`: requested PWM below default at current temperature is stored and observed RPM decreases.
- `DownwardStoredButRuntimeClamped`: requested PWM below default at current temperature is stored, but observed RPM does not decrease.
- `DownwardTransformedToDefault`: requested PWM below default at current temperature is transformed back to default or above on readback.
- `UpwardApplied`: requested PWM above default at current temperature is stored and observed RPM increases.
- `CurveTransformed`: requested curve differs from readback in a way not explained by downward-to-default clamp.
- `CurveNoRuntimeEffect`: requested/readback differ from default, but RPM does not meaningfully move.

## UI

The screen should be dense and operational:

- Baseline summary at top.
- Two run slots: `Current State` and `Changed XG Mobile State`.
- Hypothesis cards for CPU/GPU coupling, profile bounds, and curve rule discovery.
- Progress indicator with current PWM step and elapsed seconds.
- Result table with PWM, min/max/last fan2 RPM, point8, enable value, and temperatures.
- Interpretation panel with the next recommended action.

No marketing copy or decorative hero UI.

## Testing

Unit/widget tests should cover:

- The screen renders as a separate navigation destination.
- The service restores defaults after a successful run.
- The service restores defaults after a failed write or read.
- Interpretation classifies no-response sweep correctly.
- Interpretation classifies changed routing when only one of two runs responds.
- Interpretation classifies XG Mobile dependency when fan2 baseline or response differs between reboot-separated runs.
- Interpretation classifies CPU/GPU coupling when fan2 follows CPU-only sweep more than GPU-only sweep.
- Interpretation classifies profile floors/caps from per-profile RPM ranges.
- Interpretation classifies curve rules by comparing default/requested/readback values at the current temperature and observed RPM delta.

## Curve Rule Discovery Conditions

The curve rule test must not use readback alone as proof of curve behavior. It must compare requested values against the profile default CPU curve, then evaluate the curve at the measured CPU temperature and observe CPU RPM changes.

Required curve cases:

```text
default
flat_below_default_small_delta   default - 20, clamped to 0
flat_below_default_large_delta   default - 80, clamped to 0
current_bucket_below_default     only points around current temperature below default
current_bucket_above_default     only points around current temperature above default
all_above_default                default + 40, clamped to 255
flat_low                         all points 40
flat_high                        all points 255
non_monotonic_pwm                PWM not monotonic while temps remain monotonic
```

For each curve:

1. Record CPU RPM and CPU temperature before writing.
2. Compute default PWM at the current CPU temperature using linear interpolation.
3. Write the CPU curve using the default curve temperature points.
4. Read `FanCurveData` back.
5. Collect samples for the configured sample window.
6. Compute requested and readback PWM at the measured CPU temperature using linear interpolation between curve points; clamp below the first point and above the last point.
7. Convert PWM to expected CPU fan RPM using `10000 RPM` as the CPU fan max: `expectedRpm = pwm / 255 * 10000`.
8. Compare `defaultPwmAtTemp`, `requestedPwmAtTemp`, `readbackPwmAtTemp`, `rpmBefore`, `rpmAfter`, and `rpmDelta`.
9. Restore defaults after each case in `finally`.

Classification:

- `DownwardApplied`: requested and readback PWM are at least `20` below default at current temperature, and observed CPU RPM drops by at least `500`.
- `DownwardStoredButRuntimeClamped`: requested and readback PWM are at least `20` below default, but observed CPU RPM does not drop by at least `500`.
- `DownwardTransformedToDefault`: requested PWM is below default, but readback PWM is within `20` of default or above.
- `UpwardApplied`: requested and readback PWM are at least `20` above default, and observed CPU RPM rises by at least `500`.
- `CurveNoRuntimeEffect`: requested/readback PWM differ from default by at least `20`, but observed CPU RPM changes by less than `500`.
- `CurveTransformed`: requested/readback mismatch does not fit the downward-to-default pattern.

Tests should use fakes; no real D-Bus or sysfs access in tests.

## Open Safety Notes

The app cannot physically change XG Mobile state. The screen must ask the user to do that outside the app and rerun the sweep. It should not run the second sweep automatically after reconnect.

Because XG Mobile state changes require reboot on this machine, the screen persists the first run under the user's XDG state directory and reloads it after restart.
