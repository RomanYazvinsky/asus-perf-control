# ASUS Control Daemon (Rust)

D-Bus system daemon для управления оборудованием ASUS ноутбука с Polkit-авторизацией.

## Overview

Daemon предоставляет защищённый D-Bus интерфейс для:

- **Performance mode** (Silent / Balanced / Turbo)
- **TDP limits** — Sustained и Boost (W)
- **CPU temperature ceiling** (°C)
- **Battery charge limit** (20–100%)

Все write-операции требуют Polkit-авторизации. Read — без авторизации.

## Architecture

```
asus-linux-control (rust_ui)
    │ D-Bus call (via rust_ui/src/backend/daemon.rs)
    ▼
asus-control-daemon  ←── Polkit (активный пользователь пишет без пароля)
    │
    ├── hardware/tdp.rs      → ppt_pl1_spl, ppt_pl2_sppt
    ├── hardware/thermal.rs  → throttle_thermal_policy
    └── hardware/battery.rs  → charge_control_end_threshold
```

## Features

- ✅ Native Rust binary (~2–5 MB RAM, нет Python runtime)
- ✅ Type-safe compile-time checks
- ✅ D-Bus activation (запускается по требованию)
- ✅ Полное логирование через `env_logger`

## Building

```bash
cargo build --release
```

Output: `target/release/asus-control-daemon` (~1.5 MB after strip)

## Installation

Installed by `../install/install.sh` to:
- `/usr/lib/asus-linux-control/asus-control-daemon`

## Running Manually (for debugging)

```bash
# Enable debug logging
RUST_LOG=debug sudo ./target/release/asus-control-daemon
```

## D-Bus Interface

**Bus:** `com.asuslinuxcontrol.Daemon`
**Path:** `/com/asuslinuxcontrol/Daemon`

| Метод | Подпись | Auth | Описание |
| --- | --- | --- | --- |
| `GetPerformanceMode` | `→ i32` | нет | 0=Silent, 1=Balanced, 2=Turbo |
| `SetPerformanceMode` | `i32 → b` | polkit | Установить режим |
| `GetBatteryChargeLimit` | `→ i32` | нет | Текущий лимит (20–100%) |
| `SetBatteryChargeLimit` | `i32 → b` | polkit | Установить лимит |
| `GetTdpConfig` | `→ (b,i,i,i)` | нет | isSupported, sustainedW, boostW, tempC *(fallback, см. ниже)* |
| `SetTdpConfig` | `iii → b` | polkit | sustainedW, boostW, tempLimitC *(fallback, см. ниже)* |

> **TDP (GetTdpConfig/SetTdpConfig):** Приложение использует `asusd` (`xyz.ljones.Asusd`) для TDP —
> он персистентен и знает реальные min/max с конкретного железа. Методы в этом daemon оставлены как fallback.

### Polkit actions

- `com.asuslinuxcontrol.set-performance-mode`
- `com.asuslinuxcontrol.set-battery-limit`
- `com.asuslinuxcontrol.set-tdp`

## Testing

```bash
# Получить performance mode
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetPerformanceMode

# Получить TDP config (isSupported, sustainedW, boostW, tempC)
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetTdpConfig

# Установить TDP: 15W sustained, 25W boost, 90°C
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon SetTdpConfig iii 15 25 90

# Установить performance mode
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon SetPerformanceMode i 2
```

## Dependencies

Runtime:
- D-Bus system bus
- Polkit
- ASUS WMI kernel driver (`asus-nb-wmi`)

Build:
- Rust 1.70+
- `zbus` 4.0
- `tokio` runtime

## Security

- Runs as **root** for sysfs access
- Polkit enforces authorization for all write operations
- Read operations do not require authorization
- Input validation on all parameters
- Detailed error messages for troubleshooting

## Logging

Set `RUST_LOG` environment variable:

```bash
# In /usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service
[D-BUS Service]
Name=com.asuslinuxcontrol.Daemon
Exec=/usr/lib/asus-linux-control/asus-control-daemon
User=root
Environment="RUST_LOG=info"
```

Levels: `error`, `warn`, `info`, `debug`, `trace`

View logs:
```bash
journalctl -f | grep asus-control-daemon
```
