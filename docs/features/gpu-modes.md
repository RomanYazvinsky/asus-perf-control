# GPU modes

Screen: **GPU** (`rust_ui/src/ui/screens/gpu.rs`). Requires
[supergfxd](https://gitlab.com/asus-linux/supergfxctl); without it the screen shows an error.

| UI mode | supergfxd mode | Notes |
|---|---|---|
| Integrated | `Integrated` | dGPU powered off |
| Hybrid | `Hybrid` | iGPU renders, dGPU on demand |
| Ultimate | `Hybrid` | shown separately in the UI, maps to Hybrid in supergfxctl |
| MUX dGPU | `AsusMuxDgpu` | dGPU drives the panel directly |
| XG Mobile / eGPU | `AsusEgpu` | ASUS XG Mobile dock |
| VFIO | `Vfio` | dGPU passed through to a VM |

Only modes reported by supergfxd's capabilities are selectable. After a switch supergfxd
returns the action the user must take (log out, reboot, switch to Integrated first, disable
eGPU first), which the screen shows. The selected mode is saved as `hardware.gpuMode`.

Route: `gpu screen → features::gpu → Services → backend::supergfx → org.supergfxctl.Daemon`.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/gpu.rs` |
| Logic | `rust_ui/src/features/gpu.rs` |
| Client | `rust_ui/src/backend/supergfx.rs` |
