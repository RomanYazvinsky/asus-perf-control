# GV301QH Direct Fan Control Research Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Determine whether ASUS ROG Flow X13 GV301QH can support G-Helper-like direct fan duty control on Linux, and identify the smallest safe implementation path.

**Architecture:** Treat this as a staged research and proof-of-concept effort. First collect hardware evidence, then inspect Linux ASUS WMI support, then attempt read-only probing, then decide whether a kernel patch/out-of-tree module is justified. Flutter and daemon integration come only after a direct low-level primitive is proven.

**Tech Stack:** Linux `asus-wmi`, `asus-nb-wmi`, `asus-armoury`, `hwmon` sysfs, ACPI tables, kernel source inspection, Rust daemon only after proof, Flutter/Dart only after daemon API exists.

---

## Current Findings

The Windows `AsusSAIO` example cannot be ported directly to Linux. It talks to the Windows `\\.\AsusSAIO` device and `AsusWinIO64.dll`; Linux does not expose that driver stack.

The G-Helper experimental path appears to set a fan control value in the `0..255` range derived from a user percentage. It does not set an exact RPM directly. RPM is measured separately after the firmware or embedded controller reacts.

On this machine, current local evidence shows:

```text
Model: ROG Flow X13 GV301QH
Board: GV301QH
Kernel: 7.0.1-cachyos1.fc44.x86_64
/sys/class/hwmon/hwmon7/name: asus
/sys/class/hwmon/hwmon7/fan1_label: cpu_fan
/sys/class/hwmon/hwmon7/fan2_label: gpu_fan
/sys/class/hwmon/hwmon7/pwm1_enable
/sys/class/hwmon/hwmon7/pwm2_enable
/sys/class/hwmon/hwmon8/name: asus_custom_fan_curve
/sys/class/hwmon/hwmon8/pwm1_auto_point*_pwm/temp
/sys/class/hwmon/hwmon8/pwm2_auto_point*_pwm/temp
```

The important missing files are:

```text
/sys/class/hwmon/.../pwm1
/sys/class/hwmon/.../pwm2
/sys/class/hwmon/.../fan1_target
/sys/class/hwmon/.../fan2_target
```

Without one of those, standard Linux userspace does not currently expose direct fan duty or target RPM control for this model.

## Scope

This plan covers research and proof-of-concept only. It does not implement a Flutter UI, daemon D-Bus API, or production fan controller until direct low-level control is proven.

## Safety Rules

- Never write a low fan duty value during probing.
- Initial write probes must use only high values, such as `220..255`.
- Every write probe must restore automatic mode immediately.
- Every write probe must have a hard timeout.
- Every write probe must log fan RPM and CPU/GPU temperatures before and after.
- Stop the research path if a write causes unstable readings, missing RPM telemetry, firmware errors, or inability to restore auto mode.

## File Structure

- Create: `docs/research/gv301qh-direct-fan-control.md`
  - Stores the hardware report, kernel source notes, ACPI/WMI findings, probe results, and final recommendation.
- Possibly create later: `research/direct_fan_probe/`
  - Temporary research code only if read-only ACPI/WMI evidence indicates a write primitive may exist.
- Possibly modify later: `rust_daemon/src/hardware/fan.rs`
  - Only after a Linux primitive exists and is safe enough to expose through the daemon.
- Possibly modify later: `rust_daemon/src/main.rs`
  - Only after the daemon gets a direct fan D-Bus API.
- Possibly modify later: `lib/services/fan/fan_service_contract.dart`
  - Only after daemon support exists.
- Possibly modify later: `lib/services/fan/fan_service_impl.dart`
  - Only after daemon support exists.

---

### Task 1: Create Baseline Hardware Report

**Files:**
- Create: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Create the research directory**

Run:

```bash
mkdir -p docs/research
```

Expected: command exits with status `0`.

- [ ] **Step 2: Collect platform identity**

Run:

```bash
uname -a
cat /sys/class/dmi/id/product_name 2>/dev/null
cat /sys/class/dmi/id/board_name 2>/dev/null
cat /sys/class/dmi/id/product_family 2>/dev/null
```

Expected: output includes `ROG Flow X13 GV301QH`, `GV301QH`, and the active kernel version.

- [ ] **Step 3: Collect ASUS kernel module state**

Run:

```bash
lsmod | rg 'asus|wmi'
```

Expected: output includes `asus_wmi`, `asus_nb_wmi`, and possibly `asus_armoury`.

- [ ] **Step 4: Collect hwmon fan tree**

Run:

