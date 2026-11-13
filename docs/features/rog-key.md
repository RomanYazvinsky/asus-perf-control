# ROG key

Pressing the ROG (M4 / Armoury Crate) key opens ASUS Linux Control, or raises it if it is
already running.

## How the key is detected (daemon)

Enabled by the systemd drop-in `install/systemd/asus-control-daemon.service.d/rog-hotkey.conf`
(`ASUS_ROG_HOTKEY_ENABLED=1`). On start the daemon tries, in order:

1. **Firmware M-key binding.** Finds the ASUS keyboard's HID interface (`/dev/hidraw*`) and
   binds the key in firmware so it reports scan `0x38` on the `asus-nb-wmi` input device. The
   original binding is restored when the daemon stops. The report sequence is adapted from
   [G-Helper](https://github.com/seerge/g-helper/pull/5691).
2. **Native key.** If the firmware has no M-key binding, it listens on the ASUS USB keyboard
   that already reports the key as `KEY_PROG1`, matching only the ASUS vendor scan code.
3. **Software fallback** (opt-in, `ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1`): listens on the WMI
   input device without changing firmware.

Devices are read without grabbing them, and presses are debounced (300 ms). Each press emits
the D-Bus signal `RogKeyPressed`. `GetRogHotkeyStatus` returns JSON diagnostics: chosen source,
input path, firmware changes, restoration state, dropped presses, last error.

## How the app is launched (hotkey agent)

`asus-control-hotkey-agent` runs in the user session (`systemd --user`, `graphical-session.target`).

- **GNOME:** the agent registers a GNOME custom shortcut
  (`XF86Launch1` → `asus-linux-control`) via `gsettings` and then stays idle. GNOME starts the
  app itself, which gives it an `XDG_ACTIVATION_TOKEN`; with it the window may take focus
  despite Wayland focus-stealing prevention. If registering the shortcut fails, the agent
  falls back to the generic path.
- **Other desktops:** the agent activates the daemon, subscribes to `RogKeyPressed` and runs
  `gtk-launch asus-linux-control` on every press. It only does so when its session is the
  active one on `seat0` (logind), so a press doesn't open the app in a background session.

A second launch never opens a second window; see [single instance](window-and-input.md).

## Code

| Part | Files |
|---|---|
| Key detection | `rust_daemon/src/rog_hotkey.rs`, `rust_daemon/src/rog_input.rs`, `rust_daemon/src/hardware/rog_mkey.rs` |
| Agent | `rust_daemon/src/hotkey_agent.rs`, `rust_daemon/src/bin/asus-control-hotkey-agent.rs` |
| Units | `install/systemd/asus-control-hotkey-agent.service`, `install/systemd/asus-control-daemon.service.d/rog-hotkey.conf` |
| SELinux | `install/selinux/asus_control.cil` (access to `hidraw` and the input device) |

Manual test plan: [`rog-m4-hotkey-manual-verification.md`](../research/rog-m4-hotkey-manual-verification.md).
