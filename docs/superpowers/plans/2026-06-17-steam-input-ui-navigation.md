# Steam Input UI Navigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Steam Input driven navigation for the Flutter UI, including focus rings on controls and selectable/editable fan curve points.

**Architecture:** Steamworks stays behind a native/Rust adapter. `rust_core` exposes stable Flutter Rust Bridge DTOs and functions. Flutter converts normalized `UiInputCommand` values into `Intent`/`Action` handlers and regular `Focus` traversal.

**Tech Stack:** Flutter/Dart, Flutter Focus/Actions/Shortcuts APIs, flutter_rust_bridge 2.12, Rust, optional Steamworks SDK native wrapper, fl_chart.

---

## File Structure

- Existing `assets/steam_input/in_game_actions.vdf`: Steam Input action manifest for UI commands.
- Existing `pubspec.yaml`: registers the manifest as a Flutter asset.
- Existing `rust_core/src/services/steam_input.rs`: Rust service facade and unavailable stub.
- Existing `rust_core/src/services/mod.rs`: exports the Steam Input service module.
- Existing `rust_core/src/api.rs`: exposes Steam Input DTOs and stub functions through FRB.
- Existing `rust_core/tests/steam_input_tests.rs`: Rust tests for command normalization and unavailable behavior.
- Existing `lib/widgets/input_diagnostics.dart`: pointer diagnostics only; keep it separate from Steam Input diagnostics.
- Create `lib/services/input/ui_input_command.dart`: Dart enums and DTOs for normalized input.
- Create `lib/services/input/steam_input_service.dart`: Dart polling service using FRB functions.
- Create `lib/widgets/input/gamepad_input_scope.dart`: top-level command router from stream events to Flutter focus/actions.
- Create `lib/widgets/input/gamepad_focusable.dart`: reusable focus ring wrapper for custom controls.
- Create `lib/widgets/input/steam_input_diagnostics.dart`: Steam Input availability/action-set/recent-command diagnostics.
- Modify `lib/main.dart`: wrap the app shell in `GamepadInputScope` and keep pointer diagnostics as a nested optional wrapper.
- Modify `lib/widgets/fan/fan_curve_editor.dart`: add point selection and command-driven editing.
- Modify `lib/screens/fan_screen.dart`: make profile chips, tab area, and fan editor cooperate with focus traversal.
- Create `test/services/steam_input_service_test.dart`: Dart service parsing and fake-service tests.
- Create `test/widgets/gamepad_input_scope_test.dart`: focus traversal and activation widget tests.
- Create `test/widgets/steam_input_diagnostics_test.dart`: diagnostics widget tests.
- Create `test/widgets/fan_curve_editor_gamepad_test.dart`: fan curve point selection/editing widget tests.

## Current Baseline Review

Reviewed against the working tree on 2026-06-24:

- Task 1 is already implemented: `assets/steam_input/in_game_actions.vdf` exists and `pubspec.yaml` includes it under `flutter.assets`.
- Task 2 is already implemented as a stub path: `rust_core/src/services/steam_input.rs`, `rust_core/src/services/mod.rs`, `rust_core/src/api.rs`, and `rust_core/tests/steam_input_tests.rs` are present.
- `lib/main.dart` currently wraps the app in `InputDiagnostics`, but that widget reports pointer events only. It does not satisfy the spec requirement for Steam Input availability, connected controller count, active action set, and recent UI commands.
- The Dart input service, global gamepad routing scope, Steam Input diagnostics UI, fan curve controller editing, and real Steamworks adapter are still not implemented.
- The generated FRB Dart bindings are stale in the current tree: `lib/bridge_generated.dart/api.dart` and `lib/bridge_api.dart` do not expose `getSteamInputStatus`, `pollSteamInputEvents`, `SteamInputStatus`, `SteamInputEvent`, or `UiInputCommand`. Regenerate them before starting Task 3.

---

### Task 1: Add Steam Input Manifest Asset

**Files:**
- Create: `assets/steam_input/in_game_actions.vdf`
- Modify: `pubspec.yaml`

- [x] **Step 1: Add the action manifest file**

Create `assets/steam_input/in_game_actions.vdf`:

```vdf
"In Game Actions"
{
  "actions"
  {
    "ui"
    {
      "title" "#Set_ui"
      "StickPadGyro"
      {
        "focus_move" "joystick_move"
      }
      "Button"
      {
        "focus_up" "digital"
        "focus_down" "digital"
        "focus_left" "digital"
        "focus_right" "digital"
        "activate" "digital"
        "back" "digital"
        "apply" "digital"
        "next_tab" "digital"
        "previous_tab" "digital"
        "curve_previous_point" "digital"
        "curve_next_point" "digital"
        "curve_speed_up" "digital"
        "curve_speed_down" "digital"
        "curve_temp_left" "digital"
        "curve_temp_right" "digital"
      }
    }
  }
  "localization"
  {
    "english"
    {
      "Set_ui" "Application UI"
      "focus_move" "Move focus"
      "focus_up" "Move up"
      "focus_down" "Move down"
      "focus_left" "Move left"
      "focus_right" "Move right"
      "activate" "Activate"
      "back" "Back"
      "apply" "Apply"
      "next_tab" "Next tab"
      "previous_tab" "Previous tab"
      "curve_previous_point" "Previous fan curve point"
      "curve_next_point" "Next fan curve point"
      "curve_speed_up" "Increase fan speed point"
      "curve_speed_down" "Decrease fan speed point"
      "curve_temp_left" "Move fan point colder"
      "curve_temp_right" "Move fan point hotter"
    }
  }
}
```

- [x] **Step 2: Register the asset**

In `pubspec.yaml`, ensure the Flutter section includes:

```yaml
  assets:
    - assets/steam_input/in_game_actions.vdf
```

- [x] **Step 3: Validate asset registration**

Run:

```bash
flutter test test/widgets/input_diagnostics_test.dart
```

Expected: PASS. This confirms the asset addition did not break Flutter test asset resolution.

- [x] **Step 4: Commit or leave staged with the current feature batch**

```bash
git add assets/steam_input/in_game_actions.vdf pubspec.yaml
git commit -m "feat: add Steam Input action manifest"
```

Current tree note: these files already exist. Do not recreate them; verify only if re-running the plan from the current workspace.

---

### Task 2: Add Rust Steam Input Facade With Stub Behavior

