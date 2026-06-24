# SAIO Direct Fan Control Test Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Determine whether the Windows G-Helper `AsusSAIO` direct fan protocol from `source.cs` can be reached safely on Linux for ASUS ROG Flow X13 GV301QH.

**Architecture:** Treat this as a staged hardware research effort, not application feature work. First document the Windows protocol and Linux exposure points, then search for a safe transport, then perform read-only probing only if a transport exists. Manual fan writes are out of scope until a follow-up plan explicitly approves them.

**Tech Stack:** Linux ACPI/WMI/sysfs/hwmon, decompiled ACPI DSL, optional `acpi_call` or kernel-module research only after explicit approval, Rust only for a later proof-of-concept probe, no Flutter/UI work.

---

## Current Findings

The local `source.cs` file is the source of truth for the G-Helper experimental direct fan path under review.

It uses the Windows `\\.\AsusSAIO` device with IOCTL `0x80102070`, not the normal ASUS ACPI `DEVS/DSTS` fan curve path. Commands are sent as 16-byte buffers and the code reads `output[11]`.

Observed protocol:

```text
FanInit:
  BB 01 50

GetFanCount:
  FanInit()
  DD 03 02 30

GetCPUTemp:
  DD 03 02

FanSelect(fanId):
  DD 03 82 32 <fanId>

FanControl(manual):
  FanInit()
  DD 03 82 31 <0|1>

GetFanSpeed(fanId):
  FanInit()
  FanSelect(fanId)
  high = DD 03 02 34
  low  = DD 03 02 33
  rpm  = high * 256 + low

FanSpeed(fanId, percent):
  FanInit()
  FanSelect(fanId)
  value = 255 * clamp(percent, 0..100) / 100
  DD 03 82 35 <value>
```

This means `0x00110022` and `0x00110023` are not the primary source-backed path for G-Helper direct fan speed. They remain fan-related ASUS WMI IDs, but the SAIO path is a separate Windows driver protocol.

## Safety Rules

- Do not run `sudo` commands automatically. Ask the user to run them.
- Do not use git-mutating commands. No `git add`, `git commit`, branch operations, reset, amend, rebase, merge, delete, or rename through git.
- Do not write low fan speeds during this plan.
- Do not write any fan speed at all in this plan unless the plan is explicitly revised.
- Do not touch `/dev/mem`, `/dev/port`, or raw EC ports from userspace.
- Do not load kernel modules, unload kernel modules, or write ACPI calls without explicit user approval.
- Every command that may affect firmware state must be listed for manual user approval before it is run.
- Stop immediately if RPM telemetry disappears, temperatures rise unexpectedly, the system logs firmware/ACPI errors, or auto mode cannot be confirmed.

## File Structure

- Modify: `docs/research/gv301qh-direct-fan-control.md`
  - Store protocol findings, transport findings, and final SAIO recommendation.
- Create: `docs/research/saio-direct-fan-test-log.md`
  - Store command outputs and timestamped observations from this plan.
- Possibly create later: `research/saio_probe/README.md`
  - Only if a safe Linux transport exists and a separate follow-up probe plan is approved.
- Do not modify Flutter, daemon, D-Bus, or production Rust files in this plan.

---

### Task 1: Record SAIO Protocol Baseline

**Files:**
- Modify: `docs/research/gv301qh-direct-fan-control.md`
- Create: `docs/research/saio-direct-fan-test-log.md`

- [ ] **Step 1: Confirm the local source file exists**

Run:

```bash
ls -l source.cs
```

Expected: command exits `0` and prints `source.cs`.

- [ ] **Step 2: Capture SAIO-relevant source lines**

Run:

```bash
rg -n 'AsusSAIO|CONTROL_CODE|CallMethod|GetFanCount|GetCPUTemp|GetFanSpeed|FanInit|FanSelect|FanControl|FanSpeed|2148540528|221|187|130|255' source.cs
```

