# Technical Research: Extended GPU Modes Support

**Feature**: Extended GPU Modes Support  
**Date**: 2026-04-13  
**Status**: Complete

## Overview

This document consolidates technical research for implementing extended GPU modes support (Integrated, Hybrid, Ultimate, AsusMuxDgpu, AsusEgpu, Vfio) based on the supergfxctl implementation.

## 1. GPU Mode Switching - ASUS WMI Interface

### Decision: Use ASUS WMI sysfs interface for basic GPU modes

**Chosen Solution**:
- Basic GPU mode control via `/sys/class/platform/asus-nb-wmi/dgpu_disable`
- Values: `0` (Enable dGPU), `1` (Disable dGPU)
- Kernel requirement: 5.17+ for full support

**Rationale**:
- Direct sysfs access is fastest and most reliable
- No additional daemon dependencies
- Used by G-Helper (Windows) and AsusCtl (Linux) reference implementations
- Well-documented in Linux kernel ABI documentation

**Alternatives Considered**:
- **D-Bus via asusd**: Rejected - asusd provides dgpu_disable property but supergfxctl uses direct sysfs for more control
- **ACPI calls via libacpi**: Rejected - more complex, requires ACPI table parsing

**Implementation Details**:
```rust
// Read current dGPU state
pub fn read_dgpu_state() -> Result<bool, GpuError> {
    let path = "/sys/class/platform/asus-nb-wmi/dgpu_disable";
    let content = fs::read_to_string(path)?;
    let value = content.trim().parse::<u32>()?;
    Ok(value == 0) // 0 = enabled, 1 = disabled
}

// Write dGPU state
pub fn write_dgpu_state(enabled: bool) -> Result<(), GpuError> {
    let path = "/sys/class/platform/asus-nb-wmi/dgpu_disable";
    let value = if enabled { "0" } else { "1" };
    fs::write(path, value)?;
    Ok(())
}
```

**References**:
- Kernel ABI: https://www.kernel.org/doc/html/latest/admin-guide/laptops/asus-wmi.html
- G-Helper GPU: https://github.com/seerge/g-helper/blob/main/app/Mode/GPUMode.cs
- AsusCtl GPU: https://gitlab.com/asus-linux/asusctl/-/blob/main/daemon/src/ctrl_gpu.rs

---

## 2. ASUS MUX Switch Control

### Decision: Use ASUS WMI for MUX toggle with reboot requirement

**Chosen Solution**:
- MUX switch via ASUS-specific WMI calls
- Requires system reboot to take effect
- Hardware detection via sysfs presence check
- User confirmation dialog before reboot

**Rationale**:
- MUX switch physically re-routes display output between iGPU and dGPU
- This is an ACPI-level change that requires reboot
- Supergfxctl implements this with explicit reboot requirement
- Not all ASUS laptops support MUX switch

**Alternatives Considered**:
- **Runtime MUX switching**: Rejected - hardware limitation, not supported on most models
- **AsusCtl integration**: Rejected - want direct control without dependency

**Implementation Details**:
```rust
// Detect MUX support
pub fn has_mux_support() -> bool {
    Path::new("/sys/class/platform/asus-nb-wmi/mux").exists()
}

// Toggle MUX (requires reboot)
pub fn toggle_mux(use_dgpu_primary: bool) -> Result<(), GpuError> {
    let path = "/sys/class/platform/asus-nb-wmi/mux";
    let value = if use_dgpu_primary { "1" } else { "0" };
    fs::write(path, value)?;
    // Signal that reboot is required
    Ok(())
}
```

**Hardware Behavior**:
- MUX off: Display always connected to iGPU, dGPU renders via PRIME (copy)
- MUX on: Display connected directly to dGPU, iGPU can be disabled
- Reboot required because display routing is physical

**References**:
- Supergfxctl README: https://gitlab.com/asus-linux/supergfxctl
- Arch Wiki MUX: https://wiki.archlinux.org/title/ASUS_laptops#GPU_Mux

---

## 3. External GPU (AsusEgpu/XG Mobile) Support

### Decision: Use ASUS-specific eGPU detection and switching

**Chosen Solution**:
- External GPU detection via PCI enumeration and ASUS-specific sysfs
- AsusEgpu mode for ASUS Flow series and XG Mobile devices
- Auto-detection of external GPU connection
- Error handling when eGPU disconnected

**Rationale**:
- ASUS Flow series has special eGPU support via dedicated hardware
- XG Mobile uses Thunderbolt connection with ASUS-specific protocol
- Supergfxctl automatically detects and shows AsusEgpu mode when supported
- Different from generic Thunderbolt eGPU solutions

