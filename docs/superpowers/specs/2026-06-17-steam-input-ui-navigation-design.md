# Steam Input UI Navigation Design

**Goal:** Add full controller navigation for the Flutter desktop interface through Steam Input, with visible focus rings on controls and selectable/editable fan curve points.

**Decision:** Use the existing Flutter Rust Bridge direction for the app boundary. Dart receives normalized UI commands from `rust_core`; Steamworks-specific details stay behind a Rust service and a small native C ABI wrapper if direct Rust bindings are not sufficient.

---

## Scope

This feature controls the application UI, not a gameplay mode. Steam Input maps physical controller inputs to action names. The app maps those action names to UI commands such as focus movement, activation, tab switching, applying changes, and fan curve point editing.

In scope:

- Focus rings for buttons, switches, chips, tab controls, and custom controls.
- Directional focus traversal with controller stick or d-pad.
- Activation and back/cancel commands.
- Next/previous tab commands.
- Fan curve editor point selection.
- Fan curve editor point movement by controller commands.
- A diagnostics view or debug surface showing Steam Input availability, active controller count, active action set, and recent UI commands.
- Fallback behavior when Steam is unavailable or the app is not launched through Steam.

Out of scope:

- Replacing all mouse/touch behavior.
- Steam achievements, overlay, cloud, or store integration.
- Steam Input glyph rendering in every button label.
- Full controller remapping UI inside the app; remapping stays in Steam.

## Architecture

Steam Input integration has three layers.

The native layer initializes Steamworks, calls `SteamInput()->RunFrame()`, reads action states, and exposes normalized snapshots to Rust. It is allowed to be a thin C ABI wrapper around the official Steamworks SDK because Steamworks is C/C++ oriented.

The Rust layer lives under `rust_core/src/services/steam_input.rs`. It owns Steam Input lifecycle from the app point of view, converts native action states to stable DTOs, and exposes synchronous FRB functions in `rust_core/src/api.rs`. The Dart layer never sees Steamworks handles.

The Flutter layer maps `UiInputCommand` values into `Intent`/`Action` handlers and regular `Focus` traversal. Custom widgets, especially the fan curve editor, implement explicit focusable behavior instead of relying on mouse gesture state.

## Command Model

The app uses a small command vocabulary:

- `focusUp`
- `focusDown`
- `focusLeft`
- `focusRight`
- `activate`
- `back`
- `apply`
- `nextTab`
- `previousTab`
- `curvePreviousPoint`
- `curveNextPoint`
- `curveSpeedUp`
- `curveSpeedDown`
- `curveTempLeft`
- `curveTempRight`

Digital commands are edge-triggered for button presses. Directional analog commands use repeat timing so holding a stick moves focus predictably without flooding the UI.

## Action Manifest

The action manifest defines one required action set named `ui`. The manifest includes digital actions for activation, back, apply, tab switching, and curve point selection. It includes analog or directional actions for focus movement. If Steam Input analog data is awkward to consume in the first implementation, the manifest can expose directional digital actions for d-pad and left-stick directions; Steam Input still lets users remap those actions.

The app must package the manifest and a development `steam_appid.txt` path for local runs. Release packaging should place these next to the executable or in the Steam depot layout expected by Steamworks.

## Flutter Focus Design

Standard Material controls should keep using Flutter focus where possible. The feature adds a small wrapper for consistent visual state:

- `GamepadFocusable` for custom controls and card-like click targets.
- `GamepadInputScope` near `MaterialApp.builder` or `AppShell` to subscribe to Steam input events.
- `UiInputIntent` and `UiInputAction` classes to route commands.

Focus rings are drawn by Flutter, not by Steam Input. The default ring should be visible in the current dark theme: primary-colored outline, 2 px width, 4 px radius, and no layout shift.

## Fan Curve Editor Design

`FanCurveEditor` currently supports mouse drag only. It needs an explicit selection model:

- `_selectedPointIndex` tracks the controller-selected point.
- `_isKeyboardEditingPoint` or equivalent tracks whether directional commands edit the selected point or move focus out of the chart.
- Focus entering the chart selects the first point if none is selected.
- `curvePreviousPoint` and `curveNextPoint` change `_selectedPointIndex`.
- `curveSpeedUp` and `curveSpeedDown` adjust speed by 1 percent per command.
- `curveTempLeft` and `curveTempRight` adjust temperature by 1 degree per command.
- Existing clamping and monotonic safety logic remains the single source of truth for point updates.
- Selected point rendering is separate from dragged point rendering.

The Apply button remains the commit point. Controller edits update the local curve and call `onCurveChanged` only when Apply is activated, matching current mouse behavior.

## Error Handling

Steam Input can be unavailable when:

- Steam is not running.
- The app is not launched through Steam and no development App ID is present.
- The Steamworks shared library is missing.
- The action manifest path is invalid.
- No controller is connected.

The UI should remain fully usable with mouse and keyboard in all these cases. Diagnostics should expose the reason. Normal screens should not show modal errors just because Steam Input is unavailable.

## Testing Strategy

Dart tests cover the behavior that matters to users:

- Commands move Flutter focus.
- Activate triggers focused buttons/switches/chips.
- Tab commands switch fan tabs.
- Fan curve point selection works.
- Fan curve point edits preserve clamping and monotonic safety rules.

Rust tests cover command normalization and unavailable-state behavior without loading Steamworks. Native Steamworks calls are isolated behind an adapter so CI and local tests can run without the SDK.

## Open Implementation Risk

The main risk is Steamworks SDK linking and distribution. The plan should make Steam Input an opt-in native feature or provide a stub build path so `cargo test`, `flutter test`, and normal development builds do not fail when the SDK is absent.

## Self-Review

- Placeholder scan: no placeholder sections remain.
- Internal consistency: the design consistently treats FFI as the app boundary and Steamworks as native implementation detail.
- Scope check: this is one feature area with native input, Dart routing, and fan editor behavior; it can be implemented incrementally.
- Ambiguity check: Steam Input controls the app UI, and the fan curve points are explicitly selectable/editable.