Expected: output includes `\\.\AsusSAIO`, `2148540528`, `CallMethod`, and fan methods.

- [ ] **Step 3: Create the SAIO test log**

Create `docs/research/saio-direct-fan-test-log.md` with:

````markdown
# SAIO Direct Fan Test Log

## Protocol Source

Source file: `source.cs`

```text
Captured `rg` output from source.cs.
```

## Safety State

- No sudo commands are run by Codex.
- No git-mutating commands are run by Codex.
- No fan speed writes are part of this plan.
- Manual write probes require a separate follow-up plan.
````

Expected: file exists and contains the captured source summary.

- [ ] **Step 4: Update the main research document**

Ensure `docs/research/gv301qh-direct-fan-control.md` has a section named:

```markdown
## G-Helper `source.cs` AsusSAIO Findings
```

Expected: section states that `source.cs` uses `\\.\AsusSAIO`, IOCTL `0x80102070`, command `DD 03 82 35 <value>` for direct speed, and does not use `0x00110022/23`.

- [ ] **Step 5: Review diff only**

Run:

```bash
git diff -- docs/research/gv301qh-direct-fan-control.md docs/research/saio-direct-fan-test-log.md
```

Expected: diff shows documentation-only changes.

---

### Task 2: Check Current Linux Exposure Points

**Files:**
- Modify: `docs/research/saio-direct-fan-test-log.md`
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Check for Windows SAIO-equivalent device nodes**

Run:

```bash
find /dev /sys -iname '*saio*' -o -iname '*asus*win*' -o -iname '*winio*' 2>/dev/null
```

Expected: likely no Linux `AsusSAIO` equivalent exists. Record exact output.

- [ ] **Step 2: Check WMI devices**

Run:

```bash
find /sys/bus/wmi/devices -maxdepth 3 -type f 2>/dev/null | sort
```

Expected: output lists WMI device metadata. Record whether any obvious ASUS/ATK/SAIO raw method interface exists.

- [ ] **Step 3: Check loaded ACPI/WMI helpers**

Run:

```bash
lsmod | rg 'asus|wmi|acpi_call|ec_sys'
```

Expected: output includes `asus_wmi`; `acpi_call` and `ec_sys` may be absent.

- [ ] **Step 4: Check hwmon telemetry remains available**

Run:

```bash
for f in /sys/class/hwmon/hwmon*/fan*_input /sys/class/hwmon/hwmon*/temp*_input; do
  [ -e "$f" ] && printf '%s=%s\n' "$f" "$(cat "$f" 2>/dev/null)"
done
```

Expected: fan RPM and temperatures are readable.

- [ ] **Step 5: Record Linux exposure conclusion**

Append:

````markdown
## Linux SAIO Exposure Search

```text
Captured `/dev` and `/sys` SAIO search output.
```

```text
Captured WMI devices output.
```

```text
Captured module state.
```

```text
Captured telemetry.
```

Conclusion:

- Native `AsusSAIO` device present:
- Generic raw WMI call interface present:
- Existing kernel driver exposes SAIO command protocol:
- Safe read-only transport identified:
````

Expected: conclusion clearly says whether a safe transport exists.

---

### Task 3: Search ACPI Tables for SAIO Command Equivalents

**Files:**
- Modify: `docs/research/saio-direct-fan-test-log.md`
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Verify ACPI DSL files exist**

Run:

```bash
ls /tmp/gv301qh-acpi/*.dsl
```

Expected: DSL files from the earlier ACPI dump exist. If missing, ask the user to rerun the ACPI dump/decompile steps; do not run `sudo`.

- [ ] **Step 2: Search for SAIO command bytes and nearby EC plumbing**

Run:

```bash
rg -n '0xBB|0xDD|0x82|0x50|0x31|0x32|0x33|0x34|0x35|SAIO|WinIO|WMNB|ECAC|ISMI|WEBC|REBC|STCD|STCC' /tmp/gv301qh-acpi/*.dsl
```

