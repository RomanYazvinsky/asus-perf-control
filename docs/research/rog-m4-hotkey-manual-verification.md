# ROG/M4 hotkey manual verification

Protocol attribution: the ASUS M-key firmware report sequence is adapted from
G-Helper by seerge: https://github.com/seerge/g-helper/pull/5691

Run these checks on supported ASUS hardware after installing and starting the
system daemon and the per-user hotkey agent.

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
   presented instead of starting another process or Flutter window.

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
