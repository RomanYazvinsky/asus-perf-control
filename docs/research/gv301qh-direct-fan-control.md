# GV301QH Direct Fan Control Research

## Hardware

- Product: ROG Flow X13 GV301QH
- Board: GV301QH
- Product family: ROG Flow X13
- Kernel: Linux fedora 7.0.1-cachyos1.fc44.x86_64 #1 SMP PREEMPT Fri Apr 24 20:51:20 UTC 2026 x86_64 GNU/Linux

## ASUS Modules

```text
asus_nb_wmi            32768  0
asus_armoury          118784  0
wmi_bmof               12288  0
firmware_attributes_class    12288  1 asus_armoury
asus_wireless          16384  0
hid_asus               36864  0
asus_wmi              118784  3 asus_armoury,asus_nb_wmi,hid_asus
platform_profile       20480  1 asus_wmi
rfkill                 45056  10 iwlmvm,asus_wmi,bluetooth,cfg80211
sparse_keymap          12288  1 asus_wmi
video                  81920  4 asus_wmi,amdgpu,asus_nb_wmi,nvidia_modeset
wmi                    40960  4 video,asus_armoury,asus_wmi,wmi_bmof
```

## hwmon Fan Interfaces

```text
/sys/class/hwmon/hwmon0: AC0
/sys/class/hwmon/hwmon1: acpitz
/sys/class/hwmon/hwmon2: BAT0
/sys/class/hwmon/hwmon3: hid_0018:04F3:2C82.0005_battery
/sys/class/hwmon/hwmon4: nvme
/sys/class/hwmon/hwmon5: amdgpu
/sys/class/hwmon/hwmon6: k10temp
/sys/class/hwmon/hwmon7: asus
-r--r--r--. 1 nobody nobody 4096 May 19 16:32 /sys/class/hwmon/hwmon7/fan1_input
-r--r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon7/fan1_label
-r--r--r--. 1 nobody nobody 4096 May 19 16:32 /sys/class/hwmon/hwmon7/fan2_input
-r--r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon7/fan2_label
-rw-r--r--. 1 nobody nobody 4096 May 19 16:32 /sys/class/hwmon/hwmon7/pwm1_enable
-rw-r--r--. 1 nobody nobody 4096 May 19 16:32 /sys/class/hwmon/hwmon7/pwm2_enable
/sys/class/hwmon/hwmon8: asus_custom_fan_curve
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point1_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point1_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point2_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point2_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point3_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point3_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point4_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point4_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point5_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point5_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point6_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point6_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point7_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point7_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point8_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_auto_point8_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm1_enable
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point1_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point1_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point2_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point2_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point3_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point3_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point4_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point4_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point5_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point5_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point6_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point6_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point7_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point7_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_auto_point8_temp
-rw-r--r--. 1 nobody nobody 4096 May 19 10:21 /sys/class/hwmon/hwmon8/pwm2_enable
/sys/class/hwmon/hwmon9: iwlwifi_1
```

## Initial Conclusion

- Direct `pwmN` files present: no
- `fanN_target` files present: no
- Custom fan curve files present: yes

## Direct PWM Search

```text
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/fan1_input
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/fan1_label
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/fan2_input
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/fan2_label
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/pwm1_enable
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon7/pwm2_enable
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point1_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point1_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point2_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point2_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point3_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point3_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point4_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point4_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point5_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point5_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point6_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point6_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point7_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point7_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point8_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_auto_point8_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm1_enable
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point1_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point1_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point2_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point2_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point3_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point3_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point4_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point4_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point5_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point5_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point6_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point6_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point7_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point7_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point8_pwm
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_auto_point8_temp
/sys/devices/platform/asus-nb-wmi/hwmon/hwmon8/pwm2_enable
```

```text
The exact command `find /sys/class/hwmon -type f 2>/dev/null | sort | rg '/pwm[0-9]+$|/fan[0-9]+_target$|/fan[0-9]+_input$|/pwm[0-9]+_enable$'` produced no output.
```