**Files:**
- Create: `rust_core/src/services/steam_input.rs`
- Modify: `rust_core/src/services/mod.rs`
- Modify: `rust_core/src/api.rs`
- Test: `rust_core/tests/steam_input_tests.rs`

- [x] **Step 1: Write failing Rust tests**

Create `rust_core/tests/steam_input_tests.rs`:

```rust
#[path = "../src/services/steam_input.rs"]
mod steam_input;

use steam_input::{SteamInputPoller, UiInputCommand};

#[test]
fn stub_reports_unavailable_without_steamworks() {
    let mut poller = SteamInputPoller::new_stub();

    let status = poller.status();

    assert!(!status.available);
    assert_eq!(status.connected_controllers, 0);
    assert_eq!(
        status.reason.as_deref(),
        Some("Steam Input native adapter is not enabled in this build")
    );
}

#[test]
fn command_names_are_stable() {
    assert_eq!(UiInputCommand::FocusUp.as_str(), "focusUp");
    assert_eq!(UiInputCommand::FocusDown.as_str(), "focusDown");
    assert_eq!(UiInputCommand::FocusLeft.as_str(), "focusLeft");
    assert_eq!(UiInputCommand::FocusRight.as_str(), "focusRight");
    assert_eq!(UiInputCommand::Activate.as_str(), "activate");
    assert_eq!(UiInputCommand::Back.as_str(), "back");
    assert_eq!(UiInputCommand::Apply.as_str(), "apply");
    assert_eq!(UiInputCommand::NextTab.as_str(), "nextTab");
    assert_eq!(UiInputCommand::PreviousTab.as_str(), "previousTab");
    assert_eq!(UiInputCommand::CurvePreviousPoint.as_str(), "curvePreviousPoint");
    assert_eq!(UiInputCommand::CurveNextPoint.as_str(), "curveNextPoint");
    assert_eq!(UiInputCommand::CurveSpeedUp.as_str(), "curveSpeedUp");
    assert_eq!(UiInputCommand::CurveSpeedDown.as_str(), "curveSpeedDown");
    assert_eq!(UiInputCommand::CurveTempLeft.as_str(), "curveTempLeft");
    assert_eq!(UiInputCommand::CurveTempRight.as_str(), "curveTempRight");
}
```

- [x] **Step 2: Run tests to verify they fail on a clean pre-Task-2 tree**

Run:

```bash
cd rust_core && cargo test --test steam_input_tests
```

Expected on a clean pre-Task-2 tree: FAIL because `rust_core/src/services/steam_input.rs` does not exist.

Current tree note: `rust_core/src/services/steam_input.rs` already exists, so this command should now PASS instead of fail.

- [x] **Step 3: Add the Rust facade**

Create `rust_core/src/services/steam_input.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInputCommand {
    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,
    Activate,
    Back,
    Apply,
    NextTab,
    PreviousTab,
    CurvePreviousPoint,
    CurveNextPoint,
    CurveSpeedUp,
    CurveSpeedDown,
    CurveTempLeft,
    CurveTempRight,
}

impl UiInputCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FocusUp => "focusUp",
            Self::FocusDown => "focusDown",
            Self::FocusLeft => "focusLeft",
            Self::FocusRight => "focusRight",
            Self::Activate => "activate",
            Self::Back => "back",
            Self::Apply => "apply",
            Self::NextTab => "nextTab",
            Self::PreviousTab => "previousTab",
            Self::CurvePreviousPoint => "curvePreviousPoint",
            Self::CurveNextPoint => "curveNextPoint",
            Self::CurveSpeedUp => "curveSpeedUp",
            Self::CurveSpeedDown => "curveSpeedDown",
            Self::CurveTempLeft => "curveTempLeft",
            Self::CurveTempRight => "curveTempRight",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamInputStatus {
    pub available: bool,
    pub connected_controllers: i32,
    pub active_action_set: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamInputEvent {
    pub command: UiInputCommand,
    pub pressed: bool,
    pub sequence: i64,
}

#[derive(Debug, Default)]
pub struct SteamInputPoller {
    sequence: i64,
}

impl SteamInputPoller {
    pub fn new_stub() -> Self {
        Self { sequence: 0 }
    }

    pub fn status(&self) -> SteamInputStatus {
        SteamInputStatus {
            available: false,
            connected_controllers: 0,
            active_action_set: None,
            reason: Some("Steam Input native adapter is not enabled in this build".to_string()),
        }
    }

    pub fn poll(&mut self) -> Vec<SteamInputEvent> {
        self.sequence += 1;
        Vec::new()
    }
}
```

- [x] **Step 4: Export the module**

In `rust_core/src/services/mod.rs`, add:

```rust
pub mod steam_input;
```

- [x] **Step 5: Expose FRB API**

In `rust_core/src/api.rs`, add near the other DTO definitions:

```rust
pub use crate::services::steam_input::{
    SteamInputEvent, SteamInputStatus, UiInputCommand,
};
```

Then add these functions near the end of `rust_core/src/api.rs`:

```rust
#[flutter_rust_bridge::frb(sync)]
pub fn get_steam_input_status() -> SteamInputStatus {
    crate::services::steam_input::SteamInputPoller::new_stub().status()
}

#[flutter_rust_bridge::frb(sync)]
pub fn poll_steam_input_events() -> Vec<SteamInputEvent> {
    let mut poller = crate::services::steam_input::SteamInputPoller::new_stub();
    poller.poll()
}
```

- [x] **Step 6: Run Rust tests**

Run:

```bash
cd rust_core && cargo test --test steam_input_tests
```

Expected: PASS.

- [x] **Step 7: Regenerate FRB bindings**

Run:

```bash
flutter_rust_bridge_codegen generate \
  --rust-input=crate::api \
  --rust-root=rust_core \
  --dart-output=lib/bridge_generated.dart
```

Expected: generated Dart bindings include `getSteamInputStatus` and `pollSteamInputEvents`.

- [x] **Step 8: Commit or leave staged with the current feature batch**

```bash
git add rust_core/src/services/steam_input.rs rust_core/src/services/mod.rs rust_core/src/api.rs rust_core/tests/steam_input_tests.rs lib/bridge_generated.dart
git commit -m "feat: expose Steam Input facade over FFI"
```

