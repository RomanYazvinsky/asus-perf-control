# Window, single instance & gamepad

## Single instance

Only one window ever runs.

1. The first instance listens on `$XDG_RUNTIME_DIR/asus-linux-control.sock`.
2. A later launch connects, sends `activate` (or `activate <token>`, see below) and exits.
3. The first instance un-minimises and raises its window.

A socket nobody answers on (a crashed instance) is removed and re-created.

## Raising the window on Wayland

Compositors with focus-stealing prevention (GNOME) ignore a plain "focus me" request and only
show an "is ready" notification. A window is raised only when it presents an
`xdg_activation_v1` token the compositor issued for a user action.

- At start the app takes `XDG_ACTIVATION_TOKEN` from its environment (and removes it so child
  processes don't reuse it).
- A second launch forwards its token to the running instance over the socket.
- The token is presented on the window's own Wayland connection (`rust_ui/src/activation.rs`),
  because winit has no API for activating an existing window with a foreign token.
- Without a token, or on X11, a normal focus request is sent.

## Gamepad navigation (not active yet)

> **Status:** the Steam Input adapter in `rust_core` is a stub (`SteamInputPoller::new_stub`)
> and produces no events in the current build. The command mapping below is already wired
> into the app, so a real adapter only needs to feed `UiInputCommand`s.
> Keyboard navigation (Tab / arrows / Enter) works through egui as usual.

Commands from `rust_core` are turned into egui events by `rust_ui/src/input.rs`:

| Command | Effect |
|---|---|
| D-pad / stick | move focus |
| Activate / Back | press the focused control / go back |
| Next / previous tab | switch navigation destination |
| Apply | apply the current screen's edits |
| Curve commands | select a point and move it in the fan curve editor |

## Code

| Part | Files |
|---|---|
| Single instance | `rust_ui/src/single_instance.rs`, `rust_ui/src/main.rs` |
| Activation | `rust_ui/src/activation.rs`, `raise_window` in `rust_ui/src/app.rs` |
| Gamepad | `rust_ui/src/input.rs`, `rust_core/src/services/steam_input.rs` |
