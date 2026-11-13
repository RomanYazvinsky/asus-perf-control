# Fan control

Screen: **Fans** (`rust_ui/src/ui/screens/fan.rs`). CPU and GPU fans are shown side by side,
each with an 8-point curve editor (`rust_ui/src/ui/curve_editor.rs`).

## Control modes

Each fan uses one of three modes, chosen per fan and saved in the profile
(`hardware.fan.controlModes`):

| Mode | How it works | Owner |
|---|---|---|
| **Curve** (default) | 8-point temperature → speed curve written to firmware per platform profile | asusd `FanCurves` |
| **Constant** | A flat asusd curve at the selected speed | asusd |
| **Dynamic engine** | The daemon reads temperatures and writes fan PWM itself every tick | own daemon |

**Reset** calls asusd `SetCurvesToDefaults` for the profile.

## Dynamic fan engine

- Config: per-fan 8-point curve, `tick_ms`, `hysteresis_c`. Validated by the daemon
  (`rust_daemon/src/fan_engine/config.rs`).
- Safety: at 80 °C or more the speed never drops below 30 %
  (`rust_daemon/src/fan_engine/safety.rs`).
- Runs on its own thread (`threaded_driver.rs`), writes PWM through `hardware/fan.rs`.
- Config is saved to `/var/lib/asus-linux-control/fan-engine-config.json` and restored on
  daemon start. **Stop** clears it and restores automatic fan control.
- D-Bus: `StartFanEngine`, `UpdateFanEngineCurve`, `StopFanEngine`, `GetFanEngineRunning`.
  Polkit action: `com.asuslinuxcontrol.manage-fan-engine`.

## Readings

Fan RPMs and CPU/GPU temperatures come from `rust_core` (hwmon sysfs) and refresh
periodically while the screen is open. Refresh failures are ignored.

## Max speed test

Applies a temporary flat 100 % asusd curve, measures the reached RPM through `rust_core`,
restores the previous curve and shows the measured maximum.

## Keyboard / gamepad in the curve editor

`[` `]` or PageUp/PageDown select a point, ←/→ move its temperature, ↑/↓ its speed. The mouse
drags points. Nothing is written until **Apply**.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/fan.rs`, `rust_ui/src/ui/curve_editor.rs` |
| Logic | `rust_ui/src/features/fan/` (`mod.rs`, `model.rs`) |
| Clients | `rust_ui/src/backend/asusd.rs`, `rust_ui/src/backend/daemon.rs` |
| Readers | `rust_core/src/services/fan.rs`, `rust_core/src/services/temp.rs` |
| Daemon | `rust_daemon/src/fan_engine/`, `rust_daemon/src/hardware/fan.rs` |

Research behind direct fan control: [`docs/research/gv301qh-direct-fan-control.md`](../research/gv301qh-direct-fan-control.md).
