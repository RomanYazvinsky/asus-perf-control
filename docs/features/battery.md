# Battery charge limit

Screen: **Battery** (`rust_ui/src/ui/screens/battery.rs`)

Stops charging at the selected level (20–100 %) to reduce battery wear. Backed by the kernel
attribute `charge_control_end_threshold` through asusd's `ChargeControlEndThreshold` property.

1. The screen checks support first; unsupported hardware shows a placeholder.
2. It loads the current limit. If a saved limit exists and differs, the saved one is restored.
3. **Apply** (sticky action bar) writes the new limit and saves it as
   `hardware.batteryChargeLimit` in the active profile.

Route: `battery screen → features::battery → Services → backend::asusd → asusd`.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/battery.rs` |
| Logic | `rust_ui/src/features/battery.rs` |
| Client | `rust_ui/src/backend/asusd.rs` |
