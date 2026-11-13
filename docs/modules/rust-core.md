# `rust_core` — hardware readers

A small library linked into `rust_ui`. It only **reads** hardware state; every write goes
through asusd, supergfxd or the own daemon.

```text
rust_core/src/
├── lib.rs               # re-exports
├── api.rs               # facade used by the UI: sensor snapshot, fan RPMs, fan discovery
└── services/
    ├── fan.rs           # ASUS fan discovery (asus-wmi hwmon), RPM reading, FanId
    ├── temp.rs          # CPU / GPU temperature from hwmon (k10temp, coretemp, amdgpu, …)
    ├── cpu.rs           # CPU model name (/proc/cpuinfo), average clock (scaling_cur_freq)
    ├── gpu.rs           # PCI GPU enumeration, pci.ids names, hwmon / nvidia-smi readings
    └── steam_input.rs   # gamepad command types; Steam Input poller (currently a stub)
```

## Notable behaviour

- **GPU reading never wakes a suspended GPU.** `power/runtime_status == suspended` → the GPU is
  reported as inactive without touching hwmon or `nvidia-smi`.
- **pci.ids lookups are cached** per (vendor, device), because the database is ~1 MB.
- **Integrated GPU** = the PCI display device with `boot_vga == 1`; it is listed first.
- All readers return `Option` / empty collections on missing sensors instead of failing.

## Tests

`cargo test` in `rust_core/`: unit tests next to the code (`cpu.rs`, `gpu.rs`) plus
`tests/steam_input_tests.rs`.