Current tree note: the Rust facade is present and the generated Dart bindings expose
`getSteamInputStatus`, `pollSteamInputEvents`, `SteamInputStatus`,
`SteamInputEvent`, and `UiInputCommand`.

---

### Task 3: Add Dart Input Models And Polling Service

**Files:**
- Create: `lib/services/input/ui_input_command.dart`
- Create: `lib/services/input/steam_input_service.dart`
- Test: `test/services/steam_input_service_test.dart`

- [x] **Step 1: Write failing Dart tests**

Create `test/services/steam_input_service_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';

void main() {
  test('fake service emits queued commands in order', () async {
    final service = FakeSteamInputService();
    final events = <UiInputEvent>[];
    final subscription = service.events.listen(events.add);

    service.emit(UiInputCommand.focusDown);
    service.emit(UiInputCommand.activate);
    await Future<void>.delayed(Duration.zero);

    expect(events.map((event) => event.command), [
      UiInputCommand.focusDown,
      UiInputCommand.activate,
    ]);

    await subscription.cancel();
    service.dispose();
  });

  test('command parser rejects unknown command names', () {
    expect(UiInputCommandX.tryParse('focusUp'), UiInputCommand.focusUp);
    expect(UiInputCommandX.tryParse('notACommand'), isNull);
  });
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
flutter test test/services/steam_input_service_test.dart
```

Expected: FAIL because the service files do not exist.

- [x] **Step 3: Add Dart command models**

Create `lib/services/input/ui_input_command.dart`:

```dart
enum UiInputCommand {
  focusUp,
  focusDown,
  focusLeft,
  focusRight,
  activate,
  back,
  apply,
  nextTab,
  previousTab,
  curvePreviousPoint,
  curveNextPoint,
  curveSpeedUp,
  curveSpeedDown,
  curveTempLeft,
  curveTempRight,
}

extension UiInputCommandX on UiInputCommand {
  static UiInputCommand? tryParse(String value) {
    for (final command in UiInputCommand.values) {
      if (command.name == value) {
        return command;
      }
    }
    return null;
  }
}

class UiInputEvent {
  final UiInputCommand command;
  final bool pressed;
  final int sequence;

  const UiInputEvent({
    required this.command,
    this.pressed = true,
    required this.sequence,
  });
}

class SteamInputAvailability {
  final bool available;
  final int connectedControllers;
  final String? activeActionSet;
  final String? reason;

  const SteamInputAvailability({
    required this.available,
    required this.connectedControllers,
    this.activeActionSet,
    this.reason,
  });
}
```

- [x] **Step 4: Add the polling service**

Create `lib/services/input/steam_input_service.dart`:

```dart
import 'dart:async';

import '../../bridge_api.dart' as ffi;
import 'ui_input_command.dart';

abstract class SteamInputService {
  Stream<UiInputEvent> get events;
  Future<SteamInputAvailability> getStatus();
  void start();
  void dispose();
}

class FfiSteamInputService implements SteamInputService {
  final Duration pollInterval;
  final StreamController<UiInputEvent> _events =
      StreamController<UiInputEvent>.broadcast();
  Timer? _timer;

  FfiSteamInputService({
    this.pollInterval = const Duration(milliseconds: 16),
  });

  @override
  Stream<UiInputEvent> get events => _events.stream;

  @override
  Future<SteamInputAvailability> getStatus() async {
    final status = ffi.getSteamInputStatus();
    return SteamInputAvailability(
      available: status.available,
      connectedControllers: status.connectedControllers,
      activeActionSet: status.activeActionSet,
      reason: status.reason,
    );
  }

  @override
  void start() {
    _timer ??= Timer.periodic(pollInterval, (_) => _poll());
  }

  void _poll() {
    for (final event in ffi.pollSteamInputEvents()) {
      final command = UiInputCommandX.tryParse(event.command.name);
      if (command == null || !event.pressed) {
        continue;
      }
      _events.add(
        UiInputEvent(
          command: command,
          pressed: event.pressed,
          sequence: event.sequence,
        ),
      );
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    _timer = null;
    _events.close();
  }
}

class FakeSteamInputService implements SteamInputService {
  final StreamController<UiInputEvent> _events =
      StreamController<UiInputEvent>.broadcast();
  int _sequence = 0;

  @override
  Stream<UiInputEvent> get events => _events.stream;

  @override
  Future<SteamInputAvailability> getStatus() async {
    return const SteamInputAvailability(
      available: true,
      connectedControllers: 1,
      activeActionSet: 'ui',
    );
  }

  @override
  void start() {}

  void emit(UiInputCommand command) {
    _sequence += 1;
    _events.add(UiInputEvent(command: command, sequence: _sequence));
  }

  @override
  void dispose() {
    _events.close();
  }
}
```

- [x] **Step 5: Run tests**

Run:

```bash
flutter test test/services/steam_input_service_test.dart
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add lib/services/input/ui_input_command.dart lib/services/input/steam_input_service.dart test/services/steam_input_service_test.dart
git commit -m "feat: add Dart Steam Input service"
```

---

### Task 4: Add Global Gamepad Input Scope

**Files:**
- Create: `lib/widgets/input/gamepad_input_scope.dart`
- Create: `lib/widgets/input/gamepad_focusable.dart`
- Modify: `lib/main.dart`
- Test: `test/widgets/gamepad_input_scope_test.dart`

- [x] **Step 1: Write failing widget tests**

Create `test/widgets/gamepad_input_scope_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';
import 'package:flutter_application_1/widgets/input/gamepad_input_scope.dart';

void main() {
  testWidgets('focus commands move focus between controls', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: Column(
            children: [
              ElevatedButton(onPressed: () {}, child: const Text('One')),
              ElevatedButton(onPressed: () {}, child: const Text('Two')),
            ],
          ),
        ),
      ),
    );

    FocusScope.of(tester.element(find.text('One'))).requestFocus(
      Focus.of(tester.element(find.text('One'))),
    );
    await tester.pump();

    service.emit(UiInputCommand.focusDown);
    await tester.pump();

    expect(Focus.of(tester.element(find.text('Two'))).hasFocus, isTrue);
  });

  testWidgets('activate command invokes focused button', (tester) async {
    final service = FakeSteamInputService();
    var pressed = 0;

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: ElevatedButton(
            onPressed: () => pressed += 1,
            child: const Text('Apply'),
          ),
        ),
      ),
    );

    FocusScope.of(tester.element(find.text('Apply'))).requestFocus(
      Focus.of(tester.element(find.text('Apply'))),
    );
    await tester.pump();

    service.emit(UiInputCommand.activate);
    await tester.pump();

    expect(pressed, 1);
  });
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
flutter test test/widgets/gamepad_input_scope_test.dart
```

