# Power limits (TDP) & CPU temperature limit

Screen: **TDP** (`rust_ui/src/ui/screens/tdp.rs`)

The screen has one tab per platform profile (Silent / Balanced / Turbo). A tab selects which
profile is **edited**; it never switches the active profile.

## Sustained and boost power (PPT)

| Setting | sysfs attribute | Typical range | Owner |
|---|---|---|---|
| Sustained (PL1 / SPL) | `ppt_pl1_spl` | 5–54 W | asusd Armoury PPT objects |
| Boost (PL2 / SPPT) | `ppt_pl2_sppt` | 5–54 W | asusd Armoury PPT objects |

- Real min/max values come from asusd for the current machine.
- asusd stores PPT values per profile but can only change the **active** profile's values, so
  the power sliders are editable only on the active profile's tab.
- **Clear** restores firmware defaults (`RestoreDefault`) and disables the PPT group
  (`EnablePptGroup=false`).
- When the hardware reports 0 (never configured), the UI falls back to the saved values.

## CPU temperature limit

- Range 60–100 °C; 0 means no limit.
- Stored **per platform profile** by the daemon in
  `/var/lib/asus-linux-control/per-mode-settings.json`, so it can be edited on any tab.
- Applied with `ryzenadj --tctl-temp` (AMD Ryzen only; the daemon expects
  `/usr/local/bin/ryzenadj`).
- Re-applied by the daemon on start and after every platform profile change, because firmware
  resets it when the profile is rewritten.

Polkit action: `com.asuslinuxcontrol.set-temp-limit`.

TDP and temperature values written to hardware are volatile; the app always reads the current
values on start.

## Routes

```text
Load    tdp screen → features::tdp → Services → asusd (PPT, active profile)
                                            └→ daemon GetProfileTempLimit(profile)
Apply   tdp screen → features::tdp → Services → asusd (PPT)        [active profile only]
                                            └→ daemon SetProfileTempLimit(profile, °C) → ryzenadj
```

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/tdp.rs` |
| Logic | `rust_ui/src/features/tdp.rs` |
| Clients | `rust_ui/src/backend/asusd.rs`, `rust_ui/src/backend/daemon.rs` |
| Daemon | `rust_daemon/src/hardware/temp.rs`, platform profile watcher in `rust_daemon/src/main.rs` |
