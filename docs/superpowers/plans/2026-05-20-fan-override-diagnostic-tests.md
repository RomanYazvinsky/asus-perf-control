# Fan Override Diagnostic Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove whether fan speed drops from 100% to about 70% because `asusd` reapplies profile fan curves, the app writes the wrong profile/state, the dynamic engine stops/restores auto mode, or the firmware/EC clamps RPM despite a persisted 255 PWM curve.

**Architecture:** Run small, isolated hardware experiments and log the same state at every boundary: `asusd` D-Bus profile data, hwmon curve files, hwmon enable files, active platform profile, RPM, and temperatures. Start read-only, then use short controlled writes through the existing `asusd` and daemon paths, restoring defaults after each write test.

**Tech Stack:** Linux sysfs hwmon, `busctl`, `journalctl`, `asusd` D-Bus interface `xyz.ljones.FanCurves`, app daemon D-Bus interface `com.asuslinuxcontrol.Daemon`, Flutter/Rust app fan service code.

**Current constraint:** Do not create commits during the app diagnostic iteration.

---

## File Structure

- Create: `docs/research/fan-override-diagnostic-log.md`
  - Stores timestamped command output, observations, and conclusions from each experiment.
- Modify: `lib/services/xg_mobile_fan_diagnostic_service.dart`
  - Adds reboot-safe XG Mobile fan diagnostics and curve rule discovery.
- Modify: `lib/screens/xg_mobile_fan_diagnostic_screen.dart`
  - Adds the separate diagnostic screen controls and result interpretation.
- First pass was documentation-only. The follow-up app iteration adds a separate diagnostic screen and does not change the normal fan-control path.
  - If the tests confirm profile reapply as the root cause, a later implementation should touch `lib/services/fan/fan_service_impl.dart`, `lib/services/asusd_fan_client.dart`, `rust_daemon/src/fan_engine/*`, and fan bloc/UI tests.

## Safety Rules

- Run write tests only on AC power.
- Keep 100% fan writes short-lived: 30-90 seconds per test.
- Record current curves before every write test.
- Restore previous curves or defaults after each write test.
- Stop immediately if CPU/GPU temperature telemetry disappears or fan RPM reads return zero while hot.

### Task 1: Create Diagnostic Log

**Files:**
- Create: `docs/research/fan-override-diagnostic-log.md`

- [x] **Step 1: Create the log with fixed sections**

```markdown
# Fan Override Diagnostic Log

Date: 2026-05-20
Machine: ROG Flow X13 GV301QH

## Baseline

## Test A: Read-Only State Snapshot

## Test B: App/asusd 100% Constant Speed Write

## Test C: Platform Profile Reapply Trigger

## Test D: AC/Battery Profile Trigger

## Test E: Dynamic Engine 100% Write

## Test F: Kernel Full-Speed Enable Probe

## Conclusion
```

- [x] **Step 2: Commit the empty log**

Run:

```bash
git add docs/research/fan-override-diagnostic-log.md
git commit -m "docs: add fan override diagnostic log"
```

Expected: commit succeeds.

### Task 2: Baseline Read-Only Snapshot

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [x] **Step 1: Capture active platform profile and all asusd curves**

Run:

```bash
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnAc
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnBattery
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnAc
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnBattery
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 1
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 2
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 3
```

Expected: current active profile is known, and each profile reports CPU/GPU `pwm`, `temp`, and `enabled`.

- [x] **Step 2: Capture hwmon state**

Run:

```bash
for f in /sys/class/hwmon/hwmon7/fan*_input /sys/class/hwmon/hwmon7/fan*_label /sys/class/hwmon/hwmon7/pwm*_enable /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm*_auto_point*_pwm /sys/class/hwmon/hwmon8/pwm*_auto_point*_temp /sys/class/hwmon/hwmon1/temp1_input /sys/class/hwmon/hwmon5/temp1_input /sys/class/hwmon/hwmon6/temp1_input; do printf '%s=' "$f"; cat "$f" 2>/dev/null; done
```

Expected: hwmon8 curve values match the active `asusd` profile curve.

- [x] **Step 3: Capture recent asusd fan writes**

Run:

