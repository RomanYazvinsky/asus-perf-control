# Feature Specification: Extended GPU Modes Support

**Feature Branch**: `002-gpu-modes-egpu`  
**Created**: 2026-04-13  
**Status**: Draft  
**Input**: User description: "Переделать спецификацию GPU modes: добавить поддержку eGPU (AsusEgpu, XG Mobile mode) в дополнение к MUX. Источники: supergfxctl"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic GPU Mode Switching (Priority: P1)

As an ASUS laptop user with hybrid graphics, I want to switch between Integrated, Hybrid, and Ultimate modes to balance battery life and graphics performance for everyday use.

**Why this priority**: Core functionality for battery optimization and performance tuning. Most users need these basic modes for daily work and gaming.

**Independent Test**: Can be fully tested by switching GPU modes and verifying that discrete GPU is enabled/disabled via system power consumption and GPU detection tools.

**Acceptance Scenarios**:

1. **Given** laptop has hybrid graphics with dGPU, **When** user selects "Integrated" mode, **Then** discrete GPU is disabled and system uses integrated graphics only
2. **Given** user is in "Integrated" mode on battery, **When** user plugs into AC power, **Then** system can optionally auto-switch to "Hybrid" mode if automation is enabled
3. **Given** GPU mode change requires session restart, **When** user changes mode, **Then** system warns user about restart requirement

---

### User Story 2 - ASUS MUX Switching (Priority: P2)

As an ASUS gaming laptop owner with MUX switch support, I want to toggle the MUX to use dGPU as primary graphics for maximum performance in games.

**Why this priority**: Advanced feature for enthusiasts with MUX-capable hardware. Provides performance benefit but requires reboot, so lower priority than basic modes.

**Independent Test**: Can be fully tested by toggling MUX switch, rebooting, and verifying display is connected directly to dGPU via GPU detection tools and performance benchmarks.

**Acceptance Scenarios**:

1. **Given** laptop supports ASUS MUX switch, **When** user selects "AsusMuxDgpu" mode, **Then** system warns about reboot requirement and applies MUX change after confirmation
2. **Given** MUX is enabled with dGPU as primary, **When** system reboots, **Then** display output is routed through dGPU directly
3. **Given** MUX switch is not supported on hardware, **When** user opens GPU settings, **Then** "AsusMuxDgpu" option is hidden or disabled with explanatory message

---

### User Story 3 - External GPU Support (Priority: P3)

As an ASUS Flow series or XG Mobile user, I want to switch to external GPU mode to use my dedicated eGPU for maximum graphics performance.

**Why this priority**: Niche feature for users with external GPU hardware. Lower priority as it applies to specific laptop models and use cases.

**Independent Test**: Can be fully tested by connecting an external GPU (e.g., ASUS XG Mobile), switching to "AsusEgpu" mode, and verifying external GPU is active and rendering graphics.

**Acceptance Scenarios**:

1. **Given** external GPU (XG Mobile) is connected to laptop, **When** user selects "AsusEgpu" mode, **Then** system switches graphics output to external GPU
2. **Given** external GPU is disconnected, **When** user attempts to select "AsusEgpu" mode, **Then** system displays error message indicating no external GPU detected
3. **Given** laptop does not support external GPU switching, **When** user opens GPU settings, **Then** "AsusEgpu" option is hidden or disabled with explanatory message

---

### User Story 4 - VFIO GPU Passthrough (Priority: P4)

As a power user running virtual machines, I want to bind the dGPU to VFIO for GPU passthrough to VMs for gaming or GPU computing workloads.

**Why this priority**: Advanced feature for virtualization enthusiasts. Requires kernel configuration and specialized knowledge, so lowest priority.

**Independent Test**: Can be fully tested by switching to VFIO mode, verifying dGPU is bound to vfio-pci driver, and confirming GPU is available for VM passthrough.

**Acceptance Scenarios**:

1. **Given** vfio modules are loaded and configured, **When** user selects "Vfio" mode, **Then** dGPU is unbound from graphics driver and bound to vfio-pci
2. **Given** vfio modules are not available, **When** user attempts to select "Vfio" mode, **Then** system displays error message indicating vfio support not available
3. **Given** user is in "Vfio" mode, **When** user switches to any other mode, **Then** dGPU is rebound to graphics driver automatically

---

### Edge Cases

- What happens when GPU mode switch fails due to driver conflicts or kernel issues?
- How does system handle when external GPU is disconnected while in AsusEgpu mode?
- What happens when MUX switch fails or hardware does not support it?
- How does system handle when vfio modules are not compiled as separate kernel modules?
- What happens when user tries to switch modes while a graphical session is active?
- How does system handle conflicting settings from other GPU switchers (optimus-manager, supergfxctl, etc.)?
- What happens when kernel version is too old to support required GPU switching features?
- How does system handle when dGPU vendor cannot be detected (Nvidia vs AMD)?

## Requirements *(mandatory)*

### Functional Requirements