```bash
for d in /sys/class/hwmon/hwmon*; do
  printf '%s: ' "$d"
  cat "$d/name" 2>/dev/null
  ls -l "$d"/fan* "$d"/pwm* 2>/dev/null
done
```

Expected: output shows `asus` and `asus_custom_fan_curve` devices. Record whether direct `pwm1`, `pwm2`, `fan1_target`, or `fan2_target` files exist.

- [ ] **Step 5: Save findings**

Add this structure to `docs/research/gv301qh-direct-fan-control.md` and fill the code blocks with the command output from Steps 2-4:

````markdown
# GV301QH Direct Fan Control Research

## Hardware

- Product:
- Board:
- Product family:
- Kernel:

## ASUS Modules

```text
Captured `lsmod | rg 'asus|wmi'` output.
```

## hwmon Fan Interfaces

```text
Captured hwmon fan tree output.
```

## Initial Conclusion

- Direct `pwmN` files present: yes/no
- `fanN_target` files present: yes/no
- Custom fan curve files present: yes/no
```
````

- [ ] **Step 6: Commit baseline report**

Run:

```bash
git add docs/research/gv301qh-direct-fan-control.md
git commit -m "docs: record GV301QH fan control baseline"
```

Expected: commit succeeds.

---

### Task 2: Verify Direct PWM Is Not Hidden Elsewhere

**Files:**
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Search physical platform paths**

Run:

```bash
find /sys/devices/platform/asus-nb-wmi -maxdepth 8 -type f 2>/dev/null | sort | rg 'fan|pwm'
```

Expected: list includes all ASUS platform fan-related sysfs files.

- [ ] **Step 2: Search all hwmon direct-control candidates**

Run:

```bash
find /sys/class/hwmon -type f 2>/dev/null | sort | rg '/pwm[0-9]+$|/fan[0-9]+_target$|/fan[0-9]+_input$|/pwm[0-9]+_enable$'
```

Expected: output confirms whether any direct `pwmN` or target RPM file exists.

- [ ] **Step 3: Record result**

Append to `docs/research/gv301qh-direct-fan-control.md` and fill the code blocks with the command output from Steps 1-2:

````markdown
## Direct PWM Search

```text
Captured physical path search output.
```

```text
Captured hwmon direct-control search output.
```

Conclusion:

- Direct duty interface found:
- Target RPM interface found:
- Only custom-curve interface found:
```
````

- [ ] **Step 4: Commit sysfs verification**

Run:

```bash
git add docs/research/gv301qh-direct-fan-control.md
git commit -m "docs: verify GV301QH fan sysfs controls"
```

Expected: commit succeeds.

---

### Task 3: Inspect ACPI/WMI Tables

**Files:**
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Install ACPI tools if missing**

Run:

```bash
command -v acpidump
command -v acpixtract
command -v iasl
```

Expected: each command prints a path. If any command is missing, install the distro package that provides ACPI tools before continuing.

- [ ] **Step 2: Dump ACPI tables**

Run:

```bash
sudo acpidump -o /tmp/gv301qh-acpi.dat
```

Expected: `/tmp/gv301qh-acpi.dat` exists.

- [ ] **Step 3: Extract and decompile ACPI tables**

Run:

```bash
mkdir -p /tmp/gv301qh-acpi
cp /tmp/gv301qh-acpi.dat /tmp/gv301qh-acpi/
cd /tmp/gv301qh-acpi
acpixtract -a gv301qh-acpi.dat
iasl -d *.dat
```

Expected: `.dsl` files are created in `/tmp/gv301qh-acpi`.

- [ ] **Step 4: Search fan-related ACPI symbols**

Run:

```bash
rg -n 'AGFN|GFAN|SFUN|MFUN|FAN|CPU_FAN|GPU_FAN|0x00110013|0x00110014|0x00110022|0x00110023|0x00110024|0x00110025' /tmp/gv301qh-acpi/*.dsl
```

Expected: output identifies whether AGFN or ASUS fan device IDs appear in firmware tables.

- [ ] **Step 5: Record ACPI findings**

Append to `docs/research/gv301qh-direct-fan-control.md` and fill the code block with the command output from Step 4:

````markdown
## ACPI/WMI Table Search

Searched symbols:

- AGFN
- GFAN
- SFUN
- MFUN
- FAN
- 0x00110013
- 0x00110014
- 0x00110022
- 0x00110023
- 0x00110024
- 0x00110025

```text
Captured ACPI/WMI symbol search output.
```

Conclusion:

- AGFN present:
- Fan curve device IDs present:
- Fan direct-control candidate present:
```
````

- [ ] **Step 6: Commit ACPI findings**

Run:

```bash
git add docs/research/gv301qh-direct-fan-control.md
git commit -m "docs: inspect GV301QH ACPI fan methods"
```

Expected: commit succeeds.

---

### Task 4: Compare Against Linux Kernel ASUS WMI Implementation

**Files:**
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Inspect upstream `asus-wmi.c`**

Review the current upstream Linux source:

```text
https://raw.githubusercontent.com/torvalds/linux/master/drivers/platform/x86/asus-wmi.c
```

Focus on:

```text
asus_wmi_has_agfn_fan
asus_agfn_fan_speed_write
asus_fan_set_auto
pwm1_store
ASUS_WMI_DEVID_CPU_FAN_CTRL
ASUS_WMI_DEVID_GPU_FAN_CTRL
ASUS_WMI_DEVID_CPU_FAN_CURVE
ASUS_WMI_DEVID_GPU_FAN_CURVE
```

- [ ] **Step 2: Record the implementation constraints**

Append to `docs/research/gv301qh-direct-fan-control.md`:

````markdown
## Linux Kernel ASUS WMI Notes

Relevant upstream functions:

- `asus_wmi_has_agfn_fan`
- `asus_agfn_fan_speed_write`
- `asus_fan_set_auto`
- `pwm1_store`

Observed constraints:

- AGFN direct fan write is exposed as:
- AGFN direct write supports fan IDs:
- Modern CPU/GPU fan curve support is exposed as:
- Kernel currently exposes direct `pwmN` for this machine:

Conclusion:

- Existing upstream interface is enough:
- Kernel patch likely required:
```
````

- [ ] **Step 3: Commit kernel notes**

Run:

```bash
git add docs/research/gv301qh-direct-fan-control.md
git commit -m "docs: compare GV301QH fans with asus-wmi"
```

Expected: commit succeeds.

---

### Task 5: Decide Whether a Probe Is Justified

**Files:**
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Evaluate probe prerequisites**

Use this checklist:

````markdown
## Probe Gate

- ACPI/WMI table contains AGFN or a fan direct-control candidate:
- Kernel source has a matching read path:
- Existing sysfs does not already expose direct control:
- Probe can be read-only first:
- Probe can restore auto mode after write:
- Current RPM and temperature telemetry remain readable:

Decision:

- Proceed to read-only probe:
- Stop research:
```
````

- [ ] **Step 2: Stop if prerequisites are not met**

If the decision is `Stop research`, append:

````markdown
## Final Recommendation

Direct fan duty or target RPM control is not currently exposed on this GV301QH Linux stack. The application should not claim G-Helper-like direct fan control. Supported fan control remains custom curves and dynamic curve updates through `asus-wmi`/`asusd`/`hwmon`.
```
````

- [ ] **Step 3: Continue only if prerequisites are met**

If the decision is `Proceed to read-only probe`, create a separate follow-up implementation plan before writing probe code. The probe must be reviewed as kernel-adjacent work, not normal app code.

- [ ] **Step 4: Commit probe decision**

Run:

```bash
git add docs/research/gv301qh-direct-fan-control.md
git commit -m "docs: decide GV301QH direct fan probe gate"
```

Expected: commit succeeds.

---

## Later Implementation Path If Direct Control Is Proven

If a safe direct duty primitive is proven, create a new implementation plan for production work:

- Add daemon method `SetDirectFanDuty(fan, duty)` with polkit authorization.
- Add daemon method `SetDirectFanAuto(fan)`.
- Add optional daemon method `SetTargetFanRpm(fan, rpm)` as a closed-loop controller.
- Add tests for duty clamping, auto restore, timeout, and telemetry failure.
- Add Dart client methods in `lib/services/fan/`.
- Add UI only after daemon behavior is verified on hardware.

## Stop Conditions

Stop the direct-control path if any of these are true:

- No AGFN or equivalent ACPI/WMI primitive is present.
- Direct write works only for one fan when the feature requires CPU and GPU fans.
- RPM telemetry disappears in manual mode.
- Auto restore cannot be proven.
- Fan duty write produces unstable firmware behavior.
- The only workable control remains custom fan curves.

## References

- G-Helper experimental manual fan control: `https://g-helper.com/experimental/`
- Linux ASUS WMI source: `https://raw.githubusercontent.com/torvalds/linux/master/drivers/platform/x86/asus-wmi.c`
- Linux hwmon sysfs ABI: `https://www.kernel.org/doc/html/latest/hwmon/sysfs-interface.html`
- ASUS Linux custom fan curves: `https://asus-linux.org/faq/asusctl/custom-fan-curves/`
