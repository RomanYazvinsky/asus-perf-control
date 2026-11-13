# Performance mode & CPU boost

Screen: **Performance** (`rust_ui/src/ui/screens/home.rs`)

## Performance mode

Silent / Balanced / Turbo, mapped to asusd's `PlatformProfile`:

| UI mode | asusd profile | `throttle_thermal_policy` |
|---|---|---|
| Silent (0) | Quiet / LowPower | 2 |
| Balanced (1) | Balanced | 0 |
| Turbo (2) | Performance | 1 |

Route: `home screen → features::performance → Services → backend::asusd → asusd`.

On load the hardware mode is read. If a saved mode exists and differs, it is re-applied;
with no saved mode the hardware mode becomes the saved one.

Changing the profile also changes which per-profile CPU temperature limit the daemon applies
(see [power limits](power-limits.md)).

## CPU boost

Four presets, each resolving to the first mode the CPU actually supports:

| Preset | Daemon modes (in order of preference) |
|---|---|
| Off | `disabled` |
| Efficient | `efficient_enabled`, `efficient` |
| Balanced | `efficient_aggressive`, `enabled` |
| Aggressive | `aggressive` |

The daemon discovers the boost knobs under `/sys/devices/system/cpu/cpufreq`
(global `boost`, per-policy `boost`, or per-CPU `cpufreq/boost`) and sets the energy
performance preference (EPP) where `amd-pstate` / `intel_pstate` expose it. The selected mode
is saved to `/var/lib/asus-linux-control/cpu-boost-mode.json` and re-applied on daemon start.

Route: `home screen → features::cpu_boost → Services → backend::daemon → GetCpuBoostStatus / SetCpuBoostMode`.
Polkit action: `com.asuslinuxcontrol.set-cpu-boost-mode`.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/home.rs`, `rust_ui/src/ui/widgets.rs` |
| Logic | `rust_ui/src/features/performance.rs`, `rust_ui/src/features/cpu_boost.rs` |
| Clients | `rust_ui/src/backend/asusd.rs`, `rust_ui/src/backend/daemon.rs` |
| Daemon | `rust_daemon/src/hardware/cpu_boost.rs` |
