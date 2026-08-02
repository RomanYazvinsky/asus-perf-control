# ROG/M4 hotkey activation

## Goal

Pressing the physical ASUS ROG/M4 key must launch ASUS Linux Control when it
is not running and activate its window when it is already running.

## Background and attribution

The firmware-first design is based on the M-key hardware bindings approach
introduced by the G-Helper project and its author, `seerge`:
<https://github.com/seerge/g-helper/pull/5691>.

The implementation must retain this attribution in the source documentation
for the ASUS HID/EC protocol adapter.

## Design

### Hardware layer

The Rust system daemon will own an ASUS M-key adapter. It will:

1. Locate the compatible ASUS HID device and perform a read-only capability
   probe before attempting any write.
2. Determine whether a physical ROG/M4 slot exists. The slot is model
   dependent, so the implementation must derive it from the reported slot
   layout rather than assume one Linux input keycode.
3. Preserve the firmware default for the ROG/M4 action where possible, then
   listen for its resulting hardware event.
4. Emit a `RogKeyPressed` D-Bus signal with debouncing. Unsupported devices
   and failed probes remain unchanged and are reported as unsupported.

The adapter must not modify any M-key binding outside the ROG/M4 slot, and it
must restore the original ROG/M4 firmware value if it has changed it.

### Session layer

A small `systemd --user` service will subscribe to the daemon signal. It will
activate the desktop application through its desktop entry / single-instance
activation path. This service remains alive independently of Flutter, so a
ROG/M4 press works when the GUI is closed. Repeated presses only raise the
existing window.

### Fallback and diagnostics

For devices that do not expose compatible M-key firmware bindings, the daemon
will expose a diagnostic command that reports the detected input source and
scan/event value. The session layer may then use that event as a software
fallback. The fallback is opt-in until the hardware event can be verified on
the laptop; it must never consume an arbitrary keyboard event.

## Error handling

- HID unavailable, malformed, or unsupported responses: log capability as
  unsupported and leave firmware unchanged.
- D-Bus/session agent unavailable: retain the event in daemon logs and do not
  start a GUI process from the privileged daemon.
- Activation failure: log the command result; the next key press retries.

## Testing

- Unit-test capability parsing, ROG/M4 slot detection, message construction,
  debounce, and the no-write path for unsupported hardware.
- Unit-test the session activation request and idempotent repeated presses.
- Manual Linux verification: press ROG/M4 with the GUI absent, minimized, and
  focused; verify that other M-keys keep their original action.

## Non-goals

- General remapping UI for M1–M5.
- Reusing Windows-specific keycodes or Windows APIs.
- Changing the firmware bindings of unrelated keys.
