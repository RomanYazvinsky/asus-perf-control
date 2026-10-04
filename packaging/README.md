# Native Packaging And Updates

ASUS Linux Control publishes native packages for repository-backed installs and
updates. The recommended install target is the top-level `asus-linux-control`
package, which depends on both the desktop GUI and the privileged daemon.

## Fedora And Bazzite RPM Repository

Repository deliverables:

- signed `asus-linux-control`, `asus-linux-control-gui`, and
  `asus-linux-control-daemon` RPM packages
- signed repository metadata
- documented GPG key fingerprint before users import the key

Bazzite uses rpm-ostree layered packages. Updates are staged as new deployments
and generally require a reboot before the updated GUI and daemon are active.

## Arch And SteamOS Pacman Repository

Repository deliverables:

- signed `.pkg.tar.zst` packages for the meta, GUI, and daemon packages
- signed repository database
- documented package signing key fingerprint before users trust the key

Example repository block:

```ini
[asus-linux-control]
SigLevel = Required DatabaseOptional
Server = https://example.invalid/asus-linux-control/$arch
```

SteamOS is an advanced target. Some systems may keep the host filesystem
read-only or otherwise unsuitable for pacman package updates; those systems
should report an unsupported update backend.

## Local Package Files

Local files installed with `dnf install ./file.rpm` or
`pacman -U ./file.pkg.tar.zst` are supported only for installation. Automatic
or one-click update checks require the matching signed repository to be
configured.

## Update Model

The GUI never downloads binaries or runs arbitrary privileged commands. Update
checks and one-click updates are delegated to the installed system daemon, which
selects a fixed native package-manager backend and fixed command allowlist.

## Manual Verification

Before publishing a repository, run the checklist in
`docs/research/package-update-manual-verification.md` for each target ecosystem:

- Fedora RPM install and repository update
- Bazzite rpm-ostree layered install and staged update
- Arch pacman install and repository update
- SteamOS supported update or clear unsupported status
- polkit denial, package manager lock, missing repository, and signature failure
