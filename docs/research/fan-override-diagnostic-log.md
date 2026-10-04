# Fan Override Diagnostic Log

Date: 2026-05-20
Machine: ROG Flow X13 GV301QH

## Baseline

Log created in commit `db6f279 docs: add fan override diagnostic log`.

## Test A: Read-Only State Snapshot

Captured: 2026-05-20, local system.

### Platform profile state

```text
$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile
u 0

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnAc
u 1

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnBattery
u 2

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnAc
b true

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnBattery
b true
```

Interpretation: active platform profile is `0` (balanced). Automatic profile switching is enabled for both AC and battery; AC target is profile `1`, battery target is profile `2`.

### asusd fan curve data

```text
$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false

$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 1
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 81 86 117 132 170 239 239 239 48 56 60 63 67 72 72 72 false "GPU" 86 94 124 155 173 224 224 224 48 56 60 64 67 72 72 72 false

$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 2
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 58 81 109 124 124 124 55 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 114 145 145 145 55 58 62 65 68 73 73 73 false

$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 3
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 58 81 109 124 124 124 55 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 114 145 145 145 55 58 62 65 68 73 73 73 false
```

All saved profile curves currently report `enabled=false`.

### hwmon state

```text
/sys/class/hwmon/hwmon7/fan1_input=5000
/sys/class/hwmon/hwmon7/fan2_input=4500
/sys/class/hwmon/hwmon7/fan1_label=cpu_fan
/sys/class/hwmon/hwmon7/fan2_label=gpu_fan
/sys/class/hwmon/hwmon7/pwm1_enable=2
/sys/class/hwmon/hwmon7/pwm2_enable=0
/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm1_auto_point1_pwm=20
/sys/class/hwmon/hwmon8/pwm1_auto_point2_pwm=43
/sys/class/hwmon/hwmon8/pwm1_auto_point3_pwm=66
/sys/class/hwmon/hwmon8/pwm1_auto_point4_pwm=81
/sys/class/hwmon/hwmon8/pwm1_auto_point5_pwm=117
/sys/class/hwmon/hwmon8/pwm1_auto_point6_pwm=193
/sys/class/hwmon/hwmon8/pwm1_auto_point7_pwm=193
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=193
/sys/class/hwmon/hwmon8/pwm2_auto_point1_pwm=15
/sys/class/hwmon/hwmon8/pwm2_auto_point2_pwm=45
/sys/class/hwmon/hwmon8/pwm2_auto_point3_pwm=66
/sys/class/hwmon/hwmon8/pwm2_auto_point4_pwm=94
/sys/class/hwmon/hwmon8/pwm2_auto_point5_pwm=155
/sys/class/hwmon/hwmon8/pwm2_auto_point6_pwm=173
/sys/class/hwmon/hwmon8/pwm2_auto_point7_pwm=173
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=173
/sys/class/hwmon/hwmon8/pwm1_auto_point1_temp=53
/sys/class/hwmon/hwmon8/pwm1_auto_point2_temp=58
/sys/class/hwmon/hwmon8/pwm1_auto_point3_temp=62
/sys/class/hwmon/hwmon8/pwm1_auto_point4_temp=65
/sys/class/hwmon/hwmon8/pwm1_auto_point5_temp=68
/sys/class/hwmon/hwmon8/pwm1_auto_point6_temp=73
/sys/class/hwmon/hwmon8/pwm1_auto_point7_temp=73
/sys/class/hwmon/hwmon8/pwm1_auto_point8_temp=73
/sys/class/hwmon/hwmon8/pwm2_auto_point1_temp=53
/sys/class/hwmon/hwmon8/pwm2_auto_point2_temp=58
/sys/class/hwmon/hwmon8/pwm2_auto_point3_temp=62
/sys/class/hwmon/hwmon8/pwm2_auto_point4_temp=65
/sys/class/hwmon/hwmon8/pwm2_auto_point5_temp=68
/sys/class/hwmon/hwmon8/pwm2_auto_point6_temp=73
/sys/class/hwmon/hwmon8/pwm2_auto_point7_temp=73
/sys/class/hwmon/hwmon8/pwm2_auto_point8_temp=73
/sys/class/hwmon/hwmon1/temp1_input=62000
/sys/class/hwmon/hwmon5/temp1_input=58000
/sys/class/hwmon/hwmon6/temp1_input=63125
```