Expected: FAIL because `GamepadInputScope` does not exist.

- [x] **Step 3: Add focusable wrapper**

Create `lib/widgets/input/gamepad_focusable.dart`:

```dart
import 'package:flutter/material.dart';

class GamepadFocusable extends StatelessWidget {
  final Widget child;
  final bool autofocus;
  final VoidCallback? onActivate;

  const GamepadFocusable({
    super.key,
    required this.child,
    this.autofocus = false,
    this.onActivate,
  });

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return FocusableActionDetector(
      autofocus: autofocus,
      actions: {
        ActivateIntent: CallbackAction<ActivateIntent>(
          onInvoke: (_) {
            onActivate?.call();
            return null;
          },
        ),
      },
      child: Builder(
        builder: (context) {
          final focused = Focus.of(context).hasFocus;
          return AnimatedContainer(
            duration: const Duration(milliseconds: 80),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: focused ? colors.primary : Colors.transparent,
                width: 2,
              ),
            ),
            child: child,
          );
        },
      ),
    );
  }
}
```

- [x] **Step 4: Add input scope**

Create `lib/widgets/input/gamepad_input_scope.dart`:

```dart
import 'dart:async';

import 'package:flutter/material.dart';

import '../../services/input/steam_input_service.dart';
import '../../services/input/ui_input_command.dart';

class GamepadInputScope extends StatefulWidget {
  final Widget child;
  final SteamInputService service;

  const GamepadInputScope({
    super.key,
    required this.child,
    required this.service,
  });

  @override
  State<GamepadInputScope> createState() => _GamepadInputScopeState();
}

class _GamepadInputScopeState extends State<GamepadInputScope> {
  StreamSubscription<UiInputEvent>? _subscription;

  @override
  void initState() {
    super.initState();
    widget.service.start();
    _subscription = widget.service.events.listen(_handleEvent);
  }

  @override
  void dispose() {
    _subscription?.cancel();
    widget.service.dispose();
    super.dispose();
  }

  void _handleEvent(UiInputEvent event) {
    final focus = FocusManager.instance.primaryFocus;
    switch (event.command) {
      case UiInputCommand.focusUp:
        focus?.focusInDirection(TraversalDirection.up);
      case UiInputCommand.focusDown:
        focus?.focusInDirection(TraversalDirection.down);
      case UiInputCommand.focusLeft:
        focus?.focusInDirection(TraversalDirection.left);
      case UiInputCommand.focusRight:
        focus?.focusInDirection(TraversalDirection.right);
      case UiInputCommand.activate:
        final context = focus?.context;
        if (context != null) {
          Actions.invoke(context, const ActivateIntent());
        }
      case UiInputCommand.back:
        final context = focus?.context;
        if (context != null) {
          Actions.invoke(context, const DismissIntent());
        }
      case UiInputCommand.apply:
      case UiInputCommand.nextTab:
      case UiInputCommand.previousTab:
      case UiInputCommand.curvePreviousPoint:
      case UiInputCommand.curveNextPoint:
      case UiInputCommand.curveSpeedUp:
      case UiInputCommand.curveSpeedDown:
      case UiInputCommand.curveTempLeft:
      case UiInputCommand.curveTempRight:
        final context = focus?.context;
        if (context != null) {
          Actions.maybeInvoke(context, UiInputIntent(event.command));
        }
    }
  }

  @override
  Widget build(BuildContext context) {
    return FocusTraversalGroup(
      policy: OrderedTraversalPolicy(),
      child: widget.child,
    );
  }
}

class UiInputIntent extends Intent {
  final UiInputCommand command;

  const UiInputIntent(this.command);
}
```

- [x] **Step 5: Wire the scope in the app**

In `lib/main.dart`, add imports:

```dart
import 'services/input/steam_input_service.dart';
import 'widgets/input/gamepad_input_scope.dart';
```

Near `navigatorKey`, add one shared app-level service instance:

```dart
final SteamInputService steamInputService = FfiSteamInputService();
```

In `MaterialApp.builder`, wrap the existing `InputDiagnostics` result with that shared service:

```dart
        return GamepadInputScope(
          service: steamInputService,
          child: InputDiagnostics(
            enabled: const bool.fromEnvironment('ASUS_INPUT_DIAGNOSTICS'),
            child: child ?? const SizedBox.shrink(),
          ),
        );
```

- [x] **Step 6: Run widget tests**

Run:

```bash
flutter test test/widgets/gamepad_input_scope_test.dart
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add lib/widgets/input/gamepad_input_scope.dart lib/widgets/input/gamepad_focusable.dart lib/main.dart test/widgets/gamepad_input_scope_test.dart
git commit -m "feat: route gamepad commands through Flutter focus"
```

---

### Task 5: Make Fan Curve Points Selectable And Editable

**Files:**
- Modify: `lib/widgets/fan/fan_curve_editor.dart`
- Test: `test/widgets/fan_curve_editor_gamepad_test.dart`

- [x] **Step 1: Write failing fan editor tests**

Create `test/widgets/fan_curve_editor_gamepad_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';
import 'package:flutter_application_1/widgets/fan/fan_curve_editor.dart';
import 'package:flutter_application_1/widgets/input/gamepad_input_scope.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';

List<FanCurvePoint> _curve() => List.generate(
      8,
      (index) => FanCurvePoint(
        tempC: 30 + index * 10,
        speedPercent: 30 + index * 5,
      ),
    );

void main() {
  testWidgets('controller commands select next fan curve point', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: Scaffold(
            body: FanCurveEditor(curve: _curve()),
          ),
        ),
      ),
    );
    await tester.pump();

    FocusScope.of(tester.element(find.byType(FanCurveEditor))).requestFocus(
      Focus.of(tester.element(find.byType(FanCurveEditor))),
    );
    await tester.pump();

    service.emit(UiInputCommand.curveNextPoint);
    await tester.pump();

    expect(find.text('Point 2: 40°C / 35%'), findsOneWidget);
  });

  testWidgets('controller speed command edits selected point', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: Scaffold(
            body: FanCurveEditor(curve: _curve()),
          ),
        ),
      ),
    );
    await tester.pump();

    FocusScope.of(tester.element(find.byType(FanCurveEditor))).requestFocus(
      Focus.of(tester.element(find.byType(FanCurveEditor))),
    );
    await tester.pump();

    service.emit(UiInputCommand.curveSpeedUp);
    await tester.pump();

    expect(find.text('Point 1: 30°C / 31%'), findsOneWidget);
  });
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
flutter test test/widgets/fan_curve_editor_gamepad_test.dart
```