**Alternatives Considered**:
- **Generic Thunderbolt eGPU**: Rejected - ASUS has specific hardware support
- **AsusCtl integration**: Rejected - want direct control

**Implementation Details**:
```rust
// Detect eGPU support
pub fn has_egpu_support() -> bool {
    // Check for ASUS Flow series or XG Mobile support
    Path::new("/sys/class/platform/asus-nb-wmi/egpu_enable").exists()
}

// Find hotplug power slot for a PCI device
// CRITICAL: Iterates through /sys/bus/pci/slots/ and reads "address" file
// to match against device address (e.g., "0000:01:00.0")
pub fn find_slot_power(device_address: &str) -> Result<PathBuf, GpuError> {
    let slots_path = Path::new("/sys/bus/pci/slots");
    
    for entry in slots_path.read_dir()? {
        let slot_path = entry?.path();
        let address_file = slot_path.join("address");
        
        if let Ok(slot_address) = fs::read_to_string(&address_file) {
            // Compare device address with slot address
            if device_address.contains(slot_address.trim()) {
                let power_path = slot_path.join("power");
                if power_path.exists() {
                    return Ok(power_path);
                }
            }
        }
    }
    
    Err(GpuError::HotplugSlotNotFound)
}

// Detect eGPU connection status
// eGPU devices have hotplug capability (removable PCI slots)
pub fn is_egpu_connected() -> bool {
    if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
        for entry in entries.flatten() {
            let device_path = entry.path();
            
            // Check if this is a GPU (class 0x030000 or 0x030200)
            let vendor_path = device_path.join("vendor");
            let class_path = device_path.join("class");
            
            if let (Ok(vendor_str), Ok(class_str)) = (
                fs::read_to_string(&vendor_path),
                fs::read_to_string(&class_path)
            ) {
                let vendor_id = vendor_str.trim();
                let class_code = class_str.trim();
                
                // GPU: NVIDIA (0x10de), AMD (0x1002), Intel (0x8086)
                if (vendor_id == "0x10de" || vendor_id == "0x1002" || vendor_id == "0x8086")
                    && (class_code == "0x030000" || class_code == "0x030200")
                {
                    // Get device address
                    let device_name = device_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    
                    // Check if device has hotplug slot (indicates removable/external)
                    if find_slot_power(&device_name).is_ok() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// Enable eGPU mode
pub fn enable_egpu_mode() -> Result<(), GpuError> {
    if !is_egpu_connected() {
        return Err(GpuError::NoEgpuConnected);
    }
    let path = "/sys/class/platform/asus-nb-wmi/egpu_enable";
    fs::write(path, "1")?;
    Ok(())
}
```

