# Status toolbar

Always visible at the top of the window. It refreshes once per second.

```text
CPU: Ryzen 9 5980HS  63°C │ GPU1: Radeon Vega Series  58°C │ GPU2: GeForce GTX 1650 Mobile  off │ Profile
⏱ 3.21 GHz  🌀 2300 RPM  │ ⏱ 2.01 GHz  🌀 2300 RPM       │ ⏱ --  🌀 2100 RPM                 │ Balanced
```

| Column | Source |
|---|---|
| CPU name | `/proc/cpuinfo` `model name`, shortened (`AMD Ryzen 9 5980HS with Radeon Graphics` → `Ryzen 9 5980HS`) |
| CPU temperature | hwmon (`k10temp`, `coretemp`, …) |
| CPU clock | average `scaling_cur_freq` of all online cores |
| GPU list | PCI display controllers (`class 0x03xxxx`), integrated GPU first, at most two |
| GPU name | `pci.ids` (`/usr/share/hwdata/pci.ids` and fallbacks), shortened to the marketing name |
| GPU temperature / clock | the device's hwmon (`edge` sensor, `freq1_input`); `nvidia-smi` for the proprietary NVIDIA driver |
| Fan RPM | CPU fan for the CPU and the integrated GPU, GPU fan for the discrete GPU |
| Profile | asusd platform profile |

- Temperatures above 85 °C are shown in red.
- A runtime-suspended GPU shows `off` and is **not** woken up to be read.
- Hovering a column shows the full PCI name, address and which fan is used.
- Sensor reads run on a blocking thread pool; a missing sensor shows `--`.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/toolbar.rs` |
| Logic | `rust_ui/src/features/telemetry.rs` |
| Readers | `rust_core/src/services/cpu.rs`, `rust_core/src/services/gpu.rs`, `rust_core/src/services/temp.rs`, `rust_core/src/services/fan.rs` |