Conclusion:

- Direct duty interface found: no
- Target RPM interface found: no
- Only custom-curve interface found: yes

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

Extraction/decompile notes:

```text
acpixtract -a gv301qh-acpi.dat succeeded and extracted DSDT plus SSDT tables into /tmp/gv301qh-acpi.

iasl -d dsdt.dat ssdt*.dat produced DSL output for DSDT and most SSDT files. One SSDT hit
AE_ALREADY_EXISTS for \_SB.PCI0.GP17.XHC0._S0W, but the fan/WMI search still ran against the
generated *.dsl files.
```

Search output:

```text
/tmp/gv301qh-acpi/ssdt4.dsl:1311:            CreateDWordField (GPSP, 0x10, SFAN)
/tmp/gv301qh-acpi/dsdt.dsl:6384:                    Return (SFUN ())
/tmp/gv301qh-acpi/dsdt.dsl:6507:                    If ((IIA0 == 0x00110024))
/tmp/gv301qh-acpi/dsdt.dsl:6529:                    If ((IIA0 == 0x00110025))
/tmp/gv301qh-acpi/dsdt.dsl:6759:                    If ((IIA0 == 0x00110013))
/tmp/gv301qh-acpi/dsdt.dsl:6766:                    If ((IIA0 == 0x00110014))
/tmp/gv301qh-acpi/dsdt.dsl:6868:                    If ((IIA0 == 0x00110022))
/tmp/gv301qh-acpi/dsdt.dsl:6875:                    If ((IIA0 == 0x00110023))
/tmp/gv301qh-acpi/dsdt.dsl:6980:                    If ((IIA0 == 0x00110024))
/tmp/gv301qh-acpi/dsdt.dsl:6985:                    If ((IIA0 == 0x00110025))
/tmp/gv301qh-acpi/dsdt.dsl:7374:                    If ((IIA0 == 0x00110013))
/tmp/gv301qh-acpi/dsdt.dsl:7390:                    If ((IIA0 == 0x00110014))
/tmp/gv301qh-acpi/dsdt.dsl:7427:                    If ((IIA0 == 0x00110022))
/tmp/gv301qh-acpi/dsdt.dsl:7441:                    If ((IIA0 == 0x00110023))
/tmp/gv301qh-acpi/dsdt.dsl:7987:            Method (SFUN, 0, NotSerialized)
/tmp/gv301qh-acpi/dsdt.dsl:8755:            MFUN,   16,
/tmp/gv301qh-acpi/dsdt.dsl:8756:            SFUN,   16,
/tmp/gv301qh-acpi/dsdt.dsl:8772:            MFUN = 0x30
/tmp/gv301qh-acpi/dsdt.dsl:8773:            SFUN = One
/tmp/gv301qh-acpi/dsdt.dsl:11534:        Name (FANS, Zero)
/tmp/gv301qh-acpi/dsdt.dsl:11584:        Method (RFAN, 1, NotSerialized)
/tmp/gv301qh-acpi/dsdt.dsl:11599:                    FANS = Local0
/tmp/gv301qh-acpi/dsdt.dsl:11603:                    Local0 = FANS /* \_TZ_.FANS */
```

Relevant DSDT context:

```text
WMNB dispatch includes 0x4E554653 -> SFUN().
SFUN() returns 0x21.

0x00110013 and 0x00110014:
- DSTS/read path uses EC0.RTAH(0/1), then divides by 100 before returning.
- DEVS/write path toggles a bit through EC0.RRAM/WRAM, not a direct duty value.

0x00110022 and 0x00110023:
- DSTS/read path calls EC0.STCC(0, 0xD3) and EC0.STCC(0, 0xDB).
- DEVS/write path splits IIA1 into low/high bytes and writes EC0.STCD offsets 0xD2/0xD1
  for CPU and 0xDA/0xD9 for GPU, then enables control bits at 0xD0/0xD8.

0x00110024 and 0x00110025:
- DSTS/read path maps point index 0/1/2 to GDFC selectors 0x80..0x82 and 0x84..0x86.
- DEVS/write path calls EC0.SUFC(..., 0x40) and EC0.SUFC(..., 0x44), matching fan curve data writes.

No AGFN symbol was found in the decompiled ACPI tables.
```