**Hardware Models**:
- ASUS Flow series (13" Flow with eGPU support)
- XG Mobile (external GPU dock)
- Auto-detection based on hardware capabilities

**Critical Bug Fix**:
The initial implementation incorrectly used PCI device address (e.g., `0000:01:00.0`) as hotplug slot name.
Correct approach: iterate `/sys/bus/pci/slots/`, read each slot's `address` file, match with device address.

**References**:
- Supergfxctl AsusEgpu: https://gitlab.com/asus-linux/supergfxctl
- Supergfxctl source (find_slot_power): https://gitlab.com/asus-linux/supergfxctl/-/blob/main/src/lib.rs
- ASUS XG Mobile: https://www.asus.com/Monitors/XG-Mobile/

---

## 3.1. Alternative: Use supergfxctl via D-Bus

### Decision: Delegate to existing supergfxctl daemon instead of reimplementing

**Chosen Solution**:
- supergfxctl is a mature daemon for ASUS GPU mode switching
- Provides D-Bus interface `org.supergfxctl.Daemon`
- Already handles all GPU modes including AsusEgpu (XG Mobile)
- No need to reimplement complex PCI enumeration and hotplug detection

**Rationale**:
- supergfxctl is the reference implementation for ASUS GPU switching
- Well-tested on ASUS Flow series and XG Mobile
- Handles edge cases (driver conflicts, logout/reboot requirements)
- D-Bus interface provides signals for mode change notifications
- Reduces code complexity and maintenance burden

**D-Bus Interface**:
- **Bus name**: `org.supergfxctl.Daemon`
- **Object path**: `/org/supergfxctl/Gfx`
- **Interface**: `org.supergfxctl.Daemon`

**Methods**:
```
Mode() -> y                    # Get current GPU mode (0-6)
Supported() -> ay              # List of supported modes
Vendor() -> s                  # GPU vendor (Nvidia/AMD)
Power() -> y                   # Power status
SetMode(y) -> y                # Set mode, returns UserActionRequired
PendingMode() -> y             # Pending mode if reboot needed
```

**GpuMode enum (D-Bus byte values)**:
```
0 = Hybrid
1 = Integrated  
2 = NvidiaNoModeset
3 = Vfio
4 = AsusEgpu          # ← XG Mobile mode
5 = AsusMuxDgpu
6 = None
```

**UserActionRequired enum**:
```
0 = Logout
1 = Reboot
2 = SwitchToIntegrated
3 = AsusEgpuDisable
4 = Nothing
```

**Implementation Details**:
```dart
// lib/services/supergfx_client.dart
import 'package:dbus/dbus.dart';

class SupergfxClient {
  static const String _busName = 'org.supergfxctl.Daemon';
  static const String _objectPath = '/org/supergfxctl/Gfx';
  static const String _interface = 'org.supergfxctl.Daemon';

  final DBusClient _client = DBusClient.system();
  late final DBusRemoteObject _remote;

  SupergfxClient() {
    _remote = DBusRemoteObject(
      _client,
      name: _busName,
      path: DBusObjectPath(_objectPath),
    );
  }

  Future<int> getMode() async {
    final result = await _remote.callMethod(
      _interface, 'Mode', [],
      replySignature: DBusSignature('y'),
    );
    return result.returnValues[0].asByte();
  }

  Future<List<int>> getSupportedModes() async {
    final result = await _remote.callMethod(
      _interface, 'Supported', [],
      replySignature: DBusSignature('ay'),
    );
    return (result.returnValues[0] as DBusArray)
        .children.map((v) => v.asByte()).toList();
  }

  Future<int> setMode(int mode) async {
    final result = await _remote.callMethod(
      _interface, 'SetMode',
      [DBusByte(mode)],
      replySignature: DBusSignature('y'),
    );
    return result.returnValues[0].asByte(); // UserActionRequired
  }
}
```

**Advantages**:
- ✅ No reimplementation of complex PCI enumeration
- ✅ Proven XG Mobile support on ASUS Flow series
- ✅ Handles driver conflicts and logout/reboot requirements
- ✅ D-Bus signals for real-time mode change notifications
- ✅ polkit integration for authorization
- ✅ Active maintenance by asus-linux community

**Disadvantages**:
- ❌ External dependency (supergfxctl must be installed)
- ❌ D-Bus dependency (already used for asusd in this project)
- ❌ Less control over low-level implementation

**Decision**: Recommend using supergfxctl D-Bus for production, keep Rust implementation for learning/fallback.

**References**:
- supergfxctl D-Bus interface: https://gitlab.com/asus-linux/supergfxctl/-/blob/main/src/zbus_iface.rs
- asus-linux docs: https://asus-linux.org/

---

## 4. VFIO GPU Passthrough

### Decision: Use Linux VFIO subsystem for GPU passthrough

**Chosen Solution**:
- Bind dGPU to vfio-pci driver for VM passthrough
- Verify vfio kernel modules availability
- Unbind from graphics driver, bind to vfio-pci
- Rebind to graphics driver when switching out of VFIO mode

**Rationale**:
- Standard Linux mechanism for GPU passthrough to VMs
- Supergfxctl implements Vfio mode for this use case
- Requires kernel configuration (vfio modules as separate modules, not built-in)
- Advanced feature for virtualization enthusiasts

**Alternatives Considered**:
- **NVIDIA vGPU**: Rejected - proprietary, requires license
- **Custom kernel module**: Rejected - unnecessary complexity

**Implementation Details**:
```rust
// Check vfio availability
pub fn has_vfio_support() -> bool {
    Path::new("/sys/module/vfio_pci").exists()
}

// Bind dGPU to vfio
pub fn bind_dgpu_to_vfio(pci_id: &str) -> Result<(), GpuError> {
    let driver_path = format!("/sys/bus/pci/devices/{}/driver", pci_id);
    let vfio_path = "/sys/bus/pci/drivers/vfio-pci";
    
    // Unbind from current driver
    fs::write(format!("{}/unbind", driver_path), pci_id)?;
    
    // Bind to vfio-pci
    fs::write(format!("{}/bind", vfio_path), pci_id)?;
    
    Ok(())
}

// Rebind to graphics driver
pub fn rebind_dgpu_to_graphics(pci_id: &str, driver: &str) -> Result<(), GpuError> {
    let driver_path = format!("/sys/bus/pci/drivers/{}", driver);
    fs::write(format!("{}/bind", driver_path), pci_id)?;
    Ok(())
}
```

**Kernel Requirements**:
- vfio-pci module must be loaded
- vfio modules must be separate (not built into kernel)
- IOMMU must be enabled in kernel cmdline

**References**:
- VFIO documentation: https://www.kernel.org/doc/html/latest/driver-api/vfio.html
- Supergfxctl Vfio: https://gitlab.com/asus-linux/supergfxctl
- Arch Wiki VFIO: https://wiki.archlinux.org/title/PCI_passthrough_via_OVMF

---

## 5. GPU Monitoring - Nvidia NVML

### Decision: Use nvml-wrapper crate for Nvidia GPU monitoring

**Chosen Solution**:
- nvml-wrapper crate for safe Rust bindings to libnvidia-ml.so
- Read GPU temperature, utilization, memory usage
- Handle Nvidia driver absence gracefully

**Rationale**:
- NVML is official Nvidia API
- nvml-wrapper provides safe Rust bindings
- Most reliable for Nvidia GPUs
- Used by reference implementations

**Alternatives Considered**:
- **nvidia-smi parsing**: Rejected - fragile, parsing text output
- **Direct NVML C bindings**: Rejected - unsafe, manual FFI

**Implementation Details**:
```rust
use nvml_wrapper::NVML;

pub struct NvidiaGpuMonitor {
    nvml: NVML,
}

impl NvidiaGpuMonitor {
    pub fn new() -> Result<Self, GpuError> {
        let nvml = NVML::init()?;
        Ok(Self { nvml })
    }
    
    pub fn get_temperature(&self) -> Result<u32, GpuError> {
        let device = self.nvml.device_by_index(0)?;
        let temp = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)?;
        Ok(temp)
    }
    
    pub fn get_utilization(&self) -> Result<u32, GpuError> {
        let device = self.nvml.device_by_index(0)?;
        let util = device.utilization_rates()?;
        Ok(util.gpu)
    }
}
```

**References**:
- nvml-wrapper: https://github.com/rust-nvml/nvml-wrapper
- NVML docs: https://docs.nvidia.com/deploy/nvml-api/

---

## 6. GPU Monitoring - AMD sysfs

### Decision: Use AMD DRM sysfs interfaces for AMD GPU monitoring

**Chosen Solution**:
- Direct sysfs reads from `/sys/class/drm/card0/device/`
- Temperature: `hwmon/hwmonX/temp1_input`
- Utilization: `gpu_busy_percent` (if available)
- Power profile: `pp_power_profile_mode`

**Rationale**:
- AMD has no stable library API like NVML
- sysfs is standard approach for AMD GPUs
- Well-documented in kernel documentation
- No additional dependencies

**Alternatives Considered**:
- **ROCm**: Rejected - heavyweight, focused on compute not monitoring
- **libdrm bindings**: Rejected - low-level, complex

**Implementation Details**:
```rust
pub struct AmdGpuMonitor {
    card_path: PathBuf,
}

impl AmdGpuMonitor {
    pub fn new() -> Result<Self, GpuError> {
        let card_path = Self::find_amd_card()?;
        Ok(Self { card_path })
    }
    
    pub fn get_temperature(&self) -> Result<u32, GpuError> {
        let hwmon_path = self.find_hwmon()?;
        let temp_path = hwmon_path.join("temp1_input");
        let content = fs::read_to_string(temp_path)?;
        let temp_millidegrees: u32 = content.trim().parse()?;
        Ok(temp_millidegrees / 1000)
    }
    
    fn find_amd_card() -> Result<PathBuf, GpuError> {
        // Search /sys/class/drm for AMD card
        Ok(PathBuf::from("/sys/class/drm/card0"))
    }
}
```

**References**:
- AMD GPU thermal: https://docs.kernel.org/gpu/amdgpu/thermal.html
- AMD sysfs: https://wiki.archlinux.org/title/AMDGPU

---

## 7. GPU Mode Mapping

### Decision: Map supergfxctl modes to internal representation

**Chosen Solution**:
- Integrated → iGPU only, dGPU disabled
- Hybrid → dGPU-offload mode (PRIME)
- Ultimate → dGPU primary for rendering
- AsusMuxDgpu → MUX switch with dGPU as primary
- AsusEgpu → External GPU mode
- Vfio → GPU bound to vfio-pci for VM passthrough

**Rationale**:
- Supergfxctl is the reference implementation
- These modes cover all ASUS GPU switching scenarios
- Mapping preserves semantics while using our own implementation

**Mode Requirements**:
- Integrated: dgpu_disable=1, no MUX change
- Hybrid: dgpu_disable=0, MUX off (if available)
- Ultimate: dgpu_disable=0, MUX off (if available), dGPU always used
- AsusMuxDgpu: MUX on, dgpu_disable=0, requires reboot
- AsusEgpu: egpu_enable=1, requires eGPU hardware
- Vfio: bind dGPU to vfio-pci, requires vfio modules

---

## 8. Hardware Detection Strategy

### Decision: Multi-layer detection for GPU capabilities

**Chosen Solution**:
1. Detect dGPU presence via PCI enumeration
2. Detect dGPU vendor (Nvidia/AMD) via PCI vendor ID
3. Detect MUX support via sysfs presence check
4. Detect eGPU support via ASUS-specific sysfs
5. Detect vfio support via module availability check
6. Detect external GPU connection via PCI/Thunderbolt

**Rationale**:
- Not all modes available on all hardware
- UI must adapt to hardware capabilities
- Graceful degradation for missing features
- Supergfxctl uses similar detection strategy

**Implementation Details**:
```rust
pub struct GpuCapabilities {
    pub has_dgpu: bool,
    pub dgpu_vendor: Option<GpuVendor>,
    pub has_mux_switch: bool,
    pub supports_egpu: bool,
    pub external_gpu_connected: bool,
    pub vfio_available: bool,
}

pub fn detect_gpu_capabilities() -> GpuCapabilities {
    GpuCapabilities {
        has_dgpu: detect_dgpu(),
        dgpu_vendor: detect_dgpu_vendor(),
        has_mux_switch: Path::new("/sys/class/platform/asus-nb-wmi/mux").exists(),
        supports_egpu: Path::new("/sys/class/platform/asus-nb-wmi/egpu_enable").exists(),
        external_gpu_connected: detect_egpu_connection(),
        vfio_available: Path::new("/sys/module/vfio_pci").exists(),
    }
}
```

**References**:
- Supergfxctl detection: https://gitlab.com/asus-linux/supergfxctl
- PCI vendor IDs: https://pci-ids.ucw.cz/

---

## 9. Error Handling and User Feedback

### Decision: Descriptive error messages for all failure modes

**Chosen Solution**:
- Result<T, String> for all fallible operations
- User-facing error messages (no internal details)
- Troubleshooting guidance for common issues
- Detection of conflicts with other GPU switchers

**Rationale**:
- Constitution Principle V requires actionable error messages
- GPU switching can fail in many ways
- Users need guidance to resolve issues
- Conflicts with other tools (optimus-manager, supergfxctl) must be detected

**Error Scenarios**:
- Permission denied → udev rules not installed
- Hardware not supported → model doesn't have feature
- Driver conflicts → other GPU switcher active
- Kernel too old → upgrade required
- vfio not available → modules not loaded
- eGPU disconnected → physical connection issue

**References**:
- Constitution Principle V: Least Privilege Security Model
- Supergfxctl error handling: https://gitlab.com/asus-linux/supergfxctl

---

## 10. Configuration Persistence

### Decision: JSON config for GPU mode preferences

**Chosen Solution**:
- Store GPU mode preference in `~/.config/asus-linux-control/config.json`
- Add `gpu_mode` field to existing config structure
- Support kernel parameter override (supergfxd.mode=)
- Auto-switch based on AC/battery state (optional)

**Rationale**:
- Consistent with existing project pattern
- XDG Base Directory specification compliance
- Supergfxctl supports kernel parameter override
- Automation is a common use case

**Implementation Details**:
```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gpu_mode: Option<GpuMode>,
    pub auto_gpu_switch: bool,
    // ... existing fields
}

pub fn save_gpu_mode(mode: GpuMode) -> Result<(), ConfigError> {
    let mut config = load_config()?;
    config.gpu_mode = Some(mode);
    save_config(&config)
}
```

**References**:
- XDG Base Directory: https://specifications.freedesktop.org/basedir-spec/
- Supergfxctl config: https://asus-linux.org/manual/supergfxctl-manual/

---

## Summary

All technical decisions are based on:
1. Supergfxctl reference implementation (primary source)
2. Linux kernel documentation (sysfs interfaces)
3. Existing project patterns (Flutter + Rust FFI)
4. Constitution principles (safety, modularity, security)

No NEEDS CLARIFICATION items remain. All technical approaches are well-documented and have proven reference implementations.
