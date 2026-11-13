# Modules

| Module | Kind | Page |
|---|---|---|
| `rust_ui/` | binary `asus-linux-control` | [rust-ui.md](rust-ui.md) |
| `rust_core/` | library | [rust-core.md](rust-core.md) |
| `rust_daemon/` | binaries `asus-control-daemon`, `asus-control-hotkey-agent` | [rust-daemon.md](rust-daemon.md) |
| `install/`, `packaging/` | system integration files | [install-and-packaging.md](install-and-packaging.md) |

```text
rust_ui ──depends on──► rust_core
rust_daemon (independent crate, talks to rust_ui only over D-Bus)
```

Each crate has its own `Cargo.toml` and `target/`; there is no Cargo workspace.