The active hwmon8 curve matches profile `0` from `asusd`: CPU max PWM `193`, GPU max PWM `173`, temp points `53 58 62 65 68 73 73 73`.

### Recent asusd fan writes

```text
May 20 10:44:09 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: CPU, pwm: [20, 43, 66, 81, 117, 193, 193, 193], temp: [53, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 10:44:10 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: GPU, pwm: [15, 45, 66, 94, 155, 173, 173, 173], temp: [53, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 10:44:10 fedora asusd[1860]: [DEBUG asusd::ctrl_fancurves] watch_platform_profile changed
May 20 10:44:10 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(performance), CurveData { fan: CPU, pwm: [81, 86, 117, 132, 170, 239, 239, 239], temp: [48, 56, 60, 63, 67, 72, 72, 72], enabled: false }
May 20 10:44:10 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(performance), CurveData { fan: GPU, pwm: [86, 94, 124, 155, 173, 224, 224, 224], temp: [48, 56, 60, 64, 67, 72, 72, 72], enabled: false }
May 20 10:48:51 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(quiet), CurveData { fan: CPU, pwm: [20, 43, 58, 81, 109, 124, 124, 124], temp: [55, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 10:48:51 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(quiet), CurveData { fan: GPU, pwm: [15, 45, 66, 94, 114, 145, 145, 145], temp: [55, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 12:43:52 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(performance), CurveData { fan: CPU, pwm: [81, 86, 117, 132, 170, 239, 239, 239], temp: [48, 56, 60, 63, 67, 72, 72, 72], enabled: false }
May 20 12:43:52 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(performance), CurveData { fan: GPU, pwm: [86, 94, 124, 155, 173, 224, 224, 224], temp: [48, 56, 60, 64, 67, 72, 72, 72], enabled: false }
May 20 12:43:52 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: CPU, pwm: [20, 43, 66, 81, 117, 193, 193, 193], temp: [53, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 12:43:52 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: GPU, pwm: [15, 45, 66, 94, 155, 173, 173, 173], temp: [53, 58, 62, 65, 68, 73, 73, 73], enabled: false }
May 20 19:41:06 fedora asusd[1860]: [DEBUG asusd::ctrl_platform] Platform: watch_platform_profile changed
May 20 19:41:06 fedora asusd[1860]: [DEBUG asusd::ctrl_fancurves] watch_platform_profile changed
```

Baseline interpretation:
- Active profile: `0` balanced.
- Active hwmon8 CPU max PWM: `193`.
- Active hwmon8 GPU max PWM: `173`.
- asusd CPU enabled: `false`.
- asusd GPU enabled: `false`.
- Recent asusd profile reapply seen: yes. Journal shows `write_profile_curve_to_platform` after `watch_platform_profile changed` events for balanced/performance/quiet profile transitions.

## Test B: App/asusd 100% Constant Speed Write

Captured: 2026-05-20 20:20:41-20:22:12 +02:00.

Note: an earlier sandboxed monitor attempt was discarded because `busctl` inside the compound shell loop could not access the system bus. The curve was restored immediately after that failed attempt. The evidence below is from the full escalated rerun with automatic `SetCurvesToDefaults u 0` restore.

### Before write

```text
$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false
```

### Write result

The four `SetFanCurve` / `SetProfileFanCurveEnabled` calls returned no error. Immediate readback:

```text
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
```

`asusd` journal showed the balanced profile receiving the 255 curves:

```text
May 20 20:20:41 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: CPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
May 20 20:20:41 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: GPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
```

### 90-second monitor summary

Raw monitor log: `/tmp/fan-task3-monitor-full.log` (1080 lines, 90 timestamp samples).