Conclusion:

- AGFN present: no
- Fan curve device IDs present: yes, 0x00110024 and 0x00110025
- Fan direct-control candidate present: inconclusive/weak. 0x00110022 and 0x00110023 look like two-byte EC writes for CPU/GPU fan-related data, but the currently loaded kernel exposes them only as fan RPM telemetry/control plumbing through `asus-wmi`, not as standard direct `pwmN` or `fanN_target` sysfs files on this machine.

## Linux Kernel ASUS WMI Notes

Sources reviewed:

- Upstream Linux `drivers/platform/x86/asus-wmi.c`: https://codebrowser.dev/linux/linux/drivers/platform/x86/asus-wmi.c.html
- Upstream raw Linux `drivers/platform/x86/asus-wmi.c`: https://raw.githubusercontent.com/torvalds/linux/master/drivers/platform/x86/asus-wmi.c
- Linux hwmon sysfs ABI: https://www.kernel.org/doc/html/latest/hwmon/sysfs-interface.html
- ASUS Linux custom fan curves FAQ: https://asus-linux.org/faq/asusctl/custom-fan-curves/

Relevant upstream functions:

- `asus_wmi_has_agfn_fan`
- `asus_agfn_fan_speed_write`
- `asus_fan_set_auto`
- `pwm1_store`
- `pwm1_enable_store`
- `fan2_input_show`
- `pwm2_enable_store`
- `asus_wmi_fan_init`
- `asus_wmi_custom_fan_curve_init`

Observed constraints:

- AGFN direct fan write is exposed as: standard hwmon `pwm1` through `DEVICE_ATTR_RW(pwm1)`, but only when `fan_type == FAN_TYPE_AGFN`.
- AGFN direct write supports fan IDs: `0` for auto restore and `1` for first fan speed. The AGFN path rejects other fan IDs, so it does not provide an upstream GPU fan duty primitive.
- AGFN write value shape: `0..255` duty-like speed value, cached as `agfn_pwm`.
- Modern CPU/GPU fan support is exposed as: `FAN_TYPE_SPEC83`, using `ASUS_WMI_DEVID_CPU_FAN_CTRL`, `ASUS_WMI_DEVID_GPU_FAN_CTRL`, and optional middle fan control.
- Modern CPU/GPU `pwm*_enable` support accepts standard hwmon mode values for full speed and automatic mode. It does not expose a `pwm2` duty file.
- `pwm1` sysfs visibility is explicitly gated on `FAN_TYPE_AGFN`; if the machine initializes as `FAN_TYPE_SPEC83`, `pwm1` is hidden even when `fan1_input` and `pwm1_enable` are visible.
- `asus_wmi_fan_init` prefers `FAN_TYPE_SPEC83` when `ASUS_WMI_DEVID_CPU_FAN_CTRL` is present, then falls back to AGFN only if SPEC83 is not present and `asus_wmi_has_agfn_fan()` succeeds.
- Custom fan curve support is a separate hwmon device named `asus_custom_fan_curve` and uses `ASUS_WMI_DEVID_CPU_FAN_CURVE`, `ASUS_WMI_DEVID_GPU_FAN_CURVE`, and optional middle fan curve devices.
- The Linux hwmon ABI identifies `pwm[1-*]` as the direct PWM control file and `fan[1-*]_target` as desired fan speed. Neither exists on this GV301QH system.

Conclusion:

- Existing upstream interface is enough: no, not for G-Helper-like direct duty control of both CPU and GPU fans.

