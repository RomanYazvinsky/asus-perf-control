import 'package:flutter/material.dart';

typedef InputDiagnosticsEventSink = void Function(PointerEvent event);

class InputDiagnostics extends StatelessWidget {
  final Widget child;
  final bool enabled;
  final InputDiagnosticsEventSink? onEvent;

  const InputDiagnostics({
    super.key,
    required this.child,
    this.enabled = false,
    this.onEvent,
  });

  @override
  Widget build(BuildContext context) {
    if (!enabled) {
      return child;
    }

    return Listener(
      behavior: HitTestBehavior.translucent,
      onPointerDown: _handleEvent,
      onPointerUp: _handleEvent,
      onPointerCancel: _handleEvent,
      child: child,
    );
  }

  void _handleEvent(PointerEvent event) {
    onEvent?.call(event);
    debugPrint(
      'input ${event.runtimeType} kind=${event.kind.name} '
      'position=${event.position}',
    );
  }
}
