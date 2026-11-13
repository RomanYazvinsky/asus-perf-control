# Packaging & distribution

Native packages are the recommended way to install and update. Flatpak, a custom binary
downloader and GUI-provided package-manager commands are intentionally not supported: the
daemon needs system integration (D-Bus, Polkit, systemd, udev, SELinux), and updates must go
through the distribution's signed repositories.

## Building

```bash
make package-rpm     # Fedora / Bazzite → dist/rpmbuild/RPMS/, dist/rpmbuild/SRPMS/
make package-arch    # Arch / SteamOS   → dist/arch/
make package-all     # both
```

Both build from a source tarball with pre-built release binaries (`make source-tarball`).
Package layout and contents: [install & packaging module](../modules/install-and-packaging.md).

## Repositories

### Fedora & Bazzite (RPM)

Deliverables: signed `asus-linux-control`, `-gui` and `-daemon` RPMs, signed repository
metadata, and a published GPG key fingerprint.

Bazzite layers the packages with rpm-ostree. Updates are staged as a new deployment and become
active after a reboot.

### Arch & SteamOS (pacman)

Deliverables: signed `.pkg.tar.zst` packages for the meta, GUI and daemon packages, a signed
repository database, and a published signing key fingerprint.

```ini
[asus-linux-control]
SigLevel = Required DatabaseOptional
Server = https://example.invalid/asus-linux-control/$arch
```

SteamOS is an advanced target. Systems that keep the host read-only report an unsupported
update backend.

## Local package files

`dnf install ./file.rpm` or `pacman -U ./file.pkg.tar.zst` work for installing only.
In-app update checks need the signed repository.

## Before publishing

Run [`package-update-manual-verification.md`](../research/package-update-manual-verification.md)
for each ecosystem: install and repository update on Fedora, Bazzite, Arch, and SteamOS (or a
clear "unsupported" status), plus Polkit denial, package manager lock, missing repository and
signature failure.
