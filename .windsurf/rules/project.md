---
trigger: always_on
---

# Project Rules

Before starting any work in this project, read `README.md` and the relevant pages in `docs/`:

- `docs/architecture.md` — processes, ownership rules, persistence, security model
- `docs/features/` — one page per feature
- `docs/modules/` — crate reference, including the daemon's D-Bus API
- `docs/guides/development.md` — build/test loops and conventions

Key facts:

- `rust_ui` links `rust_core` directly as a Rust library; there is no FFI layer
- Screen logic lives in `rust_ui/src/features/` behind service traits and is unit-tested with fakes
- Never replace real D-Bus or sysfs calls with mocks in production code — find and fix the root cause instead
- `RUST_LOG=debug` traces every D-Bus call