Expected: FAIL because fan curve editor does not handle `UiInputIntent`.

- [x] **Step 3: Add selection state and actions**

In `lib/widgets/fan/fan_curve_editor.dart`, add imports:

```dart
import '../../services/input/ui_input_command.dart';
import '../input/gamepad_input_scope.dart';
```

In `_FanCurveEditorState`, add:

```dart
int? _selectedPointIndex;
```

Wrap the existing `Card` returned by `build` in `FocusableActionDetector` and `Actions`:

```dart
    return Actions(
      actions: {
        UiInputIntent: CallbackAction<UiInputIntent>(
          onInvoke: (intent) {
            _handleUiInputCommand(intent.command);
            return null;
          },
        ),
      },
      child: FocusableActionDetector(
        onFocusChange: (focused) {
          if (focused && _selectedPointIndex == null && _points.isNotEmpty) {
            setState(() => _selectedPointIndex = 0);
          }
        },
        child: Card(
```

Close the two new wrappers after the existing `Card`.

- [x] **Step 4: Add selected point label**

Under the chart and before `Temperature (°C) -> Fan Speed (%)`, add:

```dart
            if (_selectedPointIndex != null) ...[
              const SizedBox(height: 8),
              Text(
                _selectedPointLabel(),
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ],
```

Add this helper:

```dart
String _selectedPointLabel() {
  final index = _selectedPointIndex;
  if (index == null || index < 0 || index >= _points.length) {
    return 'No point selected';
  }
  final point = _points[index];
  return 'Point ${index + 1}: ${point.tempC.round()}°C / ${point.speedPercent.round()}%';
}
```

- [x] **Step 5: Add command handling**

Add this method to `_FanCurveEditorState`:

```dart
void _handleUiInputCommand(UiInputCommand command) {
  if (widget.readOnly || _points.isEmpty) {
    return;
  }

  final current = _selectedPointIndex ?? 0;
  switch (command) {
    case UiInputCommand.curvePreviousPoint:
      setState(() {
        _selectedPointIndex = (current - 1).clamp(0, _points.length - 1);
      });
    case UiInputCommand.curveNextPoint:
      setState(() {
        _selectedPointIndex = (current + 1).clamp(0, _points.length - 1);
      });
    case UiInputCommand.curveSpeedUp:
      _nudgeSelectedPoint(deltaSpeed: 1);
    case UiInputCommand.curveSpeedDown:
      _nudgeSelectedPoint(deltaSpeed: -1);
    case UiInputCommand.curveTempLeft:
      _nudgeSelectedPoint(deltaTemp: -1);
    case UiInputCommand.curveTempRight:
      _nudgeSelectedPoint(deltaTemp: 1);
    case UiInputCommand.focusUp:
    case UiInputCommand.focusDown:
    case UiInputCommand.focusLeft:
    case UiInputCommand.focusRight:
    case UiInputCommand.activate:
    case UiInputCommand.back:
    case UiInputCommand.apply:
    case UiInputCommand.nextTab:
    case UiInputCommand.previousTab:
      return;
  }
}

void _nudgeSelectedPoint({double deltaTemp = 0, double deltaSpeed = 0}) {
  final index = _selectedPointIndex ?? 0;
  final point = _points[index];
  _updatePointPosition(
    index,
    point.tempC + deltaTemp,
    point.speedPercent + deltaSpeed,
  );
}
```

- [x] **Step 6: Render selected points distinctly**

In `FlDotCirclePainter`, change the color decision to:

```dart
                                  color: _draggedPointIndex == index
                                      ? Theme.of(context).colorScheme.secondary
                                      : _selectedPointIndex == index
                                          ? Theme.of(context).colorScheme.tertiary
                                          : Theme.of(context).colorScheme.primary,
                                  strokeWidth: _selectedPointIndex == index ? 4 : 2,
```

- [x] **Step 7: Run fan editor tests**

Run:

```bash
flutter test test/widgets/fan_curve_editor_gamepad_test.dart
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add lib/widgets/fan/fan_curve_editor.dart test/widgets/fan_curve_editor_gamepad_test.dart
git commit -m "feat: make fan curve points controller selectable"
```

---

### Task 6: Add Fan Screen Tab And Apply Commands

**Files:**
- Modify: `lib/screens/fan_screen.dart`
- Test: `test/screens/fan_screen_test.dart`

- [ ] **Step 1: Add regression tests**

Append to `test/screens/fan_screen_test.dart`:

```dart
testWidgets('fan screen exposes apply button as focusable control', (tester) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: FanTabContent(
          fanId: FanId.cpu,
          state: _loadedState(cpuMode: FanControlMode.asusctlCurve),
        ),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 100));

  final apply = find.widgetWithText(ElevatedButton, 'Apply');
  expect(apply, findsOneWidget);
  expect(Focus.of(tester.element(apply)).canRequestFocus, isTrue);
});
```

- [ ] **Step 2: Run the focused test**

Run:

```bash
flutter test test/screens/fan_screen_test.dart
```

Expected: PASS before code changes for the focusable Apply button, giving a baseline.

- [ ] **Step 3: Add tab command handling**

In `lib/screens/fan_screen.dart`, add imports:

```dart
import '../services/input/ui_input_command.dart';
import '../widgets/input/gamepad_input_scope.dart';
```

Wrap the `Column` returned by `_FanContentState.build` in:

```dart
    return Actions(
      actions: {
        UiInputIntent: CallbackAction<UiInputIntent>(
          onInvoke: (intent) {
            if (intent.command == UiInputCommand.nextTab) {
              final next = (_tabController.index + 1) % _tabController.length;
              _tabController.animateTo(next);
            }
            if (intent.command == UiInputCommand.previousTab) {
              final previous =
                  (_tabController.index - 1 + _tabController.length) %
                  _tabController.length;
              _tabController.animateTo(previous);
            }
            return null;
          },
        ),
      },
      child: Column(
```

