# Quickstart: Extended GPU Modes Support

**Feature**: Extended GPU Modes Support  
**Date**: 2026-04-13  
**Status**: Complete

## Overview

This guide provides a quick reference for implementing the extended GPU modes feature, including 6 GPU modes (Integrated, Hybrid, Ultimate, AsusMuxDgpu, AsusEgpu, Vfio) based on the supergfxctl implementation.

## Prerequisites

### Hardware Requirements

- ASUS laptop with hybrid graphics (Intel/AMD iGPU + Nvidia/AMD dGPU)
- Linux kernel 6.1+ for full GPU switching support
- For AsusEgpu: ASUS Flow series or XG Mobile device
- For Vfio: vfio kernel modules available

### Software Requirements

- Rust 1.75+
- Dart 3.0+ / Flutter 3.16+
- `nvml-wrapper` crate for Nvidia GPU monitoring
- Existing project structure with Flutter + Rust FFI

### Permissions

- udev rules must be installed (already in `install/99-asus-control.rules`)
- Application runs as normal user (not root)
- D-Bus access for UPower (battery monitoring)

## Implementation Steps

### 1. Rust Backend - GPU Service

Create `rust_core/src/services/gpu.rs`:

```rust
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuMode {
    Integrated,
    Hybrid,
    Ultimate,
    AsusMuxDgpu,
    AsusEgpu,
    Vfio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuVendor {
    Nvidia,
    Amd,
}

#[derive(Debug, Clone, serde::Serialize)]
#[frb(sync)]
pub struct GpuConfiguration {
    pub has_dgpu: bool,
    pub dgpu_vendor: Option<GpuVendor>,
    pub has_mux_switch: bool,
    pub supports_egpu: bool,
    pub external_gpu_connected: bool,
    pub vfio_available: bool,
}

pub struct GpuController {
    asus_wmi_path: String,
}

impl GpuController {
    pub fn new() -> Result<Self, GpuError> {
        Ok(Self {
            asus_wmi_path: "/sys/class/platform/asus-nb-wmi".to_string(),
        })
    }

    pub fn get_capabilities(&self) -> GpuConfiguration {
        GpuConfiguration {
            has_dgpu: self.detect_dgpu(),
            dgpu_vendor: self.detect_dgpu_vendor(),
            has_mux_switch: self.has_mux_support(),
            supports_egpu: self.has_egpu_support(),
            external_gpu_connected: self.is_egpu_connected(),
            vfio_available: self.has_vfio_support(),
        }
    }

    fn detect_dgpu(&self) -> bool {
        // Check PCI for discrete GPU
        Path::new("/sys/bus/pci/devices").exists()
    }

    fn detect_dgpu_vendor(&self) -> Option<GpuVendor> {
        // Check PCI vendor ID
        // 0x10de = Nvidia, 0x1002 = AMD
        None // Implement based on PCI enumeration
    }

    fn has_mux_support(&self) -> bool {
        Path::new("/sys/class/platform/asus-nb-wmi/mux").exists()
    }

    fn has_egpu_support(&self) -> bool {
        Path::new("/sys/class/platform/asus-nb-wmi/egpu_enable").exists()
    }

    fn is_egpu_connected(&self) -> bool {
        // CRITICAL: Proper hotplug detection algorithm
        // See research.md section 3 for details
        if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
            for entry in entries.flatten() {
                let device_path = entry.path();
                let vendor_path = device_path.join("vendor");
                let class_path = device_path.join("class");
                
                if let (Ok(vendor_str), Ok(class_str)) = (
                    fs::read_to_string(&vendor_path),
                    fs::read_to_string(&class_path)
                ) {
                    let vendor_id = vendor_str.trim();
                    let class_code = class_str.trim();
                    
                    // Check if GPU (NVIDIA, AMD, Intel)
                    if (vendor_id == "0x10de" || vendor_id == "0x1002" || vendor_id == "0x8086")
                        && (class_code == "0x030000" || class_code == "0x030200")
                    {
                        let device_name = device_path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        
                        // Check for hotplug slot (external/removable GPU)
                        if self.find_slot_power(&device_name).is_ok() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    fn find_slot_power(&self, device_address: &str) -> Result<PathBuf, std::io::Error> {
        // Iterate PCI slots and match device address
        let slots_path = Path::new("/sys/bus/pci/slots");
        
        for entry in slots_path.read_dir()? {
            let slot_path = entry?.path();
            let address_file = slot_path.join("address");
            
            if let Ok(slot_address) = fs::read_to_string(&address_file) {
                if device_address.contains(slot_address.trim()) {
                    let power_path = slot_path.join("power");
                    if power_path.exists() {
                        return Ok(power_path);
                    }
                }
            }
        }
        
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Hotplug slot not found"
        ))
    }

    fn has_vfio_support(&self) -> bool {
        Path::new("/sys/module/vfio_pci").exists()
    }
}
```