```text
start=2026-05-20T20:20:41+02:00
end=2026-05-20T20:22:12+02:00
samples=90

unique hwmon8 control values:
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=255
/sys/class/hwmon/hwmon8/pwm1_enable=1
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=255
/sys/class/hwmon/hwmon8/pwm2_enable=1

unique FanCurveData:
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true

fan1_min=3700
fan1_max=9500
fan1_last=9300
fan2_min=3300
fan2_max=3300
fan2_last=3300

/sys/class/hwmon/hwmon1/temp1_input min=57000 max=69000 last=60000
/sys/class/hwmon/hwmon5/temp1_input min=53000 max=59000 last=54000
/sys/class/hwmon/hwmon6/temp1_input min=57250 max=69875 last=60250
```

Representative start/end samples:

```text
2026-05-20T20:20:41+02:00
u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
/sys/class/hwmon/hwmon7/fan1_input=3700
/sys/class/hwmon/hwmon7/fan2_input=3300
/sys/class/hwmon/hwmon8/pwm1_enable=1
/sys/class/hwmon/hwmon8/pwm2_enable=1
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=255
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=255

2026-05-20T20:22:12+02:00
u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
/sys/class/hwmon/hwmon7/fan1_input=9300
/sys/class/hwmon/hwmon7/fan2_input=3300
/sys/class/hwmon/hwmon8/pwm1_enable=1
/sys/class/hwmon/hwmon8/pwm2_enable=1
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=255
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=255
```

Interpretation: direct `asusd` write to active profile `0` persisted for the full 90-second window. `FanCurveData`, `pwm*_enable`, and point 8 PWM stayed at `true`/`1`/`255`. CPU RPM climbed to about 9500 and ended around 9300, while GPU RPM stayed fixed at 3300 despite the GPU hwmon curve also staying at 255/enabled.

### Restore readback

```text
$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetCurvesToDefaults u 0

$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false
/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=193
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=173
```

## Test C: Platform Profile Reapply Trigger

Captured: 2026-05-20 20:23:41-20:23:50 +02:00.

### Repeat 100% write

The four `SetFanCurve` / `SetProfileFanCurveEnabled` calls returned no error. Immediate readback before reapply:

```text
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true
```

### Reapply command

```text
$ busctl --system set-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfile u 0
```

The call returned no error.

### After reapply

```text
$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true "GPU" 255 255 255 255 255 255 255 255 30 40 50 60 70 80 90 100 true

/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=255
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=255
```

Relevant journal lines:

```text
May 20 20:23:41 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: CPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
May 20 20:23:41 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: GPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
May 20 20:23:47 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: CPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
May 20 20:23:47 fedora asusd[1860]: [DEBUG rog_profiles] write_profile_curve_to_platform: writing profile:(balanced), CurveData { fan: GPU, pwm: [255, 255, 255, 255, 255, 255, 255, 255], temp: [30, 40, 50, 60, 70, 80, 90, 100], enabled: true }
May 20 20:23:50 fedora asusd[1860]: [DEBUG asusd::ctrl_fancurves] watch_platform_profile changed
May 20 20:23:50 fedora asusd[1860]: [DEBUG asusd::ctrl_platform] Platform: watch_platform_profile changed
```

Interpretation: setting `PlatformProfile u 0` did not revert the saved profile curve to default `193/173`; `FanCurveData` stayed `255/true`. However, hwmon `pwm1_enable` and `pwm2_enable` changed to `2` while point 8 PWM remained `255`. This suggests the same-profile reapply can alter the control mode without overwriting the stored curve values.

### Restore readback

```text
$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves SetCurvesToDefaults u 0

$ busctl --system call xyz.ljones.Asusd /xyz/ljones xyz.ljones.FanCurves FanCurveData u 0
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false
/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm=193
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=173
```

## Test D: AC/Battery Profile Trigger

Captured: 2026-05-20, Step 1 only.

```text
$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnAc
b true

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform ChangePlatformProfileOnBattery
b true

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnAc
u 1

$ busctl --system get-property xyz.ljones.Asusd /xyz/ljones xyz.ljones.Platform PlatformProfileOnBattery
u 2
```