Close `Actions` after the `Column`.

- [ ] **Step 4: Run fan screen tests**

Run:

```bash
flutter test test/screens/fan_screen_test.dart
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add lib/screens/fan_screen.dart test/screens/fan_screen_test.dart
git commit -m "feat: support controller tab commands on fan screen"
```

---

### Task 7: Add Steam Input Diagnostics UI

**Files:**
- Create: `lib/widgets/input/steam_input_diagnostics.dart`
- Modify: `lib/main.dart`
- Test: `test/widgets/steam_input_diagnostics_test.dart`

- [ ] **Step 1: Write failing diagnostics widget tests**

Create `test/widgets/steam_input_diagnostics_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';
import 'package:flutter_application_1/widgets/input/steam_input_diagnostics.dart';

void main() {
  testWidgets('shows Steam Input status from service', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: SteamInputDiagnostics(
          enabled: true,
          service: service,
          child: const SizedBox.shrink(),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Steam Input: available'), findsOneWidget);
    expect(find.text('Controllers: 1'), findsOneWidget);
    expect(find.text('Action set: ui'), findsOneWidget);

    service.dispose();
  });

  testWidgets('records recent controller commands', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: SteamInputDiagnostics(
          enabled: true,
          service: service,
          child: const SizedBox.shrink(),
        ),
      ),
    );
    await tester.pump();

    service.emit(UiInputCommand.focusDown);
    await tester.pump();

    expect(find.text('Recent: focusDown'), findsOneWidget);

    service.dispose();
  });
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
flutter test test/widgets/steam_input_diagnostics_test.dart
```

Expected: FAIL because `SteamInputDiagnostics` does not exist.

- [ ] **Step 3: Add the diagnostics widget**

Create `lib/widgets/input/steam_input_diagnostics.dart`:

```dart
import 'dart:async';

import 'package:flutter/material.dart';

import '../../services/input/steam_input_service.dart';
import '../../services/input/ui_input_command.dart';

class SteamInputDiagnostics extends StatefulWidget {
  final Widget child;
  final bool enabled;
  final SteamInputService service;

  const SteamInputDiagnostics({
    super.key,
    required this.child,
    required this.service,
    this.enabled = false,
  });

  @override
  State<SteamInputDiagnostics> createState() => _SteamInputDiagnosticsState();
}

class _SteamInputDiagnosticsState extends State<SteamInputDiagnostics> {
  SteamInputAvailability? _status;
  StreamSubscription<UiInputEvent>? _subscription;
  final List<UiInputCommand> _recent = [];

  @override
  void initState() {
    super.initState();
    if (widget.enabled) {
      _loadStatus();
      _subscription = widget.service.events.listen(_recordEvent);
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }

  Future<void> _loadStatus() async {
    final status = await widget.service.getStatus();
    if (mounted) {
      setState(() => _status = status);
    }
  }

  void _recordEvent(UiInputEvent event) {
    if (!event.pressed) {
      return;
    }
    setState(() {
      _recent.insert(0, event.command);
      if (_recent.length > 5) {
        _recent.removeLast();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.enabled) {
      return widget.child;
    }

    final status = _status;
    final label = status == null
        ? 'Steam Input: checking'
        : status.available
            ? 'Steam Input: available'
            : 'Steam Input: unavailable';
    final actionSet = status?.activeActionSet ?? 'none';
    final recent = _recent.isEmpty
        ? 'Recent: none'
        : 'Recent: ${_recent.map((command) => command.name).join(', ')}';

    return Stack(
      children: [
        widget.child,
        Positioned(
          right: 12,
          bottom: 12,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: Theme.of(context).colorScheme.outline),
            ),
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: DefaultTextStyle(
                style: Theme.of(context).textTheme.bodySmall!,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(label),
                    Text('Controllers: ${status?.connectedControllers ?? 0}'),
                    Text('Action set: $actionSet'),
                    if (status?.reason case final reason?) Text(reason),
                    Text(recent),
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
```

- [ ] **Step 4: Wire Steam diagnostics separately from pointer diagnostics**

In `lib/main.dart`, add:

```dart
import 'widgets/input/steam_input_diagnostics.dart';
```

Then update `MaterialApp.builder` so `GamepadInputScope` and diagnostics both use the shared `steamInputService` instance:

```dart
      builder: (context, child) {
        ErrorWidget.builder = (FlutterErrorDetails details) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            final ctx = navigatorKey.currentContext;
            if (ctx != null) {
              CopyableAlert.showError(
                ctx,
                title: 'Application Error',
                message: 'An unexpected error occurred in the UI',
                errorDetails: details.exception.toString(),
              );
            }
          });
          return const SizedBox.shrink();
        };
        return GamepadInputScope(
          service: steamInputService,
          child: SteamInputDiagnostics(
            enabled: const bool.fromEnvironment('ASUS_STEAM_INPUT_DIAGNOSTICS'),
            service: steamInputService,
            child: InputDiagnostics(
              enabled: const bool.fromEnvironment('ASUS_INPUT_DIAGNOSTICS'),
              child: child ?? const SizedBox.shrink(),
            ),
          ),
        );
      },
```

- [ ] **Step 5: Run diagnostics tests**

Run:

```bash
flutter test test/widgets/steam_input_diagnostics_test.dart
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add lib/widgets/input/steam_input_diagnostics.dart lib/main.dart test/widgets/steam_input_diagnostics_test.dart
git commit -m "feat: add Steam Input diagnostics overlay"
```

---

### Task 8: Add Real Steamworks Native Adapter Behind The Rust Facade

**Files:**
- Create: `rust_core/native/steam_input_adapter.h`
- Create: `rust_core/native/steam_input_adapter.cpp`
- Create: `rust_core/build.rs`
- Modify: `rust_core/Cargo.toml`
- Modify: `rust_core/src/services/steam_input.rs`

- [ ] **Step 1: Add build script dependency**

In `rust_core/Cargo.toml`, add:

```toml
build = "build.rs"

[build-dependencies]
cc = "1.0"
```

- [ ] **Step 2: Add the C ABI header**

Create `rust_core/native/steam_input_adapter.h`:

