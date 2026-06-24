# Package Auto-Updates Design

## Context

ASUS Linux Control is a Flutter GUI backed by a privileged Rust daemon. The
daemon is installed as a system service and owns hardware operations through
system D-Bus, polkit, systemd, and udev files.

Flatpak is out of scope for this packaging design. The supported packaging
targets are native Linux packages:

- Fedora: RPM
- Bazzite: RPM layered with rpm-ostree
- Arch Linux: pacman package
- SteamOS: pacman package, documented as an advanced target because host system
  changes can be more fragile on SteamOS

## Reference Instructions

Implementation must follow these upstream instructions and command references:

- Fedora systemd packaging:
  [Fedora Systemd Packaging Guidelines](https://docs.fedoraproject.org/en-US/packaging-guidelines/Systemd/)
- Fedora/RPM scriptlets:
  [Fedora Scriptlets Guidelines](https://docs.fedoraproject.org/en-US/packaging-guidelines/Scriptlets/)
- RPM spec syntax:
  [RPM spec manual](https://rpm-software-management.github.io/rpm/manual/spec.html)
- DNF automatic updates:
  [dnf-automatic documentation](https://dnf.readthedocs.io/en/latest/automatic.html)
- DNF5 update checks:
  [dnf5 check-upgrade](https://dnf5.readthedocs.io/en/latest/commands/check-upgrade.8.html)
- rpm-ostree package layering and updates:
  [rpm-ostree administrator handbook](https://coreos.github.io/rpm-ostree/administrator-handbook/)
- Bazzite rpm-ostree guidance:
  [Bazzite rpm-ostree documentation](https://docs.bazzite.gg/Installing_and_Managing_Software/rpm-ostree/)
- Arch package metadata:
  [PKGBUILD manual](https://man.archlinux.org/man/PKGBUILD.5.en)
- Arch package manager commands:
  [pacman manual](https://man.archlinux.org/man/pacman.8.en)
- systemd daemon and service behavior:
  [systemd daemon documentation](https://www.freedesktop.org/software/systemd/man/latest/daemon.html)
- polkit authorization model:
  [polkit manual](https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html)

If a later implementation needs behavior not covered here, add the relevant
upstream link to this section before implementing that behavior.

## Goals

- Let users install one top-level package and receive both GUI and daemon.
- Support automatic update setups and one-click updates without a custom binary
  downloader.
- Keep updates auditable by using native package managers and signed package
  repositories.
- Prevent the GUI from executing arbitrary privileged commands.
- Keep the existing daemon, D-Bus, and polkit architecture.

## Non-Goals

- Flatpak packaging.
- Downloading and replacing binaries directly from the GUI.
- Running shell scripts fetched from the network.
- Supporting arbitrary package manager commands supplied by the GUI.
- Implementing a full graphical package manager.

## Package Model

The project publishes three packages per native package ecosystem:

- `asus-linux-control`: meta package that depends on GUI and daemon.
- `asus-linux-control-gui`: Flutter desktop application.
- `asus-linux-control-daemon`: Rust daemon plus system integration files.

The daemon package owns:

- `/usr/lib/asus-linux-control/asus-control-daemon`
- `/usr/lib/systemd/system/asus-control-daemon.service`
- `/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service`
- `/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf`
- `/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy`
- `/usr/lib/udev/rules.d/99-asus-control.rules`

The GUI package owns:

- `/usr/lib/asus-linux-control/gui/` Flutter Linux bundle files
- `/usr/bin/asus-linux-control` launcher
- `/usr/share/applications/asus-linux-control.desktop`
- `/usr/share/icons/hicolor/.../apps/asus-linux-control.png`
- `/usr/share/metainfo/asus-linux-control.metainfo.xml`

The top-level `asus-linux-control` package has no payload except package
metadata and dependencies. It is the documented install target for users.

## Packaging Implementation Contract

RPM packages:

- The daemon RPM must install system files under `/usr`, not `/etc`, except for
  files that must be administrator-editable.
- The daemon RPM must use Fedora's systemd scriptlet macros for daemon reload,
  enablement state preservation, and restart handling.
- The daemon RPM must not automatically enable the service unless the selected
  Fedora packaging guidance explicitly allows it for this package type. If not
  enabled automatically, the package must show a post-install instruction.
- The GUI RPM must declare a runtime dependency on `asus-linux-control-daemon`.
- The meta RPM must depend on both GUI and daemon packages.

Pacman packages:

- The daemon package must install system files in the same `/usr` locations as
  RPM where Arch conventions permit it.
- Package lifecycle behavior that cannot be represented in `PKGBUILD` metadata
  must be placed in `asus-linux-control-daemon.install`.
- The install script may run `systemctl daemon-reload` and print enable/start
  instructions.
- The install script must not silently enable unattended updates.
- The GUI package must depend on `asus-linux-control-daemon`.
- The meta package must depend on both GUI and daemon packages.

## Update Architecture

Updates remain repository based:

- Fedora uses `dnf`.
- Bazzite uses `rpm-ostree`.
- Arch Linux uses `pacman`.
- SteamOS uses `pacman` with clear documentation that the target is advanced.

The GUI exposes update status and an update button, but it never runs a package
manager itself. It talks to the already installed root daemon over system D-Bus.

The daemon detects the platform from stable local sources such as
`/etc/os-release` and the available package manager binaries. The daemon maps
the detected platform to a fixed backend. Unknown platforms return an
unsupported status.

Automatic updates are provided only through the operating system's native
package automation listed in the backend rules below. The application may help
users inspect that state, but the GUI and daemon do not implement silent
unattended package installation themselves.

One-click updates are provided by the GUI calling the privileged daemon. This is
interactive and requires polkit authorization.

## D-Bus API

Add update-focused methods to `com.asuslinuxcontrol.Daemon`:

- `GetPackageBackend() -> string`
- `CheckForUpdates() -> string`
- `StartUpdate() -> string`
- `GetUpdateStatus(update_id: string) -> string`
- `CancelUpdate(update_id: string) -> bool`

JSON payloads are used for structured results so the Dart client can evolve
without frequent D-Bus signature changes.

`CheckForUpdates` returns:

- backend name
- installed GUI version
- installed daemon version
- available version, or `null` only when the selected backend cannot expose an
  available version without running an installation
- update availability
- reboot requirement, or `null` only before an update check has completed
- human-readable summary
- machine-readable error code when unsupported or failed

`StartUpdate` returns an opaque update id. The daemon keeps update state in
memory and exposes progress through `GetUpdateStatus`.

The JSON response schema is versioned:

```json
{
  "schemaVersion": 1,
  "backend": "fedora-dnf|fedora-dnf5|bazzite-rpm-ostree|arch-pacman|steamos-pacman|unsupported",
  "state": "idle|checking|available|upToDate|running|succeeded|failed|unsupported",
  "installedVersion": "0.0.1",
  "availableVersion": "0.0.2",
  "rebootRequired": false,
  "summary": "Update available",
  "errorCode": null,
  "logTail": []
}
```

`installedVersion` and `availableVersion` may be `null` only when the backend
cannot determine that value. `errorCode` is `null` on success and one of:

- `unsupported_os`
- `missing_repository`
- `package_manager_unavailable`
- `package_database_locked`
- `signature_error`
- `network_error`
- `authorization_denied`
- `update_failed`
- `reboot_required`

Only one update job may run at a time. `StartUpdate` must return the existing
job id when an update is already running instead of starting another package
manager process.

## Privilege And Security Model

Add a dedicated polkit action:

- `com.asuslinuxcontrol.manage-updates`

`StartUpdate` and `CancelUpdate` require this action. `GetPackageBackend`,
`CheckForUpdates`, and `GetUpdateStatus` are read-only and do not require admin
authorization.

The default policy for update installation must require administrator
authentication for active, inactive, and remote users. Updating system packages
is broader than changing a hardware setting and must not be silently allowed.

The daemon must use an explicit command allowlist:

- Fedora with DNF5:
  - check: `dnf5 check-upgrade asus-linux-control`
  - update: `dnf5 upgrade --assumeyes asus-linux-control`
- Fedora with DNF4:
  - check: `dnf check-update asus-linux-control`
  - update: `dnf upgrade -y asus-linux-control`
- Bazzite:
  - check: `rpm-ostree upgrade --check`
  - update: `rpm-ostree upgrade`
- Arch Linux:
  - check: `pacman -Qu asus-linux-control`
  - update: `pacman -Syu --needed --noconfirm asus-linux-control`
- SteamOS:
  - check: `pacman -Qu asus-linux-control`
  - update: `pacman -Syu --needed --noconfirm asus-linux-control`

The GUI must not provide command names, command arguments, package names, URLs,
or shell fragments. The daemon must execute package managers without `sh -c` and
without shell interpolation.

The daemon must store each allowlisted command as executable plus fixed argument
array. It must not build command strings by concatenation.

## Backend Behavior

Fedora:

- Detect DNF5 first, then DNF4.
- Check updates using DNF metadata for `asus-linux-control`.
- Install only `asus-linux-control`; dependencies bring GUI and daemon updates.
- Support automatic updates through the user's configured DNF automation, such
  as dnf-automatic or desktop software update tooling.
- Return reboot required only when the package manager or system state indicates
  it.

Bazzite:

- Detect Bazzite using `/etc/os-release` before choosing rpm-ostree.
- Use rpm-ostree because Bazzite is image-based.
- Support automatic updates through rpm-ostree or Bazzite's configured system
  update path when enabled by the user.
- Treat successful upgrades as reboot-required when a new deployment is created.
- Prefer clear status text that explains the update is staged until reboot.

Arch Linux:

- Detect Arch using `/etc/os-release`.
- Use pacman against the configured repository.
- Do not enable unattended pacman updates by default.
- Keep package selection limited to ASUS Linux Control packages.
- Report package database or signature failures clearly.

SteamOS:

- Detect SteamOS using `/etc/os-release`.
- Use the same pacman backend only when the environment appears compatible.
- Do not enable unattended pacman updates by default.
- Show advanced-target wording in the GUI and docs.
- Return unsupported when the system is read-only or pacman updates are not
  available.

## Backend Detection Rules

Backend detection order is fixed:

1. If `/etc/os-release` identifies Bazzite, use `bazzite-rpm-ostree`.
2. If `/etc/os-release` identifies SteamOS, use `steamos-pacman` only when
   pacman is available and the package database is writable.
3. If `/etc/os-release` identifies Fedora, use `fedora-dnf5` when `dnf5` exists,
   otherwise `fedora-dnf` when `dnf` exists.
4. If `/etc/os-release` identifies Arch Linux, use `arch-pacman`.
5. Otherwise return `unsupported`.

The daemon must expose the selected backend through `GetPackageBackend` so the
GUI can show the exact update path.

## UI Behavior

Add an updates area in the GUI that shows:

- current package backend
- installed version
- available version
- update availability
- last check result
- update progress and final result
- reboot-required state

The update button is disabled when:

- backend is unsupported
- no update is available
- another update is running
- daemon is unavailable

Errors must be specific enough to act on, for example missing repository,
signature failure, package database lock, unsupported OS, or reboot required.

## Packaging Repository Requirements

RPM and pacman repositories must be signed. Installation documentation must
include repository setup and key trust steps for each ecosystem.

Repository packages must declare dependencies so installing
`asus-linux-control` installs both GUI and daemon. The GUI package may depend on
the daemon package directly, but the meta package remains the recommended user
entry point.

Minimum repository deliverables:

- Fedora/Bazzite:
  - signed RPM packages
  - repository metadata
  - documented repository install command
  - documented GPG key fingerprint
- Arch/SteamOS:
  - signed `.pkg.tar.zst` packages
  - repository database
  - documented `[asus-linux-control]` pacman repository block
  - documented package signing key fingerprint

The update feature is disabled until the relevant repository is configured.
Local package files installed with `dnf install ./file.rpm` or
`pacman -U ./file.pkg.tar.zst` are supported for installation but not for
automatic update checks.

## Testing

Rust unit tests must cover:

- OS/backend detection
- command allowlist construction
- parsing update check output
- update state transitions
- unsupported platform behavior
- JSON schema stability for update status responses
- rejection of unknown package manager backends
- single-running-update enforcement

Dart tests must cover:

- D-Bus update client JSON parsing
- UI state mapping for checking, available, running, failed, unsupported, and
  reboot-required states
- disabled update button states
- daemon-unavailable state

Manual verification must cover:

- Fedora RPM install and update
- Bazzite rpm-ostree install and staged update
- Arch pacman install and update
- SteamOS compatibility or unsupported reporting
- polkit denial handling
- package manager lock handling
- missing repository handling

## Acceptance Criteria

- Installing `asus-linux-control` installs both GUI and daemon on Fedora and
  Arch-compatible targets.
- The daemon exposes the update D-Bus API and never accepts command arguments
  from the GUI.
- `StartUpdate` requires `com.asuslinuxcontrol.manage-updates`.
- Update commands are selected only from the backend allowlist.
- Unknown distributions show an unsupported state, not a generic failure.
- A missing configured repository disables update installation and shows
  `missing_repository`.
- Bazzite updates report reboot-required when rpm-ostree creates a new
  deployment.
- SteamOS either performs the pacman update through the allowlisted command or
  returns unsupported with a clear reason.

## Rollout

Phase 1 creates native packaging and repositories without GUI-triggered update
installation.

Phase 2 adds daemon-backed update checking and status reporting.

Phase 3 adds one-click update installation through the daemon and polkit.

This staged rollout keeps package distribution working before adding privileged
update execution.