- Existing upstream interface is enough for supported control: yes, for RPM telemetry, fullspeed/auto toggling, and custom fan curves.
- Kernel patch likely required: yes, if the goal is exposing a direct duty or target-RPM primitive. A patch would need to prove what `0x00110022`/`0x00110023` accept on GV301QH and map it to a safe userspace ABI; current upstream does not expose such a primitive.

## Probe Gate

Telemetry check:

```text
/sys/class/hwmon/hwmon7/fan1_input: 5600
/sys/class/hwmon/hwmon7/fan2_input: 5400

acpitz /sys/class/hwmon/hwmon1/temp1_input 68000
nvme /sys/class/hwmon/hwmon4/temp1_input 38850
nvme /sys/class/hwmon/hwmon4/temp2_input 50850
nvme /sys/class/hwmon/hwmon4/temp3_input 38850
amdgpu /sys/class/hwmon/hwmon5/temp1_input 60000
k10temp /sys/class/hwmon/hwmon6/temp1_input 68750
iwlwifi_1 /sys/class/hwmon/hwmon9/temp1_input 52000
```

- ACPI/WMI table contains AGFN or a fan direct-control candidate: no AGFN; weak/inconclusive candidates at `0x00110022` and `0x00110023`
- Kernel source has a matching read path: yes for SPEC83 RPM telemetry and fullspeed/auto mode; no upstream direct duty read/write ABI for this machine
- Existing sysfs does not already expose direct control: yes
- Probe can be read-only first: no clear read-only direct-duty primitive exists beyond already exposed RPM/curve reads
- Probe can restore auto mode after write: partially, `pwm*_enable` can request automatic mode, but no safe direct duty write path is exposed to pair with it
- Current RPM and temperature telemetry remain readable: yes

Decision:

- Proceed to read-only probe: no
- Stop research: yes

## G-Helper `source.cs` AsusSAIO Findings

After reviewing the locally provided `source.cs`, the G-Helper experimental direct fan path is not the same as the normal ASUS ACPI `DEVS/DSTS` fan curve path.

`source.cs` uses the Windows `AsusSAIO` driver and `AsusWinIO64.dll`:

```text
Device path: \\.\AsusSAIO
IOCTL/control code: 2148540528 decimal / 0x80102070 hex
Input buffer size: 16 bytes
Output buffer size: 16 bytes
Output selector byte: input[10]
Return byte used by code: output[11]
```

Observed fan protocol from `source.cs`:

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
  percent = clamp(percent, 0..100)
  value = 255 * percent / 100
  DD 03 82 35 <value>