```cpp
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SteamInputNativeStatus {
  bool available;
  int32_t connected_controllers;
  const char* reason;
} SteamInputNativeStatus;

typedef struct SteamInputNativeEvent {
  int32_t command;
  bool pressed;
  int64_t sequence;
} SteamInputNativeEvent;

bool alc_steam_input_init(const char* manifest_path);
SteamInputNativeStatus alc_steam_input_status();
int32_t alc_steam_input_poll(SteamInputNativeEvent* events, int32_t max_events);
void alc_steam_input_shutdown();

#ifdef __cplusplus
}
#endif
```

- [ ] **Step 3: Add the native adapter implementation**

Create `rust_core/native/steam_input_adapter.cpp`:

```cpp
#include "steam_input_adapter.h"

#if defined(ALC_ENABLE_STEAM_INPUT)
#include "steam/steam_api.h"
#endif

namespace {
int64_t g_sequence = 0;
bool g_initialized = false;
const char* g_reason = "Steam Input native adapter has not been initialized";
}

bool alc_steam_input_init(const char* manifest_path) {
#if defined(ALC_ENABLE_STEAM_INPUT)
  if (!SteamAPI_Init()) {
    g_reason = "SteamAPI_Init failed";
    g_initialized = false;
    return false;
  }
  if (manifest_path != nullptr && manifest_path[0] != '\0') {
    SteamInput()->SetInputActionManifestFilePath(manifest_path);
  }
  if (!SteamInput()->Init()) {
    g_reason = "SteamInput Init failed";
    g_initialized = false;
    return false;
  }
  g_initialized = true;
  g_reason = nullptr;
  return true;
#else
  (void)manifest_path;
  g_reason = "Steam Input native adapter is not enabled in this build";
  g_initialized = false;
  return false;
#endif
}

SteamInputNativeStatus alc_steam_input_status() {
  SteamInputNativeStatus status{};
  status.available = g_initialized;
  status.reason = g_reason;
#if defined(ALC_ENABLE_STEAM_INPUT)
  if (g_initialized) {
    InputHandle_t handles[STEAM_INPUT_MAX_COUNT] = {};
    status.connected_controllers = SteamInput()->GetConnectedControllers(handles);
  }
#endif
  return status;
}

int32_t alc_steam_input_poll(SteamInputNativeEvent* events, int32_t max_events) {
  if (events == nullptr || max_events <= 0 || !g_initialized) {
    return 0;
  }
#if defined(ALC_ENABLE_STEAM_INPUT)
  SteamAPI_RunCallbacks();
  SteamInput()->RunFrame();
#endif
  g_sequence += 1;
  return 0;
}

void alc_steam_input_shutdown() {
#if defined(ALC_ENABLE_STEAM_INPUT)
  if (g_initialized) {
    SteamInput()->Shutdown();
    SteamAPI_Shutdown();
  }
#endif
  g_initialized = false;
  g_reason = "Steam Input native adapter has been shut down";
}
```

- [ ] **Step 4: Add build script**

Create `rust_core/build.rs`:

```rust
fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("native/steam_input_adapter.cpp")
        .include("native");

    if let Ok(sdk_dir) = std::env::var("STEAMWORKS_SDK_DIR") {
        build.define("ALC_ENABLE_STEAM_INPUT", None);
        build.include(format!("{sdk_dir}/public"));
        println!("cargo:rustc-link-search=native={sdk_dir}/redistributable_bin/linux64");
        println!("cargo:rustc-link-lib=dylib=steam_api");
    }

    build.compile("alc_steam_input_adapter");
}
```

- [ ] **Step 5: Replace Rust stub internals with native calls**

In `rust_core/src/services/steam_input.rs`, add:

```rust
#[repr(C)]
struct NativeStatus {
    available: bool,
    connected_controllers: i32,
    reason: *const std::os::raw::c_char,
}

#[repr(C)]
struct NativeEvent {
    command: i32,
    pressed: bool,
    sequence: i64,
}

extern "C" {
    fn alc_steam_input_init(manifest_path: *const std::os::raw::c_char) -> bool;
    fn alc_steam_input_status() -> NativeStatus;
    fn alc_steam_input_poll(events: *mut NativeEvent, max_events: i32) -> i32;
    fn alc_steam_input_shutdown();
}
```

Then update `SteamInputPoller::status()` to call `alc_steam_input_status()` and convert the native reason with:

```rust
fn native_reason(ptr: *const std::os::raw::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let value = unsafe { std::ffi::CStr::from_ptr(ptr) };
    Some(value.to_string_lossy().to_string())
}
```

Keep `poll()` returning an empty vector until Task 9 maps action handles. This keeps the build verifiable before action mapping is added.

- [ ] **Step 6: Verify default build without SDK**

Run:

```bash
cd rust_core && cargo test --test steam_input_tests
```

Expected: PASS without `STEAMWORKS_SDK_DIR`.

- [ ] **Step 7: Verify SDK build when available**

Run:

```bash
cd rust_core && STEAMWORKS_SDK_DIR=/absolute/path/to/steamworks_sdk cargo build
```

Expected: PASS when the SDK exists at the provided path. If the SDK is not installed, skip this command and record the skip in the task notes.

- [ ] **Step 8: Commit**

```bash
git add rust_core/Cargo.toml rust_core/build.rs rust_core/native/steam_input_adapter.h rust_core/native/steam_input_adapter.cpp rust_core/src/services/steam_input.rs
git commit -m "feat: add native Steam Input adapter"
```

---

### Task 9: Map Steam Input Actions To UI Commands

**Files:**
- Modify: `rust_core/native/steam_input_adapter.cpp`
- Modify: `rust_core/src/services/steam_input.rs`
- Test: `rust_core/tests/steam_input_tests.rs`

- [ ] **Step 1: Extend Rust command conversion tests**

Append to `rust_core/tests/steam_input_tests.rs`:

```rust
#[test]
fn native_command_ids_map_to_ui_commands() {
    assert_eq!(UiInputCommand::from_native_id(1), Some(UiInputCommand::FocusUp));
    assert_eq!(UiInputCommand::from_native_id(15), Some(UiInputCommand::CurveTempRight));
    assert_eq!(UiInputCommand::from_native_id(99), None);
}
```

- [ ] **Step 2: Add native command IDs in Rust**

In `rust_core/src/services/steam_input.rs`, add:

