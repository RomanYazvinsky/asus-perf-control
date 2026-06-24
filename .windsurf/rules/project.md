---
trigger: always_on
---

# Project Rules

Before starting any work in this project, ALWAYS read the README.md file at the project root to understand:

- How to build the Rust backend (`cargo build --release` in `rust_core/`)
- How to run the Flutter app (`flutter run -d linux`)
- The project architecture (Flutter + Rust FFI via flutter_rust_bridge)
- Troubleshooting steps (especially: Rust .so must be built before app will start)

Key facts:

- `RustLib.init()` in `main.dart` loads `rust_core/target/release/libasus_linux_control_core.so`
- Without building Rust first, the app will hang on startup
- Never replace real FFI calls with mocks — find and fix the root cause instead
