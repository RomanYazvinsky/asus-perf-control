# Release Versioning

ASUS Linux Control uses semantic versioning for application and native package
releases:

- `MAJOR` changes for incompatible user-facing, D-Bus, package, or persisted
  configuration behavior.
- `MINOR` changes for backward-compatible features.
- `PATCH` changes for backward-compatible fixes.

The first semantic-versioned release is `0.0.1`.

## Source Of Truth

The release version must stay synchronized across:

- `rust_ui/Cargo.toml`: desktop application version.
- `Makefile`: source archive and package build version.
- `packaging/rpm/asus-linux-control.spec`: RPM package version and changelog.
- `packaging/arch/PKGBUILD`: pacman package version.

Cargo uses `version = "<semver>"`. Native packages use `<semver>` and their
ecosystem-specific package release number. For example, `0.0.1` maps to RPM
`0.0.1-1` and pacman `pkgver=0.0.1`, `pkgrel=1`.

## Git Tags

Release tags use `v<semver>`, for example `v0.0.1`.

Use Conventional Commits for changes after the initial project import:

- `feat:` for backward-compatible user-visible features.
- `fix:` for backward-compatible fixes.
- `docs:` for documentation-only changes.
- `test:` for test-only changes.
- `chore:` for versioning, packaging metadata, repository maintenance, and
  generated release bookkeeping.
