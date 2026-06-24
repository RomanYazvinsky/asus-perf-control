# SAIO Direct Fan Test Log

## Protocol Source

Source file: `source.cs`

```text
10:internal class AsusSAIO : DeviceACPI
12:  private const string FILE_NAME = "\\\\.\\\\AsusSAIO";
13:  private const uint CONTROL_CODE = 2148540528;
25:      using (ManagementObjectSearcher managementObjectSearcher = new ManagementObjectSearcher("Select * from Win32_SystemDriver WHERE Name='AsusSAIO'"))
41:  public AsusSAIO()
45:      this.Connect("\\\\.\\\\AsusSAIO");
50:      string saioPath = AsusSAIO.GetSAIOPath();
55:        ProcessHelper.StartEnableService(nameof (AsusSAIO));
57:        this.Connect("\\\\.\\\\AsusSAIO");
71:  protected byte CallMethod(byte[] args, byte output = 0, string? log = "SAIO")
77:    this.Control(2148540528U, lpInBuffer, lpOutBuffer);
83:  public int GetFanCount()
85:    this.FanInit();
86:    return (int) this.CallMethod(new byte[4]
88:      (byte) 221,
95:  public int GetCPUTemp()
97:    return (int) this.CallMethod(new byte[3]
99:      (byte) 221,
105:  public int GetFanSpeed(AsusFan fanId = AsusFan.CPU)
107:    this.FanInit();
108:    this.FanSelect(fanId);
109:    return 256 /*0x0100*/ * (int) this.CallMethod(new byte[4]
111:      (byte) 221,
115:    }, (byte) 1) + (int) this.CallMethod(new byte[4]
117:      (byte) 221,
124:  public void FanInit()
126:    int num = (int) this.CallMethod(new byte[3]
128:      (byte) 187,
134:  private void FanSelect(AsusFan fanId)
136:    int num = (int) this.CallMethod(new byte[5]
138:      (byte) 221,
140:      (byte) 130,
146:  public void FanControl(bool manual = true)
148:    this.FanInit();
149:    int num = (int) this.CallMethod(new byte[5]
151:      (byte) 221,
153:      (byte) 130,
159:  public void FanSpeed(AsusFan fanId, float speed = 0.0f)
161:    this.FanInit();
162:    this.FanSelect(fanId);
164:    int num = (int) this.CallMethod(new byte[5]
166:      (byte) 221,
168:      (byte) 130,
```

Summary:

- `AsusSAIO` uses the Windows device `\\.\AsusSAIO`.
- `CallMethod` sends a 16-byte input buffer through IOCTL decimal `2148540528`, hex `0x80102070`, and reads from output byte index `11`.
- `FanInit` sends `BB 01 50`.
- `GetFanCount` sends `DD 03 02 30`.
- `GetCPUTemp` sends `DD 03 02`.
- `FanSelect(fanId)` sends `DD 03 82 32 <fanId>`.
- `FanControl(manual)` sends `DD 03 82 31 <0|1>`.
- `GetFanSpeed(fanId)` sends `DD 03 02 34` for the high byte and `DD 03 02 33` for the low byte after `FanInit` and `FanSelect`.
- `FanSpeed(fanId, percent)` sends `DD 03 82 35 <value>`, where `value = 255 * clamp(percent, 0..100) / 100`.

## Safety State

- No sudo commands are run by Codex.
- No git-mutating commands are run by Codex.
- No fan speed writes are part of this plan.
- Manual write probes require a separate follow-up plan.

## Linux SAIO Exposure Search

SAIO/WinIO device node search:

```text
Command: find /dev /sys -iname '*saio*' -o -iname '*asus*win*' -o -iname '*winio*' 2>/dev/null
Exit: 1
Output: none
```

WMI device metadata search:

```text
Command: find /sys/bus/wmi/devices -maxdepth 3 -type f 2>/dev/null | sort
Exit: 0
Output: none
```

Loaded ACPI/WMI helper modules:

```text
asus_nb_wmi            32768  0
asus_armoury          118784  0
firmware_attributes_class    12288  1 asus_armoury
wmi_bmof               12288  0
asus_wireless          16384  0
hid_asus               36864  0
asus_wmi              118784  3 asus_armoury,asus_nb_wmi,hid_asus
platform_profile       20480  1 asus_wmi
rfkill                 45056  10 iwlmvm,asus_wmi,bluetooth,cfg80211
sparse_keymap          12288  1 asus_wmi
video                  81920  4 asus_wmi,amdgpu,asus_nb_wmi,nvidia_modeset
wmi                    40960  4 video,asus_armoury,asus_wmi,wmi_bmof
```

Telemetry:

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
- Generic raw WMI call interface present: no obvious file interface found by the planned command
- Existing kernel driver exposes SAIO command protocol: no evidence
- Safe read-only transport identified: no

## SAIO ACPI Symbol Search

ACPI DSL verification:

