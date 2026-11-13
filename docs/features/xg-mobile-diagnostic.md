# XG Mobile fan diagnostic

Screen: **XG Diag** (`rust_ui/src/ui/screens/xg_diag.rs`)

A diagnostic tool for the ASUS XG Mobile eGPU dock. It sweeps the GPU/CPU fan PWM through
asusd custom curves and samples hwmon after every step, so you can see whether the driver
actually honours the requested value.

- Each run writes and enables a custom curve, then records hwmon snapshots
  (`fan1_input`, `fan2_input`, `pwm*_enable`, `pwm2_auto_point8_pwm`).
- **Limitation:** the hwmon nodes are currently hard-coded (`hwmon7` for RPM, `hwmon8` for PWM),
  as found on the GV301QH + XG Mobile used during research. Other machines may number them differently.
- Every test restores the default curves when it finishes, also on error.
- Runs are kept in a session file:
  `$XDG_STATE_HOME/asus-linux-control/xg_mobile_fan_diagnostic.json`
  (same JSON shape as the former Flutter app).
- Unlike other screens, it keeps a persistent status/error panel next to the captured runs.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/xg_diag.rs` |
| Logic | `rust_ui/src/features/xg_diag/` (`mod.rs`, `model.rs`) |

Background: [`fan-override-diagnostic-log.md`](../research/fan-override-diagnostic-log.md),
[`saio-direct-fan-test-log.md`](../research/saio-direct-fan-test-log.md).
