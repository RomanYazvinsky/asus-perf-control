# Features

Each page covers what the feature does, how it works end to end, and where the code lives.
The UI navigation rail has eight destinations; the toolbar, ROG key, gamepad and single
instance work across all of them.

| Feature | UI destination | Owner |
|---|---|---|
| [Performance mode & CPU boost](performance.md) | Performance | asusd + own daemon |
| [Power limits (TDP) & CPU temperature limit](power-limits.md) | TDP | asusd + own daemon |
| [GPU modes](gpu-modes.md) | GPU | supergfxd |
| [Fan control](fans.md) | Fans | asusd + own daemon |
| [Battery charge limit](battery.md) | Battery | asusd |
| [XG Mobile fan diagnostic](xg-mobile-diagnostic.md) | XG Diag | asusd + sysfs |
| [Package updates](package-updates.md) | Updates | own daemon |
| [Hardware profiles & settings](profiles.md) | Profiles | UI |
| [Status toolbar](status-toolbar.md) | always visible | rust_core |
| [ROG key](rog-key.md) | — | own daemon + hotkey agent |
| [Window, single instance & gamepad](window-and-input.md) | — | UI (gamepad: stub) |

## Common behaviour

- **Desired vs. actual state.** The UI stores the desired value of every setting in the active
  profile. On load it reads the hardware value and, if it differs, re-applies the saved one and
  logs a `Settings.hardwareMismatch` event.
- **Errors** are always shown as a modal alert with copyable technical details. A screen that
  has no data at all shows a neutral placeholder with **Retry**.
- **Unapplied edits** show a sticky action bar (**Apply** / **Reset**) at the bottom of the screen.
