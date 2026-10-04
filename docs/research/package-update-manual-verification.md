# Package Update Manual Verification

Use signed test repositories and throwaway systems or snapshots for these
checks. Local package files are valid for install smoke tests only; update
checks require a configured repository.

## Fedora RPM

- Install the signed repository file and verify the documented GPG fingerprint.
- Install `asus-linux-control`.
- Verify `asus-linux-control-gui` and `asus-linux-control-daemon` are installed.
- Verify `asus-control-daemon.service` is installed but not silently enabled.
- Publish a newer signed package to the repository.
- Open the Updates screen and verify the backend is `fedora-dnf` or
  `fedora-dnf5`.
- Check for updates and install the update through the GUI.

## Bazzite rpm-ostree

- Install the signed RPM repository and verify the documented GPG fingerprint.
- Layer `asus-linux-control`.
- Publish a newer signed package to the repository.
- Verify the Updates screen reports `bazzite-rpm-ostree`.
- Install the update and verify the UI reports that a restart is required.
- Reboot and verify the new deployment contains the updated package.

## Arch pacman

- Add the signed `[asus-linux-control]` repository block.
- Verify the package signing key fingerprint before trusting it.
- Install `asus-linux-control`.
- Publish a newer signed package and repository database.
- Verify the Updates screen reports `arch-pacman`.
- Install the update through the GUI and verify package versions changed.

## SteamOS

- Add the signed pacman repository only on a test system where host package
  changes are acceptable.
- Verify the package signing key fingerprint before trusting it.
- If pacman updates are available and the package database is writable, verify
  the backend is `steamos-pacman`.
- If the host is read-only or pacman updates are unavailable, verify the UI
  reports unsupported instead of a generic failure.

## Failure Cases

- Deny the polkit prompt for `StartUpdate` and verify the UI shows
  authorization denied.
- Hold the native package database lock and verify the UI shows package
  database locked.
- Remove or disable the configured repository and verify the UI shows missing
  repository.
- Serve a package or repository metadata with a bad signature and verify the UI
  shows signature verification failed.