```bash
journalctl -u asusd.service -n 180 --no-pager | rg 'write_profile_curve_to_platform|watch_platform_profile|Platform:'
```

Expected: log shows whether `asusd` recently reapplied profile curves.

- [x] **Step 4: Record baseline interpretation**

Append to `docs/research/fan-override-diagnostic-log.md`:

```markdown
Baseline interpretation:
- Active profile:
- Active hwmon8 CPU max PWM:
- Active hwmon8 GPU max PWM:
- asusd CPU enabled:
- asusd GPU enabled:
- Recent asusd profile reapply seen: yes/no
```

### Task 3: Prove Whether 100% Persists After Direct asusd Write

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [x] **Step 1: Save active profile curves before writing**

Run:

```bash
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
```

Expected: output is copied into the log under “Before write”.

- [x] **Step 2: Write CPU and GPU flat 100% curves to active balanced profile**

Run on AC power only:

```bash
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetFanCurve 'u(s(yyyyyyyy)(yyyyyyyy)b)' 0 CPU 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetProfileFanCurveEnabled usb 0 CPU true
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetFanCurve 'u(s(yyyyyyyy)(yyyyyyyy)b)' 0 GPU 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetProfileFanCurveEnabled usb 0 GPU true
```

Expected: calls return no error.

- [x] **Step 3: Monitor for 90 seconds**

Run:

```bash
for i in $(seq 1 90); do date --iso-8601=seconds; busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile; busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0; for f in /sys/class/hwmon/hwmon7/fan*_input /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm /sys/class/hwmon/hwmon1/temp1_input /sys/class/hwmon/hwmon5/temp1_input /sys/class/hwmon/hwmon6/temp1_input; do printf '%s=' "$f"; cat "$f" 2>/dev/null; done; sleep 1; done
```

Expected for profile reapply hypothesis: at the drop moment, `FanCurveData` or hwmon8 point 8 changes from `255` to profile values such as `193`/`173`, or `enabled` flips to `false`.

Expected for EC clamp hypothesis: `FanCurveData` and hwmon8 stay `255`/enabled, but RPM falls anyway.

- [x] **Step 4: Restore defaults or saved curve**

Run:

```bash
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetCurvesToDefaults u 0
```

Expected: balanced profile returns to default curve.

### Task 4: Trigger Platform Profile Reapply

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [x] **Step 1: Repeat Task 3 write**

Run the four `SetFanCurve` / `SetProfileFanCurveEnabled` commands from Task 3.

Expected: profile 0 returns to flat 100%.

- [x] **Step 2: Force active profile reapply without changing user intent**

Run:

```bash
busctl --system set-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile u 0
```

Expected: call succeeds.

- [x] **Step 3: Inspect curve after reapply**

Run:

```bash
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
for f in /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm; do printf '%s=' "$f"; cat "$f"; done
journalctl -u asusd.service -n 60 --no-pager | rg 'write_profile_curve_to_platform|watch_platform_profile|Platform:'
```

Expected for profile reapply root cause: the curve is changed back to saved profile values or `enabled=false`, and journal shows `write_profile_curve_to_platform`.

- [x] **Step 4: Restore defaults**

Run:

```bash
busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetCurvesToDefaults u 0
```

Expected: no error.

### Task 5: Test AC/Battery Auto Profile Trigger

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [x] **Step 1: Confirm auto profile switching is enabled**

Run:

```bash
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnAc
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnBattery
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnAc
busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnBattery
```

Expected: if both booleans are true, AC/Battery events can overwrite the active fan curve by switching platform profile.

- [ ] **Step 2: With user present, unplug/replug AC while monitoring**

Run before unplugging:

```bash
for i in $(seq 1 120); do date --iso-8601=seconds; busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile; for f in /sys/class/hwmon/hwmon7/fan*_input /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm; do printf '%s=' "$f"; cat "$f" 2>/dev/null; done; sleep 1; done
```

Expected: profile and fan curve change around AC state changes if asusd auto-switch is involved.

### Task 6: Test Dynamic Engine Behavior Separately

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [ ] **Step 1: Confirm engine starts and writes 255 through sysfs**

