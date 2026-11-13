# `rust_ui` — desktop application

Binary: `asus-linux-control`. Built with [egui / eframe](https://github.com/emilk/egui) 0.36,
[zbus](https://github.com/dbus2/zbus) 4 and Tokio.

## Layout

```text
rust_ui/src/
├── main.rs             # logging, activation token, single instance, eframe bootstrap
├── app.rs              # shell: toolbar, navigation rail, screens, alerts, window raising
├── activation.rs       # xdg-activation on Wayland
├── single_instance.rs  # Unix-socket single instance
├── runtime.rs          # background Tokio runtime + pollable Task<T>
├── input.rs            # gamepad commands → egui events
├── logging.rs          # structured "[Backend] STATUS op key=value elapsedMs=N" lines
├── backend/            # D-Bus clients
│   ├── bus.rs          #   shared lazy system-bus connection, call helpers
│   ├── asusd.rs        #   xyz.ljones.Asusd: profile, PPT, charge limit, fan curves
│   ├── supergfx.rs     #   org.supergfxctl.Daemon: GPU modes
│   └── daemon.rs       #   com.asuslinuxcontrol.Daemon: own privileged daemon
├── features/           # screen logic: state + async operations over service traits
│   ├── services.rs     #   production implementation of every service trait
│   ├── performance.rs  cpu_boost.rs  tdp.rs  battery.rs  gpu.rs  updates.rs  telemetry.rs
│   ├── fan/            #   fan model + logic
│   └── xg_diag/        #   XG Mobile diagnostic model + logic
├── settings/           # settings.json + profiles/*.json (model, lenient JSON, store)
└── ui/                 # presentation
    ├── screens/        #   one module per navigation destination
    ├── toolbar.rs      #   status toolbar
    ├── curve_editor.rs #   8-point fan curve editor
    ├── alert.rs        #   modal error queue with copyable details
    ├── widgets.rs      #   shared widgets, 12-column responsive container
    └── theme.rs        #   dark Material 3 palette
```

## Layers

| Layer | Rule |
|---|---|
| `ui/` | egui only. Reads feature state, starts jobs, shows alerts. No D-Bus, no sysfs. |
| `features/` | Pure logic over service traits (`PerformanceService`, `TdpService`, `FanService`, …). Unit-tested with fakes. |
| `features/services.rs` | The single `Services` struct implementing all traits on top of `backend/` and `rust_core`. |
| `backend/` | Knows bus names, object paths and D-Bus signatures. |

## Async model

The UI thread never awaits. A screen calls `cx.rt.spawn(async { … })`, stores the returned
`Task<T>` and checks it every frame with `take_ready(&mut task)`. A finished job calls
`request_repaint`, so idle screens cost nothing. Timers (fan RPM refresh, toolbar) use
`cx.repaint_after(duration)`.

## Screen contract

Each screen has `poll(&mut ScreenCtx)` (called from `App::logic`, handles finished tasks and
timers) and `ui(&mut Ui, &mut ScreenCtx)` (draws). `ScreenCtx` gives access to the runtime,
`Services`, the settings store and the alert queue.

## Error handling

All errors from user actions go through the alert queue:

```rust
if let Some(state) = take_ready(&mut self.task) {
    if let TdpState::Error(message) = &state {
        cx.alert(Alert::error("Error title", "User-friendly description", message.clone()));
    }
    self.state = state;
}
```

- No inline error cards, toasts or ad-hoc dialogs.
- Background refresh polls may ignore failures; user actions may not.
- A screen with no data at all shows `widgets::centered_message` with **Retry**.
- Exception: the XG Mobile diagnostic keeps a persistent status/error panel.

## Logging

`RUST_LOG=debug asus-linux-control` traces every backend call:

```text
[Backend] OK asusd.setPlatformProfile mode=2 elapsedMs=4
```

## Tests

`cargo test` in `rust_ui/` runs the feature tests (fakes for all services), settings tests,
single-instance tests and `tests/packaging.rs` (checks the desktop entry, D-Bus activation file
and systemd unit stay consistent).
