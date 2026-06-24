import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Alert dialog with copyable text functionality
class CopyableAlert extends StatelessWidget {
  final String title;
  final String message;
  final String? copyableText;
  final IconData icon;
  final Color iconColor;
  final List<Widget>? actions;

  const CopyableAlert({
    super.key,
    required this.title,
    required this.message,
    this.copyableText,
    this.icon = Icons.info,
    this.iconColor = Colors.blue,
    this.actions,
  });

  /// Show error alert with copy functionality
  static Future<void> showError(
    BuildContext context, {
    required String title,
    required String message,
    String? errorDetails,
  }) {
    return showDialog(
      context: context,
      builder: (context) => CopyableAlert(
        title: title,
        message: message,
        copyableText: errorDetails,
        icon: Icons.error,
        iconColor: Colors.red,
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Закрыть'),
          ),
        ],
      ),
    );
  }

  /// Show success alert with copy functionality
  static Future<void> showSuccess(
    BuildContext context, {
    required String title,
    required String message,
    String? details,
  }) {
    return showDialog(
      context: context,
      builder: (context) => CopyableAlert(
        title: title,
        message: message,
        copyableText: details,
        icon: Icons.check_circle,
        iconColor: Colors.green,
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  /// Show info alert with copy functionality
  static Future<void> showInfo(
    BuildContext context, {
    required String title,
    required String message,
    String? details,
  }) {
    return showDialog(
      context: context,
      builder: (context) => CopyableAlert(
        title: title,
        message: message,
        copyableText: details,
        icon: Icons.info,
        iconColor: Colors.blue,
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Понятно'),
          ),
        ],
      ),
    );
  }

  void _copyToClipboard(BuildContext context) {
    if (copyableText != null) {
      Clipboard.setData(ClipboardData(text: copyableText!));
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Скопировано в буфер обмена'),
          duration: Duration(seconds: 2),
        ),
      );
    }
  }

  /// Reusable widget for copyable text with copy button.
  /// Use this to embed copyable text in any UI (cards, dialogs, etc).
  static Widget buildCopyableText(
    BuildContext context, {
    required String text,
    Color? textColor,
    Color? backgroundColor,
  }) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color:
            backgroundColor ??
            Theme.of(
              context,
            ).colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withValues(alpha: 0.3),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: SelectableText(
              text,
              style: TextStyle(
                fontFamily: 'monospace',
                fontSize: 12,
                color:
                    textColor ?? Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
          const SizedBox(width: 8),
          IconButton(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: text));
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Скопировано в буфер обмена'),
                  duration: Duration(seconds: 2),
                ),
              );
            },
            icon: const Icon(Icons.copy, size: 16),
            tooltip: 'Копировать',
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      icon: Icon(icon, color: iconColor, size: 32),
      title: Text(title),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(message),
          if (copyableText != null) ...[
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Theme.of(
                    context,
                  ).colorScheme.outline.withValues(alpha: 0.3),
                ),
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: SelectableText(
                      copyableText!,
                      style: TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    onPressed: () => _copyToClipboard(context),
                    icon: const Icon(Icons.copy, size: 16),
                    tooltip: 'Копировать',
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(
                      minWidth: 24,
                      minHeight: 24,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
      actions:
          actions ??
          [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Закрыть'),
            ),
          ],
    );
  }
}

/// Simple snackbar with copy functionality
class CopyableSnackBar {
  static void show(
    BuildContext context, {
    required String message,
    String? copyableText,
    Duration duration = const Duration(seconds: 4),
  }) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        duration: duration,
        content: Row(
          children: [
            Expanded(child: Text(message)),
            if (copyableText != null)
              IconButton(
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: copyableText));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Скопировано в буфер обмена'),
                      duration: Duration(seconds: 2),
                    ),
                  );
                },
                icon: const Icon(Icons.copy, size: 16),
                tooltip: 'Копировать детали',
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
              ),
          ],
        ),
        action: SnackBarAction(
          label: 'Закрыть',
          onPressed: () {
            ScaffoldMessenger.of(context).hideCurrentSnackBar();
          },
        ),
      ),
    );
  }
}