Interpretation: AC/Battery auto switching is enabled. An AC unplug/replug event can switch profile to battery target `2` or AC target `1`, so Step 2 still needs a physical unplug/replug test while monitoring.

## Test E: Dynamic Engine 100% Write

## Test F: Kernel Full-Speed Enable Probe

## Additional Test: GPU PWM Sweep Below 255

Captured: 2026-05-20 20:37:05-20:38:36 +02:00.

Goal: test whether the GPU fan ignores only `255`, or whether lower flat PWM values also fail to affect `fan2_input`.

Method: keep CPU curve at default, write only GPU flat curves for `180`, `200`, `220`, `240`, `250`, and `255`; monitor each value for 15 seconds; restore profile `0` defaults at the end.

Raw monitor log: `/tmp/gpu-pwm-sweep.log` (826 lines).

Baseline:

```text
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false
/sys/class/hwmon/hwmon7/fan1_input=3600
/sys/class/hwmon/hwmon7/fan2_input=3300
/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=173
```

Sweep summary:

```text
value  samples  fan2_min  fan2_max  fan2_last  pwm2_enable  point8
180    15       3300      3300      3300       1            180
200    15       3300      3300      3300       1            200
220    15       3200      3300      3300       1            220
240    15       3300      3300      3300       1            240
250    15       3300      3300      3300       1            250
255    15       3300      3300      3300       1            255
```

Temperature ranges during the sweep:

```text
value  hwmon5_temp1_input        hwmon6_temp1_input
180    54000-58000 last=55000    58875-63250 last=62250
200    54000-56000 last=54000    57625-61750 last=57625
220    55000-56000 last=55000    58250-68000 last=65250
240    55000-57000 last=55000    58875-64500 last=58875
250    55000-56000 last=56000    59000-63250 last=62125
255    55000-60000 last=55000    59750-67000 last=64500
```

Restore readback:

```text
a(s(yyyyyyyy)(yyyyyyyy)b) 2 "CPU" 20 43 66 81 117 193 193 193 53 58 62 65 68 73 73 73 false "GPU" 15 45 66 94 155 173 173 173 53 58 62 65 68 73 73 73 false
/sys/class/hwmon/hwmon8/pwm1_enable=2
/sys/class/hwmon/hwmon8/pwm2_enable=2
/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm=173
/sys/class/hwmon/hwmon7/fan1_input=3700
/sys/class/hwmon/hwmon7/fan2_input=3300
```

Interpretation: GPU lower PWM values did not produce a meaningful fan response. The driver accepted and exposed each GPU PWM value (`pwm2_enable=1`, point8 matched the requested value), but `fan2_input` stayed at about 3300 RPM. This weakens the hypothesis that only `255` is special or clamped; the observed issue is more likely that the exposed GPU curve path does not control the physical second fan on this machine, or the firmware/EC ignores this curve channel under the tested conditions.

## App Diagnostic Module: Curve Rule Discovery

Updated: 2026-05-21.

The app diagnostic screen now tests curve transformation/ignore rules directly instead of treating readback as success. The test writes a CPU curve matrix derived from the balanced default CPU curve and evaluates each case at the measured CPU temperature with linear interpolation.

Cases:

```text
default
flat_below_default_small_delta
flat_below_default_large_delta
current_bucket_below_default
current_bucket_above_default
all_above_default
flat_low
flat_high
non_monotonic_pwm
```

For each case, the app records CPU RPM and CPU temperature before the write, writes the CPU curve using the default temperature points, reads `FanCurveData` back, samples runtime RPM/temperature, computes `defaultPwmAtTemp`, `requestedPwmAtTemp`, `readbackPwmAtTemp`, and maps requested PWM to expected CPU RPM with `10000 RPM` as the maximum. Defaults are restored after each case.

Classification focuses on the failure mode:

```text
DownwardApplied
DownwardStoredButRuntimeClamped
DownwardTransformedToDefault
UpwardApplied
CurveNoRuntimeEffect
CurveTransformed
```

This specifically checks the missing lowering case: whether a curve below the default is stored and applied, stored but clamped at runtime, or transformed back toward the default curve before use.

## Conclusion