#### GPU Mode Detection and Support
- **FR-001**: System MUST detect which GPU modes are supported on the current hardware (Integrated, Hybrid, AsusMuxDgpu, AsusEgpu, Vfio)
- **FR-002**: System MUST automatically hide or disable unsupported GPU modes in the UI based on hardware detection
- **FR-003**: System MUST detect dGPU vendor (Nvidia or AMD) for appropriate driver handling
- **FR-004**: System MUST detect external GPU connection status (AsusEgpu/XG Mobile)

#### Basic GPU Mode Switching
- **FR-005**: System MUST support switching to Integrated mode (iGPU only, dGPU force-disabled)
- **FR-006**: System MUST support switching to Hybrid mode (dGPU-offload mode)
- **FR-007**: System MUST support switching to Ultimate mode (dGPU primary for rendering)
- **FR-008**: System MUST warn user when mode change requires session restart or logout
- **FR-009**: System MUST persist selected GPU mode across application restarts
- **FR-010**: System MUST detect current GPU mode on application startup

#### ASUS MUX Switching
- **FR-011**: System MUST support AsusMuxDgpu mode for laptops with MUX switch capability
- **FR-012**: System MUST warn user that AsusMuxDgpu mode requires system reboot
- **FR-013**: System MUST apply MUX switch after user confirmation and initiate reboot
- **FR-014**: System MUST detect MUX switch support via hardware detection

#### External GPU Support
- **FR-015**: System MUST support AsusEgpu mode for ASUS Flow series and XG Mobile
- **FR-016**: System MUST detect external GPU connection before allowing AsusEgpu mode
- **FR-017**: System MUST switch graphics output to external GPU when AsusEgpu mode is activated
- **FR-018**: System MUST display error when AsusEgpu is selected without external GPU connected

#### VFIO GPU Passthrough
- **FR-019**: System MUST support Vfio mode to bind dGPU to vfio-pci for VM passthrough
- **FR-020**: System MUST verify vfio kernel modules are available before enabling Vfio mode
- **FR-021**: System MUST unbind dGPU from graphics driver and bind to vfio-pci in Vfio mode
- **FR-022**: System MUST rebind dGPU to graphics driver when switching out of Vfio mode

#### GPU Monitoring and Status
- **FR-023**: System MUST read GPU temperature for Nvidia GPUs via NVML
- **FR-024**: System MUST read GPU temperature for AMD GPUs from sysfs drm interfaces
- **FR-025**: System MUST read GPU utilization statistics when available
- **FR-026**: System MUST display current active GPU mode in the UI

#### Configuration and Automation
- **FR-027**: System MUST support automatic GPU mode switching based on AC/battery power state
- **FR-028**: System MUST save GPU mode preference to user configuration
- **FR-029**: System MUST provide option to set default GPU mode via kernel parameter

#### Error Handling and User Feedback
- **FR-030**: System MUST display clear error messages when GPU mode switch fails
- **FR-031**: System MUST provide troubleshooting guidance for common GPU switching issues
- **FR-032**: System MUST detect conflicts with other GPU switchers and warn user

### Key Entities

- **GPU Mode**: Represents the current graphics mode of the system. Attributes: mode_type (Integrated/Hybrid/Ultimate/AsusMuxDgpu/AsusEgpu/Vfio), is_supported (boolean), requires_reboot (boolean), requires_logout (boolean)
- **GPU Configuration**: Represents detected GPU hardware capabilities. Attributes: has_dgpu (boolean), dgpu_vendor (Nvidia/AMD/None), has_mux_switch (boolean), supports_egpu (boolean), external_gpu_connected (boolean), vfio_available (boolean)
- **GPU Status**: Represents current GPU state. Attributes: active_mode, dgpu_enabled, temperature, utilization, power_state

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully switch between all supported GPU modes on their hardware within 10 seconds (excluding reboot requirements)
- **SC-002**: System correctly detects and displays only supported GPU modes based on hardware capabilities
- **SC-003**: External GPU (AsusEgpu/XG Mobile) switching works when external GPU is connected
- **SC-004**: MUX switching successfully applies after reboot on supported hardware
- **SC-005**: VFIO mode successfully binds dGPU to vfio-pci when vfio modules are available
- **SC-006**: 95% of mode switch attempts complete successfully without errors on supported hardware
- **SC-007**: Error messages provide actionable guidance for 90% of common GPU switching failures

## Assumptions

- Target users have ASUS laptops with hybrid graphics (Intel/AMD iGPU + Nvidia/AMD dGPU)
- Users have Linux kernel 6.1+ or newer for full GPU switching support
- External GPU support primarily targets ASUS Flow series and XG Mobile devices
- VFIO mode assumes users have configured vfio modules separately if needed
- System has appropriate permissions to access GPU control interfaces (udev rules, D-Bus)
- Conflicts with other GPU switchers should be detected and reported to user
- GPU mode switching may require logout or reboot depending on the specific mode and hardware
- Not all GPU modes are available on all laptop models - system must detect and adapt
