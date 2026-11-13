# Documentation

| Section | What you find there |
|---|---|
| [Architecture](architecture.md) | Processes, crates, who owns which hardware setting, persistence, security model |
| [Features](features/README.md) | One page per user-facing feature: what it does, how it works, where the code lives |
| [Modules](modules/README.md) | Crate-level reference: `rust_ui`, `rust_core`, `rust_daemon` (incl. the D-Bus API), install & packaging files |
| [Guides](guides/) | [Installation](guides/installation.md) · [Development](guides/development.md) · [Troubleshooting](guides/troubleshooting.md) · [Packaging](guides/packaging.md) · [Releases](guides/release.md) |
| [Research](research/) | Hardware investigations and manual verification checklists |

## Research notes

- [`gv301qh-direct-fan-control.md`](research/gv301qh-direct-fan-control.md) — direct fan control on the GV301QH (Flow X13)
- [`fan-override-diagnostic-log.md`](research/fan-override-diagnostic-log.md), [`saio-direct-fan-test-log.md`](research/saio-direct-fan-test-log.md) — XG Mobile / SAIO fan test logs
- [`ghelper-asus-saio-source.cs`](research/ghelper-asus-saio-source.cs) — G-Helper reference source used during the fan research
- [`rog-m4-hotkey-manual-verification.md`](research/rog-m4-hotkey-manual-verification.md) — ROG / M4 key manual test plan
- [`package-update-manual-verification.md`](research/package-update-manual-verification.md) — package update manual test plan per distribution
