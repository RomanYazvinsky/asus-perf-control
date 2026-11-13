# `rust_daemon` — privileged daemon & hotkey agent

Two binaries:

| Binary | Runs as | Purpose |
|---|---|---|
| `asus-control-daemon` (`src/main.rs`) | root, system bus | privileged features not covered by asusd/supergfxd |
| `asus-control-hotkey-agent` (`src/bin/asus-control-hotkey-agent.rs` → `src/hotkey_agent.rs`) | user session | launches the app on ROG key presses |

Both are installed to `/usr/lib/asus-linux-control/`.

## Layout

```text
rust_daemon/src/
├── main.rs              # D-Bus interface, startup restore, platform profile watcher, shutdown
├── polkit.rs            # CheckAuthorization for the calling process
├── hardware/
│   ├── temp.rs          # per-profile CPU temperature limit, ryzenadj
│   ├── cpu_boost.rs     # cpufreq boost + EPP discovery and modes
│   ├── fan.rs           # fan PWM / temperature sysfs used by the engine
│   └── rog_mkey.rs      # ASUS M-key HID feature reports (from G-Helper)
├── fan_engine/          # dynamic fan engine: config, interpolation, safety, controller, threaded driver
├── package_updates/     # backend detection, command allowlist, update jobs, output parsing, status
├── rog_hotkey.rs        # ROG key runtime: source selection, firmware binding guard, monitor thread
├── rog_input.rs         # evdev reading and ROG event matching
└── hotkey_agent.rs      # user agent: GNOME shortcut or signal → gtk-launch
```

## D-Bus interface

- Bus name: `com.asuslinuxcontrol.Daemon` (system bus)
- Object path: `/com/asuslinuxcontrol/Daemon`
- Interface: `com.asuslinuxcontrol.Daemon`

| Method | Signature | Polkit action | Description |
|---|---|---|---|
| `GetTempLimit` | `→ i` | — | CPU temperature limit of the active profile (0 = none) |
| `SetTempLimit` | `i → b` | `set-temp-limit` | Set it for the active profile and apply |
| `GetProfileTempLimit` | `s → i` | — | Limit of a profile (`quiet` / `balanced` / `performance`) |
| `SetProfileTempLimit` | `si → b` | `set-temp-limit` | Save a profile's limit; applied now if that profile is active |
| `GetCpuBoostStatus` | `→ s` (JSON) | — | Available modes, current mode, discovered knobs |
| `SetCpuBoostMode` | `s → b` | `set-cpu-boost-mode` | `disabled`, `efficient`, `efficient_enabled`, `enabled`, `efficient_aggressive`, `aggressive` |
| `StartFanEngine` | `s → b` | `manage-fan-engine` | Start the dynamic engine from a JSON config and save it |
| `UpdateFanEngineCurve` | `ss → b` | `manage-fan-engine` | Replace one fan's curve (`cpu` / `gpu`, JSON points) |
| `StopFanEngine` | `→ b` | `manage-fan-engine` | Stop, clear saved config, restore automatic control |
| `GetFanEngineRunning` | `→ b` | — | Whether the engine runs |
| `GetPackageBackend` | `→ s` | — | Detected package backend |
| `CheckForUpdates` | `→ s` (JSON) | — | Installed / available versions |
| `StartUpdate` | `→ s` | `manage-updates` | Start an update job, returns its id |
| `GetUpdateStatus` | `s → s` (JSON) | — | Job progress / result |
| `CancelUpdate` | `s → b` | `manage-updates` | Cancel a job |
| `GetRogHotkeyStatus` | `→ s` (JSON) | — | ROG key runtime diagnostics |

| Signal | Description |
|---|---|
| `RogKeyPressed` | Emitted once per (debounced) ROG key press |

Polkit actions are prefixed with `com.asuslinuxcontrol.` and defined in
`install/dbus/com.asuslinuxcontrol.Daemon.policy`. Check the exact authorisation per method in
`src/main.rs` when changing the interface.

### Calling it by hand

```bash
busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetCpuBoostStatus

busctl call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon SetProfileTempLimit si balanced 90

busctl monitor --system --match "type='signal',interface='com.asuslinuxcontrol.Daemon'"
```

## Lifecycle

**Start**

1. Restore the saved dynamic fan engine config, if any.
2. Re-write asusd's active platform profile to refresh the system EPP baseline, then re-apply
   the saved CPU boost mode on top of it.
3. Start the platform profile watcher. Its first pass applies the active profile's CPU
   temperature limit; after that it re-applies the limit after every profile write, because
   the firmware resets `tctl` on each write.
4. Start the ROG hotkey runtime (when `ASUS_ROG_HOTKEY_ENABLED=1`). Firmware is changed only
   after all other startup calls are done.

**Stop** (SIGINT / SIGTERM): restore the original ROG firmware binding (with retries).

## Environment

| Variable | Default | Effect |
|---|---|---|
| `RUST_LOG` | `info` (unit file) | log level |
| `ASUS_ROG_HOTKEY_ENABLED` | unset (set to `1` by the drop-in) | enable ROG key handling |
| `ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK` | unset | allow the WMI software fallback without firmware binding |

## Running manually

```bash
sudo systemctl stop asus-control-daemon
sudo RUST_LOG=debug ASUS_ROG_HOTKEY_ENABLED=1 ./target/release/asus-control-daemon
```

## Tests

`cargo test` in `rust_daemon/`. Integration tests in `tests/` include the source modules
directly (`#[path]`) and cover the fan engine, CPU boost, per-mode temperature settings,
package update backends, ROG key input matching / M-key protocol, and the hotkey agent.
