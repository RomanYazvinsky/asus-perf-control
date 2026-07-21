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
    _subscribe(widget.service);
  }

  @override
  void didUpdateWidget(GamepadInputScope oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.service != widget.service) {
      _subscription?.cancel();
      oldWidget.service.dispose();
      _subscribe(widget.service);
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    widget.service.dispose();
    super.dispose();
  }

  void _subscribe(SteamInputService service) {
    service.start();
    _subscription = service.events.listen(_handleEvent);
  }

  void _handleEvent(UiInputEvent event) {
    final focus = FocusManager.instance.primaryFocus;
    switch (event.command) {
      case UiInputCommand.focusUp:
        _moveFocus(TraversalDirection.up);
      case UiInputCommand.focusDown:
        _moveFocus(TraversalDirection.down);
      case UiInputCommand.focusLeft:
        _moveFocus(TraversalDirection.left);
      case UiInputCommand.focusRight:
        _moveFocus(TraversalDirection.right);
      case UiInputCommand.activate:
        final context = focus?.context;
        if (context != null) {
          Actions.invoke(context, const ActivateIntent());
        }
      case UiInputCommand.back:
        final context = focus?.context;
        if (context != null) {
          Actions.maybeInvoke(context, const DismissIntent());
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

  void _moveFocus(TraversalDirection direction) {
    final focus = FocusManager.instance.primaryFocus;
    if (focus?.focusInDirection(direction) ?? false) {
      return;
    }

    final scope = FocusScope.of(context);
    switch (direction) {
      case TraversalDirection.up:
      case TraversalDirection.left:
        scope.previousFocus();
      case TraversalDirection.down:
      case TraversalDirection.right:
        scope.nextFocus();
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
