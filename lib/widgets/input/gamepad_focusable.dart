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
      actions: onActivate == null
          ? const {}
          : {
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
