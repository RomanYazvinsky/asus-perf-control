---
trigger: always_on
---

# Project Rules

Before starting any work in this project, ALWAYS read the README.md file at the project root to understand:

- How to build and run the desktop app (`make run`, i.e. `cargo run --release` in `rust_ui/`)
- The project architecture (Rust egui UI in `rust_ui/`, sysfs readers in `rust_core/`, privileged D-Bus daemon in `rust_daemon/`)
- Troubleshooting steps (`RUST_LOG=debug` traces every D-Bus call)

Key facts:

- `rust_ui` links `rust_core` directly as a Rust library; there is no FFI layer
- Screen logic lives in `rust_ui/src/features/` behind service traits and is unit-tested with fakes
- Never replace real D-Bus or sysfs calls with mocks in production code — find and fix the root cause instead