### 2. Rust Backend - FFI API

Add to `rust_core/src/api.rs`:

```rust
use crate::services::gpu::{GpuController, GpuMode, GpuConfiguration, GpuStatus};

#[frb(sync)]
pub fn get_gpu_capabilities() -> GpuConfiguration {
    let controller = GpuController::new().unwrap_or_else(|_| {
        // Return default config if controller init fails
        GpuConfiguration {
            has_dgpu: false,
            dgpu_vendor: None,
            has_mux_switch: false,
            supports_egpu: false,
            external_gpu_connected: false,
            vfio_available: false,
        }
    });
    controller.get_capabilities()
}

#[frb(sync)]
pub fn set_gpu_mode(mode: GpuMode) -> Result<(), String> {
    let controller = GpuController::new().map_err(|e| e.to_string())?;
    controller.set_mode(mode).map_err(|e| e.to_string())
}
```

### 3. Flutter - GPU Service Interface

Create `lib/services/gpu_service.dart`:

```dart
abstract class IGpuService {
  Future<GpuConfiguration> getCapabilities();
  Future<void> setGpuMode(GpuMode mode);
  Future<List<GpuMode>> getSupportedModes();
}
```

### 4. Flutter - GPU BLoC

Create `lib/blocs/gpu/gpu_bloc.dart`:

```dart
class GpuBloc extends Bloc<GpuEvent, GpuState> {
  final IGpuService gpuService;

  GpuBloc(this.gpuService) : super(GpuInitial()) {
    on<LoadCapabilitiesEvent>(_onLoadCapabilities);
    on<SetModeRequestedEvent>(_onSetModeRequested);
  }

  Future<void> _onLoadCapabilities(
    LoadCapabilitiesEvent event,
    Emitter<GpuState> emit,
  ) async {
    emit(GpuLoading());
    try {
      final config = await gpuService.getCapabilities();
      final status = await gpuService.getStatus();
      final supportedModes = await gpuService.getSupportedModes();
      emit(GpuLoaded(
        configuration: config,
        status: status,
        supportedModes: supportedModes,
      ));
    } catch (e) {
      emit(GpuError(e.toString()));
    }
  }

  Future<void> _onSetModeRequested(
    SetModeRequestedEvent event,
    Emitter<GpuState> emit,
  ) async {
    try {
      await gpuService.setGpuMode(event.mode);
      // Reload status after mode change
      add(LoadCapabilitiesEvent());
    } catch (e) {
      emit(GpuError(e.toString()));
    }
  }
}
```

### 5. Flutter - GPU Screen

Create `lib/screens/gpu_screen.dart`:

```dart
class GpuScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => GpuBloc(context.read<IGpuService>())
        ..add(LoadCapabilitiesEvent()),
      child: Scaffold(
        appBar: AppBar(title: Text('GPU Modes')),
        body: BlocBuilder<GpuBloc, GpuState>(
          builder: (context, state) {
            if (state is GpuLoaded) {
              return Column(
                children: [
                  GpuStatusCard(status: state.status),
                  GpuModeSelector(
                    supportedModes: state.supportedModes,
                    currentMode: state.status.activeMode,
                    onModeChanged: (mode) {
                      context.read<GpuBloc>().add(SetModeRequestedEvent(mode));
                    },
                  ),
                ],
              );
            }
            return Center(child: CircularProgressIndicator());
          },
        ),
      ),
    );
  }
}
```

### 6. Flutter - GPU Widgets

Create `lib/widgets/gpu/gpu_mode_selector.dart`:

```dart
class GpuModeSelector extends StatelessWidget {
  final List<GpuMode> supportedModes;
  final GpuMode currentMode;
  final Function(GpuMode) onModeChanged;

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: supportedModes.length,
      itemBuilder: (context, index) {
        final mode = supportedModes[index];
        return ListTile(
          title: Text(mode.toString()),
          trailing: Radio<GpuMode>(
            value: mode,
            groupValue: currentMode,
            onChanged: (value) {
              if (value != null) {
                onModeChanged(value);
              }
            },
          ),
        );
      },
    );
  }
}
```

## Testing

### Rust Unit Tests

Create `rust_core/tests/gpu_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_controller_creation() {
        let controller = GpuController::new();
        assert!(controller.is_ok());
    }

    #[test]
    fn test_gpu_mode_enum() {
        let modes = vec![
            GpuMode::Integrated,
            GpuMode::Hybrid,
            GpuMode::Ultimate,
            GpuMode::AsusMuxDgpu,
            GpuMode::AsusEgpu,
            GpuMode::Vfio,
        ];
        assert_eq!(modes.len(), 6);
    }
}
```

### Flutter Widget Tests

Create `test/blocs/gpu/gpu_bloc_test.dart`:

```dart
void main() {
  group('GpuBloc', () {
    late GpuBloc gpuBloc;
    late MockGpuService mockGpuService;

    setUp(() {
      mockGpuService = MockGpuService();
      gpuBloc = GpuBloc(mockGpuService);
    });

    test('emits GpuLoaded when LoadCapabilitiesEvent is added', () async {
      when(mockGpuService.getCapabilities())
          .thenAnswer((_) async => GpuConfiguration(/* ... */));

      gpuBloc.add(LoadCapabilitiesEvent());

      await expectLater(
        gpuBloc.stream,
        emits(isA<GpuLoaded>()),
      );
    });
  });
}
```

## Troubleshooting

### Common Issues

**Issue**: Permission denied when setting GPU mode
**Solution**: Run `install/install.sh` to set up udev rules

**Issue**: GPU mode not supported on hardware
**Solution**: Check hardware capabilities via `get_gpu_capabilities()`

**Issue**: MUX switch fails
**Solution**: Verify kernel version is 6.1+, check BIOS settings

**Issue**: eGPU not detected
**Solution**: Ensure XG Mobile is connected and powered on

**Issue**: VFIO mode not available
**Solution**: Install vfio-pci kernel module, enable IOMMU in kernel cmdline

## Alternative: Use supergfxctl via D-Bus (RECOMMENDED)

**Why**: supergfxctl is the reference implementation with proven XG Mobile support.

### Quick Setup

1. **Create D-Bus client** `lib/services/supergfx_client.dart`:

```dart
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
      _interface, 'SetMode', [DBusByte(mode)],
      replySignature: DBusSignature('y'),
    );
    return result.returnValues[0].asByte(); // UserActionRequired
  }
}
```

2. **GPU Modes** (byte values):

```dart
0 = Hybrid
1 = Integrated
2 = NvidiaNoModeset
3 = Vfio
4 = AsusEgpu        // ← XG Mobile!
5 = AsusMuxDgpu
6 = None
```

3. **Check if supergfxd is running**:

```bash
systemctl status supergfxd
busctl introspect org.supergfxctl.Daemon /org/supergfxctl/Gfx
```

### Advantages

- ✅ Proven XG Mobile detection on ASUS Flow series
- ✅ No complex PCI enumeration to maintain
- ✅ Handles driver conflicts automatically
- ✅ Real-time notifications via D-Bus signals
- ✅ Already uses D-Bus for asusd integration

## References

- **supergfxctl D-Bus**: https://gitlab.com/asus-linux/supergfxctl/-/blob/main/src/zbus_iface.rs
- **supergfxctl source**: https://gitlab.com/asus-linux/supergfxctl
- ASUS WMI kernel docs: https://www.kernel.org/doc/html/latest/admin-guide/laptops/asus-wmi.html
- nvml-wrapper: https://github.com/rust-nvml/nvml-wrapper
- Project constitution: `.specify/memory/constitution.md`

**IMPORTANT**: See `plan.md` "Implementation Strategy Update" for decision between Rust implementation vs supergfxctl D-Bus.