Use the app dynamic fan mode with both CPU/GPU curves set to 100% at all points and tick interval at a short value.

Expected: `GetFanEngineRunning` returns true.

Run:

```bash
busctl --system call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetFanEngineRunning
for f in /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm; do printf '%s=' "$f"; cat "$f"; done
```

Expected: `b true`, `pwm*_enable=1`, point 8 values `255`.

- [ ] **Step 2: Monitor for 90 seconds**

Run:

```bash
for i in $(seq 1 90); do date --iso-8601=seconds; busctl --system call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetFanEngineRunning; for f in /sys/class/hwmon/hwmon7/fan*_input /sys/class/hwmon/hwmon8/pwm*_enable /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm; do printf '%s=' "$f"; cat "$f" 2>/dev/null; done; sleep 1; done
```

Expected: if dynamic engine stops after write errors, running flips to false and auto mode is restored. If it stays running with `255` but RPM falls, EC clamp remains plausible.

- [ ] **Step 3: Stop engine and restore auto**

Use the app stop control, then run:

```bash
busctl --system call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetFanEngineRunning
for f in /sys/class/hwmon/hwmon8/pwm*_enable; do printf '%s=' "$f"; cat "$f"; done
```

Expected: `b false`, `pwm*_enable=2`.

### Task 7: Optional Kernel Full-Speed Enable Probe

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [ ] **Step 1: Record current `asus` hwmon enable values**

Run:

```bash
for f in /sys/class/hwmon/hwmon7/pwm*_enable /sys/class/hwmon/hwmon7/fan*_input; do printf '%s=' "$f"; cat "$f"; done
```

Expected: values recorded before writes.

- [ ] **Step 2: Try full-speed enable for 30 seconds**

Run only on AC power:

```bash
printf 0 | sudo tee /sys/class/hwmon/hwmon7/pwm1_enable
printf 0 | sudo tee /sys/class/hwmon/hwmon7/pwm2_enable
for i in $(seq 1 30); do date --iso-8601=seconds; for f in /sys/class/hwmon/hwmon7/pwm*_enable /sys/class/hwmon/hwmon7/fan*_input; do printf '%s=' "$f"; cat "$f"; done; sleep 1; done
printf 2 | sudo tee /sys/class/hwmon/hwmon7/pwm1_enable
printf 2 | sudo tee /sys/class/hwmon/hwmon7/pwm2_enable
```

Expected: if `0` maps to full speed on this kernel path, RPM rises and stays high during the 30-second window. If the write is rejected or silently normalized, record exact behavior.

### Task 8: Decide Follow-Up Implementation

**Files:**
- Modify: `docs/research/fan-override-diagnostic-log.md`

- [ ] **Step 1: Classify result**

Use this decision table:

```markdown
Result classification:
- Profile reapply confirmed:
  Evidence: PWM/enable changed from 255/true to profile values after asusd/platform event.
  Fix direction: persist intended constant speed per profile and reapply on profile/AC changes.

- App writes wrong profile confirmed:
  Evidence: non-active profile changes, active hwmon8 does not.
  Fix direction: resolve active profile before write and update selected profile semantics.

- Dynamic engine stop confirmed:
  Evidence: GetFanEngineRunning changes true -> false before drop.
  Fix direction: inspect daemon tick errors and sysfs write failures.

- EC clamp confirmed:
  Evidence: asusd and hwmon8 stay 255/enabled while RPM drops.
  Fix direction: do not claim direct 100% duty via curves; investigate kernel full-speed mode or SAIO transport.
```

- [ ] **Step 2: Commit completed log**

Run:

```bash
git add docs/research/fan-override-diagnostic-log.md
git commit -m "docs: record fan override diagnostic results"
```

Expected: commit succeeds with completed evidence.

## Self-Review

- Spec coverage: plan covers read-only baseline, asusd write path, profile reapply trigger, AC/Battery trigger, dynamic engine path, and optional kernel full-speed probe.
- Placeholder scan: no TBD steps; every test has commands and expected outcomes.
- Type consistency: `FanCurveData`, `SetFanCurve`, `SetProfileFanCurveEnabled`, `GetFanEngineRunning`, and hwmon paths match the observed local system.
