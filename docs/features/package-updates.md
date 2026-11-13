# Package updates

Screen: **Updates** (`rust_ui/src/ui/screens/updates.rs`)

Checks for and installs updates of the `asus-linux-control` packages through the operating
system's package manager. The GUI never downloads binaries and never sends commands: the
daemon picks a fixed backend and a fixed command allowlist.

| Distribution | Backend | Notes |
|---|---|---|
| Fedora | `dnf5` or `dnf` | |
| Bazzite | `rpm-ostree` | update is staged; reboot to activate |
| Arch Linux | `pacman` | |
| SteamOS | `pacman` | reports *unsupported* when the host filesystem is read-only |
| anything else | — | *unsupported* |

Flow:

1. `GetPackageBackend` and `CheckForUpdates` → status JSON (installed / available versions).
2. `StartUpdate` → returns an update id; the screen polls `GetUpdateStatus(id)` until the job
   finishes. `CancelUpdate(id)` stops it.

Polkit action: `com.asuslinuxcontrol.manage-updates`.

Automatic updates require the signed project repository to be configured; locally installed
package files (`dnf install ./x.rpm`, `pacman -U`) are install-only. See
[packaging](../guides/packaging.md).

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/updates.rs` |
| Logic | `rust_ui/src/features/updates.rs` |
| Client | `rust_ui/src/backend/daemon.rs` |
| Daemon | `rust_daemon/src/package_updates/` (`backend.rs` detection, `commands.rs` allowlist, `job.rs`, `parser.rs`, `status.rs`) |

Manual test plan: [`package-update-manual-verification.md`](../research/package-update-manual-verification.md).
