# Error Handling Rules

## UI Error Display

**ALL error messages MUST be shown through the alert queue (`rust_ui/src/ui/alert.rs`).**

Screens report errors with `ScreenCtx::alert(Alert::error(...))`; the app shell shows
them one at a time in a modal with selectable details and a "Копировать" button.

### Correct Usage

```rust
// In a screen's poll(): react to a finished task, like the former BlocListener
if let Some(state) = take_ready(&mut self.task) {
    if let TdpState::Error(message) = &state {
        cx.alert(Alert::error(
            "Error Title",
            "User-friendly description",
            message.clone(), // technical details, copyable
        ));
    }
    self.state = state;
}
```

### Forbidden Patterns

- ❌ Do NOT render error text inline in cards or banners on regular screens
- ❌ Do NOT use toasts/notifications for errors
- ❌ Do NOT build ad-hoc `egui::Window`/`egui::Modal` dialogs for errors — use `Alert`
- ❌ Do NOT log-and-ignore errors from user actions (background refresh polls may ignore failures)

### Allowed Exceptions

- When a screen has **no data at all** because loading failed, it shows a neutral
  placeholder with a **Retry** button (`widgets::centered_message`) after raising the alert.
  This replaces the former infinite spinner.
- Long-running diagnostics (XG Mobile) keep a persistent status/error panel, because the
  result must stay visible next to the captured runs.

### Why the alert queue?

1. **Consistent UX** - uniform error appearance across all screens
2. **Copy functionality** - users can copy error details for bug reports
3. **Dismissible** - doesn't block UI like inline cards
4. **Technical details** - separates user message from technical error
