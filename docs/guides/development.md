# Development

## Toolchain

- Rust 1.95+ (required by egui 0.36), Cargo
- `make`, `sudo`
- Optional: `rpmbuild`, `makepkg` for packages

Each crate (`rust_ui`, `rust_core`, `rust_daemon`) is built separately; there is no workspace.

## Make targets

| Command | What it does |
|---|---|
| `make run` | Build (release) and run the UI |
| `make debug` | Debug build of the UI with `RUST_LOG=debug` |
| `make test` | `cargo test` in all three crates |
| `make install` | Build daemon + UI and install them with all system files |
| `make build-ui` / `make build-daemon` | Build one part |
| `make package-rpm` / `make package-arch` / `make package-all` | Native packages → `dist/` |
| `make clean` | Remove `target/` directories and `dist/` |

## Typical loops

UI or `rust_core` change:

```bash
make test
(cd rust_ui && cargo clippy --all-targets -- -D warnings)
make run
```

Daemon change:

```bash
make install                              # rebuild + reinstall + restart the unit
journalctl -u asus-control-daemon -f      # watch it
```

The UI also starts without the daemon; privileged actions then report an error.

## Conventions

- **Layering.** `ui/` draws, `features/` decides, `backend/` talks D-Bus. Feature code must not
  know D-Bus paths, sysfs paths or CLI arguments. See [rust_ui](../modules/rust-ui.md).
- **Ownership.** Use asusd / supergfxd for what they own; add daemon methods only for
  privileged behaviour they don't cover. See [architecture](../architecture.md#ownership-rules).
- **No mocks in production code.** Fakes live in tests only. If a real D-Bus or sysfs call
  fails, fix the root cause.
- **Errors** go through the alert queue (`ScreenCtx::alert`), never inline cards or toasts.
- **Never block the UI thread**: spawn on the runtime, poll the `Task`.
- **New daemon write methods** need a Polkit action in
  `install/dbus/com.asuslinuxcontrol.Daemon.policy`, input validation, and a SELinux rule if
  they touch new files or devices.
- **Tests first** for feature logic: features are tested against fake services
  (`rust_ui/src/features/*/tests.rs`), daemon modules in `rust_daemon/tests/`.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`,
`test:`, `chore:`, optionally scoped (`fix(daemon): …`, `feat(ui): …`).
Releases: [release guide](release.md).

## Debugging

```bash
RUST_LOG=debug asus-linux-control             # every backend call with timing
sudo systemctl stop asus-control-daemon
sudo RUST_LOG=debug ASUS_ROG_HOTKEY_ENABLED=1 rust_daemon/target/release/asus-control-daemon
busctl monitor --system --match "sender='com.asuslinuxcontrol.Daemon'"
```