```text
Command: ls /tmp/gv301qh-acpi/*.dsl
Exit: 0
Output:
/tmp/gv301qh-acpi/apic.dsl
/tmp/gv301qh-acpi/bgrt.dsl
/tmp/gv301qh-acpi/cdit.dsl
/tmp/gv301qh-acpi/crat.dsl
/tmp/gv301qh-acpi/dsdt.dsl
/tmp/gv301qh-acpi/ecdt.dsl
/tmp/gv301qh-acpi/facp.dsl
/tmp/gv301qh-acpi/facs.dsl
/tmp/gv301qh-acpi/fidt.dsl
/tmp/gv301qh-acpi/fpdt.dsl
/tmp/gv301qh-acpi/ssdt1.dsl
/tmp/gv301qh-acpi/ssdt10.dsl
/tmp/gv301qh-acpi/ssdt11.dsl
/tmp/gv301qh-acpi/ssdt12.dsl
/tmp/gv301qh-acpi/ssdt13.dsl
/tmp/gv301qh-acpi/ssdt14.dsl
/tmp/gv301qh-acpi/ssdt15.dsl
/tmp/gv301qh-acpi/ssdt16.dsl
/tmp/gv301qh-acpi/ssdt17.dsl
/tmp/gv301qh-acpi/ssdt2.dsl
/tmp/gv301qh-acpi/ssdt3.dsl
/tmp/gv301qh-acpi/ssdt4.dsl
/tmp/gv301qh-acpi/ssdt5.dsl
/tmp/gv301qh-acpi/ssdt6.dsl
/tmp/gv301qh-acpi/ssdt7.dsl
/tmp/gv301qh-acpi/ssdt8.dsl
/tmp/gv301qh-acpi/ssdt9.dsl
```

Decompile note:

```text
`acpixtract -a gv301qh-acpi.dat` succeeded.
`iasl -d *.dat` failed because it also tried to decompile the aggregate `gv301qh-acpi.dat`.
`iasl -d dsdt.dat ssdt*.dat` created the AML DSL files needed for this search. It returned a non-zero exit because one SSDT hit `AE_ALREADY_EXISTS`, but `dsdt.dsl` and the SSDT DSL files were present afterward.
```

Planned broad protocol search summary:

```text
Command: rg -n '0xBB|0xDD|0x82|0x50|0x31|0x32|0x33|0x34|0x35|SAIO|WinIO|WMNB|ECAC|ISMI|WEBC|REBC|STCD|STCC' /tmp/gv301qh-acpi/*.dsl
Exit: 0
Output:
Output was broad and noisy because the individual byte values appear throughout unrelated buffers and tables.
Relevant symbolic hits include `WMNB`, `ECAC`, `ISMI`, `WEBC`, `REBC`, `STCD`, and `STCC` in `dsdt.dsl`.
No `SAIO`, `WinIO`, or `AsusSAIO` symbols were found.
```

Narrow SAIO pattern search:

```text
Command: rg -n 'SAIO|WinIO|AsusSAIO|0xBB, 0x01, 0x50|0xDD, 0x03|0xDD,\s*$' /tmp/gv301qh-acpi/*.dsl
Exit: 0
Output:
/tmp/gv301qh-acpi/dsdt.dsl:6339:                        0xDD,
```

Exact `0xDD` context:

```text
0xD2,
0xD3,
0xD4,
0xD5,
0xD6,
0xD7,
0xD8,
0xD9,
0xDA,
0xDB,
0xDC,
0xDD,
0xDE,
0xDF,
0xC0,
0xC1,
0xC2,
0xC3,
```

Conclusion:

- SAIO command bytes present in ACPI tables: no exact protocol sequence found
- SAIO commands map to `WMNB`: no evidence
- SAIO commands map to EC helper methods: no evidence
- Linux can call the same protocol through existing ACPI method: no evidence
- Notes: `WMNB`, `STCC`, `STCD`, `WEBC`, and `REBC` are present for the existing ASUS ACPI/WMI paths, including previously known fan-related IDs, but the Windows `AsusSAIO` 16-byte command protocol was not found.

## SAIO Read-Only Probe Gate

- Source protocol documented: yes
- Linux transport exists without new kernel code: no
- Transport can send 16-byte buffer and return 16-byte buffer: no
- Read-only command exists that does not require manual mode: yes in source protocol, but no Linux transport was found
- RPM/temperature telemetry remains readable: yes
- No raw EC port writes required: yes for documentation-only stop
- No fan speed writes required: yes

Decision:

- Proceed to read-only SAIO probe: no
- Stop at documentation: yes

## SAIO Transport Recommendation

The Windows `AsusSAIO` direct fan protocol is documented, but no safe Linux transport has been identified. Do not implement direct fan control in the daemon or UI. Next work would require kernel-adjacent research to expose a constrained, logged, high-safety transport for the SAIO command protocol.
