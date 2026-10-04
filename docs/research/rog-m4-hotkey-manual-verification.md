# ROG/M4 hotkey manual verification

Protocol attribution: the ASUS M-key firmware report sequence is adapted from
G-Helper by seerge: https://github.com/seerge/g-helper/pull/5691

Run these checks on supported ASUS hardware after installing and starting the
system daemon and the per-user hotkey agent.

## Input sources

The daemon picks the first source that works, in this order:

1. **Firmware M-key binding** (newer models, G-Helper protocol): the daemon
   rebinds the ROG slot over hidraw and listens on
   `/dev/input/by-path/platform-asus-nb-wmi-event` for scan `0x38`. The original
   binding is restored on shutdown.
2. **Native keyboard key** (older models whose firmware reports
   `invalid M-key slot count: 0`, e.g. ROG Flow X13 GV301QH): the ASUS USB
   keyboard ("N-KEY Device") already sends `KEY_PROG1` with HID scan
   `0xff310038`. The daemon finds that input device through sysfs (vendor
   `0b05`, USB bus, `KEY_PROG1` capability), accepts only that exact
   scan + key pair, and changes no firmware state. `GetRogHotkeyStatus`
   reports `hardwareSupported: true`, `firmwareChanged: false` and the
   `/dev/input/eventN` path as `inputPath`.
3. **Opt-in software fallback** (`ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1`, see step 7).

To see which events a model sends, watch the input devices while pressing the
key (user must be in the `input` group), for example with `evtest`.

1. Inspect capability without pressing the key. Run:

   ```bash
   busctl --system call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetRogHotkeyStatus
   ```

   Confirm the JSON reports schema version 1, the detected hardware identity,
   monitor state, firmware-change state, fallback state, and no unexpected
   errors.

2. Close the GUI completely, press ROG/M4 once, and verify exactly one ASUS
   Linux Control GUI process appears.

3. Minimize the GUI, press ROG/M4, and verify the same process and window are
   presented instead of starting another process or application window.

4. Press ROG/M4 twice within 300 ms and verify debounce prevents duplicate
   activation.

5. Verify the separate physical M1, M2, M3, and M4 slots retain their prior
   actions. The target of this integration is the ROG-logo/M4 hotkey event; it
   must not remap or consume the separate physical M-key slots.

6. Stop the daemon and inspect its log. If startup changed the ROG firmware
   opcode, verify shutdown reports that the original opcode was restored:

   ```bash
   sudo systemctl stop asus-control-daemon.service
   journalctl -u asus-control-daemon.service -n 100 --no-pager
   ```

7. On unsupported firmware, verify discovery performs no binding-update/set request
   (the read-only protocol probe may send a query feature report)
   and `GetRogHotkeyStatus` reports the hardware error with fallback disabled.
   Only for diagnosis, enable the explicit override in a systemd service
   override, restart the daemon, and confirm the diagnostic status reports
   fallback enabled:

   ```ini
   [Service]
   Environment=ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1
   ```

   Remove the override after testing. Values other than exactly `1` must remain
   disabled.
