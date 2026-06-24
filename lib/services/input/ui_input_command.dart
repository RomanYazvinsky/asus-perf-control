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
