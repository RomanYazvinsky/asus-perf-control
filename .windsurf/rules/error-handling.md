# Error Handling Rules

## UI Error Display

**ALL error messages MUST be shown through `CopyableAlert` widget only.**

### Correct Usage

```dart
// In BlocListener - always use CopyableAlert
BlocListener<SomeBloc, SomeState>(
  listener: (context, state) {
    if (state is SomeError) {
      CopyableAlert.showError(
        context,
        title: 'Error Title',
        message: 'User-friendly description',
        errorDetails: state.message,  // Technical details
      );
    }
  },
)
```

### Forbidden Patterns

- ❌ Do NOT use `_buildErrorCard()` or similar inline error widgets
- ❌ Do NOT use `SnackBar` for errors
- ❌ Do NOT use `AlertDialog` directly
- ❌ Do NOT show different error UI based on `_hasLoadedData` flag

### Why CopyableAlert?

1. **Consistent UX** - uniform error appearance across all screens
2. **Copy functionality** - users can copy error details for bug reports
3. **Dismissible** - doesn't block UI like inline cards
4. **Technical details** - separates user message from technical error

### Retry Pattern

For errors with retry action, add a button in the alert content or handle retry through BLoC:

```dart
CopyableAlert.showError(
  context,
  title: 'Failed to Load',
  message: 'Could not fetch data',
  errorDetails: state.message,
);
// Retry via Bloc event from UI button
```