```rust
impl UiInputCommand {
    pub fn from_native_id(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::FocusUp),
            2 => Some(Self::FocusDown),
            3 => Some(Self::FocusLeft),
            4 => Some(Self::FocusRight),
            5 => Some(Self::Activate),
            6 => Some(Self::Back),
            7 => Some(Self::Apply),
            8 => Some(Self::NextTab),
            9 => Some(Self::PreviousTab),
            10 => Some(Self::CurvePreviousPoint),
            11 => Some(Self::CurveNextPoint),
            12 => Some(Self::CurveSpeedUp),
            13 => Some(Self::CurveSpeedDown),
            14 => Some(Self::CurveTempLeft),
            15 => Some(Self::CurveTempRight),
            _ => None,
        }
    }
}
```

- [ ] **Step 3: Add action handle lookup in native adapter**

In `rust_core/native/steam_input_adapter.cpp`, add static handles:

```cpp
#if defined(ALC_ENABLE_STEAM_INPUT)
InputActionSetHandle_t g_ui_set = 0;
InputDigitalActionHandle_t g_actions[16] = {};
#endif
```

After successful `SteamInput()->Init()`, add:

```cpp
#if defined(ALC_ENABLE_STEAM_INPUT)
  g_ui_set = SteamInput()->GetActionSetHandle("ui");
  g_actions[1] = SteamInput()->GetDigitalActionHandle("focus_up");
  g_actions[2] = SteamInput()->GetDigitalActionHandle("focus_down");
  g_actions[3] = SteamInput()->GetDigitalActionHandle("focus_left");
  g_actions[4] = SteamInput()->GetDigitalActionHandle("focus_right");
  g_actions[5] = SteamInput()->GetDigitalActionHandle("activate");
  g_actions[6] = SteamInput()->GetDigitalActionHandle("back");
  g_actions[7] = SteamInput()->GetDigitalActionHandle("apply");
  g_actions[8] = SteamInput()->GetDigitalActionHandle("next_tab");
  g_actions[9] = SteamInput()->GetDigitalActionHandle("previous_tab");
  g_actions[10] = SteamInput()->GetDigitalActionHandle("curve_previous_point");
  g_actions[11] = SteamInput()->GetDigitalActionHandle("curve_next_point");
  g_actions[12] = SteamInput()->GetDigitalActionHandle("curve_speed_up");
  g_actions[13] = SteamInput()->GetDigitalActionHandle("curve_speed_down");
  g_actions[14] = SteamInput()->GetDigitalActionHandle("curve_temp_left");
  g_actions[15] = SteamInput()->GetDigitalActionHandle("curve_temp_right");
#endif
```

- [ ] **Step 4: Poll digital actions**

Replace the body of `alc_steam_input_poll` after `RunFrame()` with:

```cpp
#if defined(ALC_ENABLE_STEAM_INPUT)
  InputHandle_t handles[STEAM_INPUT_MAX_COUNT] = {};
  int controller_count = SteamInput()->GetConnectedControllers(handles);
  int written = 0;
  for (int controller = 0; controller < controller_count; controller++) {
    SteamInput()->ActivateActionSet(handles[controller], g_ui_set);
    for (int command = 1; command <= 15 && written < max_events; command++) {
      InputDigitalActionData_t data =
          SteamInput()->GetDigitalActionData(handles[controller], g_actions[command]);
      if (data.bActive && data.bState) {
        g_sequence += 1;
        events[written].command = command;
        events[written].pressed = true;
        events[written].sequence = g_sequence;
        written += 1;
      }
    }
  }
  return written;
#else
  return 0;
#endif
```

- [ ] **Step 5: Convert native events in Rust**

Update `SteamInputPoller::poll()`:

```rust
pub fn poll(&mut self) -> Vec<SteamInputEvent> {
    let mut native_events: Vec<NativeEvent> = (0..32)
        .map(|_| NativeEvent {
            command: 0,
            pressed: false,
            sequence: 0,
        })
        .collect();

    let count = unsafe {
        alc_steam_input_poll(native_events.as_mut_ptr(), native_events.len() as i32)
    };

    native_events
        .into_iter()
        .take(count.max(0) as usize)
        .filter_map(|event| {
            UiInputCommand::from_native_id(event.command).map(|command| SteamInputEvent {
                command,
                pressed: event.pressed,
                sequence: event.sequence,
            })
        })
        .collect()
}
```

- [ ] **Step 6: Run tests**

Run:

```bash
cd rust_core && cargo test --test steam_input_tests
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add rust_core/native/steam_input_adapter.cpp rust_core/src/services/steam_input.rs rust_core/tests/steam_input_tests.rs
git commit -m "feat: map Steam Input actions to UI commands"
```

---

### Task 10: Final Verification

**Files:**
- Verify only.

- [ ] **Step 1: Run Dart tests for input and fan editor**

Run:

```bash
flutter test test/services/steam_input_service_test.dart test/widgets/gamepad_input_scope_test.dart test/widgets/steam_input_diagnostics_test.dart test/widgets/fan_curve_editor_gamepad_test.dart test/screens/fan_screen_test.dart
```

Expected: PASS.

- [ ] **Step 2: Run Rust tests**

Run:

```bash
cd rust_core && cargo test --test steam_input_tests
```

Expected: PASS.

- [ ] **Step 3: Run formatter**

Run:

```bash
dart format lib test
cargo fmt --manifest-path rust_core/Cargo.toml
```

Expected: source files are formatted.

- [ ] **Step 4: Run analyzer**

Run:

```bash
flutter analyze
```

Expected: no new analyzer errors from the Steam Input changes.

- [ ] **Step 5: Manual Linux smoke test without Steam**

Run:

```bash
flutter run -d linux
```

Expected: app launches, mouse/keyboard UI remains usable, Steam Input diagnostics report unavailable with a clear reason.

- [ ] **Step 6: Manual Steam smoke test with SDK/development App ID**

Run the app through Steam or with the development App ID setup used by the Steamworks SDK.

Expected:

- Controller connects in diagnostics.
- D-pad or stick moves focus between controls.
- A activates focused buttons/switches/chips.
- Shoulder or configured tab actions switch fan tabs.
- Fan curve selected point changes with previous/next point actions.
- Fan curve speed/temp actions modify the selected point and keep safety clamping.
- Apply commits the edited curve.

- [ ] **Step 7: Commit final verification notes if docs changed**

If manual verification notes are added to docs, run:

```bash
git add docs
git commit -m "docs: record Steam Input verification notes"
```