```

Interpretation:

- This is direct manual fan control in the Windows `AsusSAIO` stack.
- The write value is duty-like `0..255`, derived from a user percentage.
- It selects CPU/GPU fan separately before writing speed.
- It reads RPM as two one-byte reads combined into a 16-bit value.
- It does not use `0x00110022` / `0x00110023`.
- It does not set a target RPM directly.

Correction to earlier ACPI candidate interpretation:

- `0x00110022` and `0x00110023` are still fan-related ACPI/WMI IDs, but they are not the `source.cs` direct fan speed path.
- The likely direct-control target is now the AsusSAIO command protocol (`BB 01 50`, `DD 03 82 31/32/35`, `DD 03 02 33/34`), not the standard `asus-wmi` `DEVS/DSTS` fan curve/range path.
- Linux does not expose `\\.\AsusSAIO` or `AsusWinIO64.dll`, so the key unknown is whether the same low-level protocol is reachable through Linux ACPI/WMI/EC mechanisms without porting the Windows driver.

## Linux SAIO Exposure Search

The planned `/dev` and `/sys` search found no `saio`, `asus*win`, or `winio` device nodes. The planned WMI metadata search under `/sys/bus/wmi/devices` also produced no file output.

Loaded module state includes `asus_wmi`, `asus_nb_wmi`, `asus_armoury`, `wmi`, and `wmi_bmof`. It does not include `acpi_call` or `ec_sys`.

Current hwmon telemetry remains readable:

```text
/sys/class/hwmon/hwmon7/fan1_input=3600
/sys/class/hwmon/hwmon7/fan2_input=3400
/sys/class/hwmon/hwmon1/temp1_input=59000
/sys/class/hwmon/hwmon4/temp1_input=38850
/sys/class/hwmon/hwmon4/temp2_input=50850
/sys/class/hwmon/hwmon4/temp3_input=38850
/sys/class/hwmon/hwmon5/temp1_input=56000
/sys/class/hwmon/hwmon6/temp1_input=59500
/sys/class/hwmon/hwmon9/temp1_input=51000
```

Conclusion:

- Native `AsusSAIO` device present: no
- Generic raw WMI call interface present: no obvious safe sysfs file interface found by the planned search
- Existing kernel driver exposes SAIO command protocol: no evidence
- Safe read-only transport identified: no

## SAIO ACPI Symbol Search

The ACPI dump was extracted and decompiled under `/tmp/gv301qh-acpi`. `acpixtract -a gv301qh-acpi.dat` succeeded. `iasl -d *.dat` failed because it included the aggregate `gv301qh-acpi.dat`; `iasl -d dsdt.dat ssdt*.dat` produced the needed DSL files, with one SSDT reporting `AE_ALREADY_EXISTS`.

```text
SAIO/WinIO narrow search:
rg -n 'SAIO|WinIO|AsusSAIO|0xBB, 0x01, 0x50|0xDD, 0x03|0xDD,\s*$' /tmp/gv301qh-acpi/*.dsl

/tmp/gv301qh-acpi/dsdt.dsl:6339:                        0xDD,
```

The `0xDD` hit is an element in a contiguous byte list (`0xD2` through `0xDF`, then `0xC0`...), not the SAIO `DD 03 ...` command protocol. Other broad byte hits are similarly noisy buffer or table data. `WMNB`, `ECAC`, `ISMI`, `WEBC`, `REBC`, `STCD`, and `STCC` are present for the existing ASUS ACPI/WMI paths, including the previously known fan-related IDs, but no SAIO-specific symbol or exact command sequence was found.

Conclusion:

- SAIO command bytes present in ACPI tables: no exact protocol sequence found
- SAIO commands map to `WMNB`: no evidence
- SAIO commands map to EC helper methods: no evidence
- Linux can call the same protocol through existing ACPI method: no evidence

## SAIO Read-Only Probe Gate

- Source protocol documented: yes
- Linux transport exists without new kernel code: no
- Transport can send 16-byte buffer and return 16-byte buffer: no
- Read-only command exists that does not require manual mode: yes in the Windows source protocol, but no Linux transport was found
- RPM/temperature telemetry remains readable: yes
- No raw EC port writes required: yes for documentation-only stop
- No fan speed writes required: yes

Decision:

- Proceed to read-only SAIO probe: no
- Stop at documentation: yes

## SAIO Transport Recommendation

The Windows `AsusSAIO` direct fan protocol is documented, but no safe Linux transport has been identified. Do not implement direct fan control in the daemon or UI. Next work would require kernel-adjacent research to expose a constrained, logged, high-safety transport for the SAIO command protocol.

## Final Recommendation

Direct fan duty or target RPM control is not currently exposed on this GV301QH Linux stack. The application should not claim G-Helper-like direct fan control yet. Supported fan control remains custom curves and dynamic curve updates through `asus-wmi`/`asusd`/`hwmon`.

Do not add Flutter or daemon API for direct fan duty yet. The smallest safe next step is a separate SAIO-focused research plan:

1. Determine whether the `source.cs` AsusSAIO command protocol has a safe Linux transport.
2. Prefer read-only commands first, such as fan count, CPU temperature, and RPM reads.
3. Attempt manual mode and speed writes only after transport semantics and auto-restore behavior are proven.
4. Keep all write probes high-duty only, short-lived, logged, and manually approved.
