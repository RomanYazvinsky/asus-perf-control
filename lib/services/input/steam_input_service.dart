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

  FfiSteamInputService({this.pollInterval = const Duration(milliseconds: 16)});

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