Expected: output identifies whether the SAIO byte protocol appears in ACPI tables or only generic EC/WMI plumbing appears.

- [ ] **Step 3: Inspect exact-match context**

Run:

```bash
rg -n -C 8 '0xBB|0xDD|0x82|0x35|SAIO' /tmp/gv301qh-acpi/*.dsl
```

Expected: capture the surrounding method and determine whether it is fan-related or unrelated byte data.

- [ ] **Step 4: Record ACPI transport conclusion**

Append:

````markdown
## SAIO ACPI Symbol Search

```text
Captured SAIO byte/protocol search output.
```

Conclusion:

- SAIO command bytes present in ACPI tables:
- SAIO commands map to `WMNB`:
- SAIO commands map to EC helper methods:
- Linux can call the same protocol through existing ACPI method:
````

Expected: no write probes are performed.

---

### Task 4: Decide Whether a Read-Only SAIO Probe Is Possible

**Files:**
- Modify: `docs/research/saio-direct-fan-test-log.md`
- Modify: `docs/research/gv301qh-direct-fan-control.md`

- [ ] **Step 1: Evaluate transport gate**

Use this checklist:

````markdown
## SAIO Read-Only Probe Gate

- Source protocol documented:
- Linux transport exists without new kernel code:
- Transport can send 16-byte buffer and return 16-byte buffer:
- Read-only command exists that does not require manual mode:
- RPM/temperature telemetry remains readable:
- No raw EC port writes required:
- No fan speed writes required:

Decision:

- Proceed to read-only SAIO probe:
- Stop at documentation:
````

- [ ] **Step 2: Stop if no safe transport exists**

If decision is `Stop at documentation`, append:

````markdown
## SAIO Transport Recommendation

The Windows `AsusSAIO` direct fan protocol is documented, but no safe Linux transport has been identified. Do not implement direct fan control in the daemon or UI. Next work would require kernel-adjacent research to expose a constrained, logged, high-safety transport for the SAIO command protocol.
````

- [ ] **Step 3: If transport exists, create a follow-up read-only probe plan**

Only if decision is `Proceed to read-only SAIO probe`, create a new plan. The follow-up plan may test only:

```text
GetCPUTemp: DD 03 02
GetFanCount: BB 01 50, then DD 03 02 30
GetFanSpeed: only if FanSelect is proven harmless or already required by read path
```

Expected: no write probe is added to this plan.

- [ ] **Step 4: Review diff only**

Run:

```bash
git diff -- docs/research/gv301qh-direct-fan-control.md docs/research/saio-direct-fan-test-log.md docs/superpowers/plans/2026-05-20-saio-direct-fan-control-test.md
```

Expected: diff shows docs and plan changes only.

---

## Later Path If Read-Only SAIO Probe Works

If a read-only SAIO transport is proven, create a separate write-probe plan with these constraints:

- Manual approval before every state-changing command.
- Start from high duty values only, such as 220..255.
- Enable manual mode only for a short fixed timeout.
- Restore automatic mode immediately.
- Capture RPM and temperatures before, during, and after.
- Abort on missing telemetry, unexpected RPM drop, ACPI/kernel errors, or inability to restore automatic mode.

## Stop Conditions

Stop the SAIO path if any of these are true:

- No Linux transport can send the 16-byte SAIO command buffer.
- The only possible transport requires raw `/dev/mem`, `/dev/port`, or blind EC writes.
- Read-only commands require enabling manual fan mode.
- RPM or temperature telemetry becomes unavailable.
- A transport requires loading unreviewed kernel code.
- Auto restore cannot be demonstrated before any speed write.

## References

- Local protocol source: `source.cs`
- Existing research document: `docs/research/gv301qh-direct-fan-control.md`
- ACPI dump workspace: `/tmp/gv301qh-acpi`
