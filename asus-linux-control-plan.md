# План разработки ASUS Linux Control Tool (Flutter + Rust)

## 📋 Обзор проекта

Аналог G-Helper для Linux с GUI на Flutter и backend на Rust для управления ноутбуками ASUS (ROG, TUF, Zephyrus и др.).

---

## 🔍 Анализ G-Helper

### Основные функции:
1. **Режимы производительности**: Silent, Balanced, Turbo
2. **Режимы GPU**: Eco, Standard, Ultimate, Optimized
3. **Управление вентиляторами**: Кастомные кривые, мониторинг RPM
4. **Частота обновления экрана** + Display Overdrive
5. **RGB подсветка**: Anime Matrix, клавиатура, периферия
6. **Мониторинг**: CPU/GPU температура, вентиляторы, батарея
7. **Лимит заряда батареи** (80% для продления срока службы)
8. **GPU разгон/undervolt** (Nvidia/AMD)
9. **Горячие клавиши**: Fn-комбинации
10. **Управление мышами ASUS** (Chakram, Gladius, Keris и др.)
11. **AMD CPU Undervolting**
12. **XG Mobile Control**

### Windows API (что использует G-Helper):
- **ASUS WMI Interface** - проприетарный драйвер ASUS для Windows
- **NvAPIWrapper** - Nvidia GPU API
- **Ryzen SMU** - AMD CPU управление

---

## 🐧 Linux эквиваленты и API

### 1. **Ядро Linux - ASUS WMI Driver**
**Путь**: `/sys/class/platform/asus-nb-wmi/` и `/sys/devices/platform/asus-nb-wmi/`

**Доступные интерфейсы**:
```
/sys/class/platform/asus-nb-wmi/
├── throttle_thermal_policy     # Режимы производительности (0=Silent, 1=Balanced, 2=Turbo)
├── panel_od                    # Display overdrive
├── dgpu_disable                # Отключение dGPU (Eco mode)
├── egpu_enable                 # XG Mobile control
├── charge_control_end_threshold # Лимит заряда батареи (0-100)
```

**Документация**: `/Documentation/ABI/testing/sysfs-platform-asus-wmi`

### 2. **Hardware Monitoring (hwmon)**
**Путь**: `/sys/class/hwmon/hwmonX/`

**Функции**:
- Чтение температуры CPU/GPU: `temp*_input`
- Скорость вентиляторов: `fan*_input` (RPM)
- Управление PWM: `pwm*` (0-255)
- Установка кривых: `pwm*_auto_point*_pwm`, `pwm*_auto_point*_temp`

**Библиотека**: `libsensors` (C), wrapper через Rust FFI

### 3. **GPU Control**

#### Nvidia:
- **NVML Library** (Nvidia Management Library)
  - C API: `libnvidia-ml.so`
  - Мониторинг, разгон через nvidia-settings/nvapi
  
#### AMD:
- **ROCm** / **amdgpu** драйвер
- Sysfs: `/sys/class/drm/card*/device/`
  - `pp_power_profile_mode` - режимы мощности
  - `pp_od_clk_voltage` - разгон
  - `hwmon` подсистема для температур

### 4. **Подсветка**

#### Anime Matrix / Slash Lighting:
- **Через AsusCtl** (существующий проект)
- Прямой доступ через USB HID: `/dev/hidraw*`
- Протокол: USB пакеты (см. Starlight project)

#### Клавиатура RGB:
```
/sys/class/leds/asus::kbd_backlight/
├── brightness           # 0-3
├── brightness_hw_changed
```

### 5. **Батарея**

**Sysfs интерфейс**:
```
/sys/class/power_supply/BAT0/
├── charge_control_end_threshold   # Лимит заряда
├── status                         # Charging/Discharging
├── capacity                       # % заряда
├── voltage_now
├── current_now
```

**UPower D-Bus** (альтернатива):
- Service: `org.freedesktop.UPower`
- Объект: `/org/freedesktop/UPower/devices/battery_BAT0`

### 6. **Частота экрана**

**Через X11/Wayland**:
- `xrandr` для X11
- Wayland compositor protocols

**Библиотеки**:
- `libxrandr` (X11)
- `wayland-client` (Wayland)

---

## 🦀 Rust Libraries и Crates

### Core System Integration:

1. **sysfs-class** (v0.1.3+)
   - Работа с `/sys/class/hwmon/`
   ```toml
   sysfs-class = "0.1"
   ```

2. **libmedium** (v0.2+)
   - Удобная обертка над lm_sensors
   ```toml
   libmedium = "0.2"
   ```

3. **nix** (v0.27+)
   - POSIX системные вызовы, файловые операции
   ```toml
   nix = { version = "0.27", features = ["fs", "ioctl"] }
   ```

4. **zbus** (v3.0+)
   - D-Bus коммуникация (UPower, logind)
   ```toml
   zbus = "3.14"
   tokio = { version = "1", features = ["full"] }
   ```

5. **battery** (v0.7+)
   - Кроссплатформенная информация о батарее
   ```toml
   battery = "0.7"
   ```

### USB/HID для RGB:

6. **hidapi** (v2.4+)
   - Доступ к USB HID устройствам (Anime Matrix, мыши)
   ```toml
   hidapi = "2.4"
   ```

7. **rusb** (v0.9+)
   - libusb биндинги (low-level USB)
   ```toml
   rusb = "0.9"
   ```

### GPU Libraries:

8. **nvml-wrapper** (v0.9+)
   - Nvidia GPU через NVML
   ```toml
   nvml-wrapper = "0.9"
   ```

9. **Custom AMD wrapper**
   - Прямой доступ к sysfs AMD GPU
   - Писать самостоятельно через std::fs

### Flutter-Rust Bridge:

10. **flutter_rust_bridge** (v2.0+)
    - FFI биндинги Flutter ↔ Rust
    ```toml
    flutter_rust_bridge = "2.0"
    ```

### Async Runtime:

11. **tokio** (v1.35+)
    - Асинхронный runtime для фоновых задач
    ```toml
    tokio = { version = "1", features = ["full"] }
    ```

### Serialization:

12. **serde** + **serde_json**
    - Передача данных между Flutter и Rust
    ```toml
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    ```

### Logging:

13. **tracing** + **tracing-subscriber**
    ```toml
    tracing = "0.1"
    tracing-subscriber = "0.3"
    ```

---

## 🏗️ Архитектура проекта

```
asus-linux-control/
├── flutter_app/                 # Flutter GUI
│   ├── lib/
│   │   ├── main.dart
│   │   ├── bridge_generated.dart  # Авто-генерируется FRB
│   │   ├── screens/
│   │   │   ├── dashboard.dart
│   │   │   ├── performance.dart
│   │   │   ├── fans.dart
│   │   │   ├── rgb.dart
│   │   │   └── settings.dart
│   │   └── widgets/
│   │       ├── fan_curve_editor.dart
│   │       └── temperature_chart.dart
│   └── pubspec.yaml
│
├── rust_core/                   # Rust Backend
│   ├── src/
│   │   ├── lib.rs              # FFI entry point
│   │   ├── api.rs              # Flutter-вызываемые функции
│   │   ├── platform/
│   │   │   ├── mod.rs
│   │   │   ├── performance.rs   # Режимы производительности
│   │   │   ├── gpu.rs           # GPU управление
│   │   │   ├── fans.rs          # Вентиляторы
│   │   │   ├── battery.rs       # Батарея
│   │   │   ├── display.rs       # Частота экрана
│   │   │   └── rgb.rs           # RGB подсветка
│   │   ├── monitor/
│   │   │   ├── sensors.rs       # Мониторинг (async)
│   │   │   └── events.rs        # Системные события
│   │   ├── config/
│   │   │   └── profiles.rs      # Сохранение настроек
│   │   └── bindings/
│   │       ├── nvml.rs          # Nvidia биндинги
│   │       ├── hidapi_wrap.rs   # USB HID
│   │       └── sysfs_helpers.rs # Sysfs утилиты
│   └── Cargo.toml
│
├── native/                      # C bindings (если нужны)
│   ├── libsensors/             # lm_sensors wrapper
│   └── nvml/                   # NVML wrapper
│
└── docs/
    ├── api.md
    └── hardware-support.md
```

---

## 📦 Основные компоненты Rust Backend

### 1. **Performance Module** (`platform/performance.rs`)

```rust
use std::fs;

pub enum PerformanceMode {
    Silent = 0,
    Balanced = 1,
    Turbo = 2,
}

pub fn set_performance_mode(mode: PerformanceMode) -> Result<(), String> {
    fs::write(
        "/sys/class/platform/asus-nb-wmi/throttle_thermal_policy",
        format!("{}", mode as u8)
    ).map_err(|e| e.to_string())
}

pub fn get_performance_mode() -> Result<PerformanceMode, String> {
    let content = fs::read_to_string(
        "/sys/class/platform/asus-nb-wmi/throttle_thermal_policy"
    ).map_err(|e| e.to_string())?;
    
    match content.trim().parse::<u8>() {
        Ok(0) => Ok(PerformanceMode::Silent),
        Ok(1) => Ok(PerformanceMode::Balanced),
        Ok(2) => Ok(PerformanceMode::Turbo),
        _ => Err("Unknown mode".into())
    }
}
```

### 2. **Fan Control** (`platform/fans.rs`)

```rust
use libmedium::{Hwmon, Hwmons};

pub struct FanController {
    hwmon: Hwmon,
}

impl FanController {
    pub fn new() -> Result<Self, String> {
        let hwmons = Hwmons::parse_read_only()
            .map_err(|e| e.to_string())?;
        
        // Находим ASUS hwmon
        let hwmon = hwmons.into_iter()
            .find(|(_, name, _)| name.contains("asus"))
            .ok_or("ASUS hwmon not found")?
            .2;
        
        Ok(Self { hwmon })
    }
    
    pub fn get_fan_speed(&self, fan_id: u8) -> Result<u32, String> {
        // Чтение RPM
        Ok(0) // Placeholder
    }
    
    pub fn set_fan_curve(&self, points: Vec<(u8, u8)>) -> Result<(), String> {
        // Установка кривой через pwm_auto_point
        Ok(())
    }
}
```

### 3. **GPU Control** (`platform/gpu.rs`)

```rust
use nvml_wrapper::Nvml;

pub enum GpuMode {
    Eco,        // iGPU only
    Standard,   // Hybrid
    Ultimate,   // dGPU primary
}

pub fn set_gpu_mode(mode: GpuMode) -> Result<(), String> {
    match mode {
        GpuMode::Eco => {
            fs::write("/sys/class/platform/asus-nb-wmi/dgpu_disable", "1")
        },
        GpuMode::Standard | GpuMode::Ultimate => {
            fs::write("/sys/class/platform/asus-nb-wmi/dgpu_disable", "0")
        }
    }.map_err(|e| e.to_string())
}

pub fn get_nvidia_info() -> Result<GpuInfo, String> {
    let nvml = Nvml::init().map_err(|e| e.to_string())?;
    let device = nvml.device_by_index(0).map_err(|e| e.to_string())?;
    
    Ok(GpuInfo {
        temperature: device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .ok(),
        utilization: device.utilization_rates().ok().map(|u| u.gpu),
    })
}
```

### 4. **Battery** (`platform/battery.rs`)

```rust
use battery::{Manager, State};

pub fn set_charge_limit(limit: u8) -> Result<(), String> {
    if limit > 100 {
        return Err("Limit must be 0-100".into());
    }
    
    fs::write(
        "/sys/class/power_supply/BAT0/charge_control_end_threshold",
        format!("{}", limit)
    ).map_err(|e| e.to_string())
}

pub fn get_battery_info() -> Result<BatteryInfo, String> {
    let manager = Manager::new().map_err(|e| e.to_string())?;
    let battery = manager.batteries()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("No battery found")??;
    
    Ok(BatteryInfo {
        percentage: battery.state_of_charge().value * 100.0,
        state: match battery.state() {
            State::Charging => "Charging",
            State::Discharging => "Discharging",
            _ => "Unknown",
        }.to_string(),
    })
}
```

### 5. **RGB/Anime Matrix** (`platform/rgb.rs`)

```rust
use hidapi::{HidApi, HidDevice};

const ANIME_MATRIX_VID: u16 = 0x0b05;  // ASUS vendor ID
const ANIME_MATRIX_PID: u16 = 0x193b;  // Example product ID

pub struct AnimeMatrix {
    device: HidDevice,
}

impl AnimeMatrix {
    pub fn new() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| e.to_string())?;
        let device = api.open(ANIME_MATRIX_VID, ANIME_MATRIX_PID)
            .map_err(|e| e.to_string())?;
        
        Ok(Self { device })
    }
    
    pub fn set_brightness(&mut self, level: u8) -> Result<(), String> {
        // USB HID команда
        let packet = [0x5a, 0xba, level, 0x00];
        self.device.write(&packet).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    pub fn display_image(&mut self, data: &[u8]) -> Result<(), String> {
        // Отправка кадров (см. Starlight protocol)
        Ok(())
    }
}
```

### 6. **Flutter Bridge API** (`api.rs`)

```rust
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn set_performance_mode_bridge(mode: i32) -> Result<(), String> {
    let perf_mode = match mode {
        0 => PerformanceMode::Silent,
        1 => PerformanceMode::Balanced,
        2 => PerformanceMode::Turbo,
        _ => return Err("Invalid mode".into()),
    };
    
    platform::performance::set_performance_mode(perf_mode)
}

#[frb(sync)]
pub fn get_system_status() -> Result<SystemStatus, String> {
    Ok(SystemStatus {
        cpu_temp: monitor::sensors::get_cpu_temp()?,
        gpu_temp: monitor::sensors::get_gpu_temp()?,
        fan_speeds: monitor::sensors::get_fan_speeds()?,
        battery: platform::battery::get_battery_info()?,
    })
}

// Async stream для реального времени
pub fn monitor_sensors_stream() -> impl Stream<Item = SystemStatus> {
    // tokio stream каждую секунду
}
```

---

## 🎨 Flutter UI Components

### Dashboard:
```dart
class DashboardScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return StreamBuilder<SystemStatus>(
      stream: api.monitorSensorsStream(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) return CircularProgressIndicator();
        
        return Column(
          children: [
            TemperatureCard(
              cpuTemp: snapshot.data!.cpuTemp,
              gpuTemp: snapshot.data!.gpuTemp,
            ),
            FanSpeedCard(speeds: snapshot.data!.fanSpeeds),
            BatteryCard(info: snapshot.data!.battery),
          ],
        );
      },
    );
  }
}
```

### Performance Selector:
```dart
class PerformanceModeSelector extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return SegmentedButton<int>(
      segments: [
        ButtonSegment(value: 0, label: Text('Silent')),
        ButtonSegment(value: 1, label: Text('Balanced')),
        ButtonSegment(value: 2, label: Text('Turbo')),
      ],
      selected: {currentMode},
      onSelectionChanged: (Set<int> newSelection) {
        api.setPerformanceModeBridge(mode: newSelection.first);
      },
    );
  }
}
```

---

## 🔧 План имплементации

### Фаза 1: Базовая инфраструктура (1-2 недели)

#### Задачи:
1. **Создать проект структуру**
   - [ ] Инициализировать Flutter app
   - [ ] Создать Rust library project (cdylib)
   - [ ] Настроить flutter_rust_bridge

2. **Базовые Rust модули**
   - [ ] Sysfs reader утилиты
   - [ ] Performance mode (set/get)
   - [ ] Простой FFI API для Flutter

3. **Flutter UI scaffold**
   - [ ] Material Design 3 setup
   - [ ] Навигация (Dashboard, Performance, Fans, RGB, Settings)
   - [ ] Базовый bridge integration test

4. **Тестирование**
   - [ ] Проверка прав доступа (`sudo`/`polkit`)
   - [ ] Тест на реальном железе ASUS

### Фаза 2: Мониторинг и сенсоры (2 недели)

#### Задачи:
1. **Hwmon интеграция**
   - [ ] libmedium для чтения температур
   - [ ] Fan RPM monitoring
   - [ ] Async stream в Flutter (real-time updates)

2. **Battery management**
   - [ ] Charge limit control
   - [ ] Battery status через D-Bus/sysfs
   - [ ] UI с индикатором и слайдером

3. **GPU monitoring**
   - [ ] Nvidia NVML wrapper
   - [ ] AMD sysfs reader
   - [ ] GPU temp/load display

### Фаза 3: Управление вентиляторами (2-3 недели)

#### Задачи:
1. **Fan curve editor**
   - [ ] Rust: Чтение/запись `pwm_auto_point`
   - [ ] Flutter: Интерактивный график (fl_chart)
   - [ ] Сохранение профилей в JSON

2. **Preset fan curves**
   - [ ] Silent/Balanced/Performance пресеты
   - [ ] Auto-apply при смене режима

3. **Safety checks**
   - [ ] Минимальные RPM лимиты
   - [ ] Температурные пороги

### Фаза 4: GPU режимы и управление (1-2 недели)

#### Задачи:
1. **GPU switching**
   - [ ] Eco/Standard/Ultimate через sysfs
   - [ ] X11/Wayland session restart handling
   - [ ] UI с предупреждениями

2. **Nvidia control**
   - [ ] Разгон (core/memory clock offset)
   - [ ] Power limit
   - [ ] Fan curve (если доступно)

3. **AMD control**
   - [ ] pp_od_clk_voltage sysfs
   - [ ] Power profile modes

### Фаза 5: RGB и Anime Matrix (2-3 недели)

#### Задачи:
1. **Keyboard backlight**
   - [ ] Brightness control (sysfs LED)
   - [ ] Animation modes (если доступно через WMI)

2. **Anime Matrix**
   - [ ] USB HID protocol research (AsusCtl/Starlight)
   - [ ] Bitmap rendering на матрице
   - [ ] GIF support (image crate)
   - [ ] Clock и Audio visualizer

3. **Peripheral support**
   - [ ] ASUS mice USB HID
   - [ ] DPI/polling rate/RGB

### Фаза 6: Расширенные функции (2 недели)

#### Задачи:
1. **Display control**
   - [ ] Refresh rate switching (xrandr/Wayland)
   - [ ] Display overdrive toggle

2. **Hotkeys**
   - [ ] Global hotkey listener (X11/Wayland)
   - [ ] Fn+F5 для режимов
   - [ ] Custom bindings

3. **Automation**
   - [ ] AC/Battery profiles
   - [ ] Auto GPU switching
   - [ ] Scheduled tasks

### Фаза 7: Полировка и релиз (1-2 недели)

#### Задачи:
1. **Настройки и профили**
   - [ ] Сохранение в XDG_CONFIG_HOME
   - [ ] Import/export профилей
   - [ ] First-run wizard

2. **Permissions handling**
   - [ ] Polkit rules установка
   - [ ] Systemd service (опционально)
   - [ ] sudo/pkexec проверки

3. **Packaging**
   - [ ] AppImage
   - [ ] Flatpak
   - [ ] Debian/Ubuntu .deb
   - [ ] Arch AUR package

4. **Документация**
   - [ ] README с поддерживаемыми моделями
   - [ ] Wiki/FAQ
   - [ ] Troubleshooting guide

---

## ⚠️ Важные замечания

### Permissions:
Многие операции требуют root. Решения:
1. **Polkit** - создать правила для конкретных действий
2. **udev rules** - дать доступ к sysfs без sudo
3. **Systemd service** - daemon с правами, GUI через D-Bus

Пример udev rule:
```bash
# /etc/udev/rules.d/99-asus-control.rules
KERNEL=="hwmon*", SUBSYSTEM=="hwmon", RUN+="/bin/chmod 0666 /sys/class/hwmon/%n/pwm*"
SUBSYSTEM=="platform", DRIVER=="asus-nb-wmi", RUN+="/bin/chmod 0666 /sys/class/platform/asus-nb-wmi/*"
```

### Безопасность:
- Валидация всех входных данных (fan curves, voltages)
- Температурные лимиты
- Rollback на безопасные значения при ошибке

### Совместимость:
- Поддержка разных kernel версий
- Fallback при отсутствии hwmon/sysfs
- Детекция модели ноутбука (DMI)

---

## 📚 Дополнительные ресурсы

### Документация:
- Linux Kernel: `Documentation/ABI/testing/sysfs-platform-asus-wmi`
- AsusCtl: https://gitlab.com/asus-linux/asusctl
- NVML: https://docs.nvidia.com/deploy/nvml-api/
- Flutter Rust Bridge: https://github.com/fzyzcjy/flutter_rust_bridge

### Примеры кода:
- G-Helper: https://github.com/seerge/g-helper (Windows, C#)
- AsusCtl: https://gitlab.com/asus-linux/asusctl (Linux, Rust)
- Starlight: https://github.com/vddCore/Starlight (Anime Matrix)

### Community:
- /r/AsusLaptops
- asus-linux.org Discord
- ASUS ROG Linux community

---

## 🎯 Минимальный MVP (2-3 недели)

Для быстрого старта:

**Функции MVP**:
1. ✅ Performance mode toggle (Silent/Balanced/Turbo)
2. ✅ CPU/GPU temperature monitoring
3. ✅ Fan speed display
4. ✅ Battery charge limit
5. ✅ Basic GUI (Dashboard + Settings)

**Технологии MVP**:
- Flutter (Material 3)
- Rust (только sysfs, без GPU libs)
- flutter_rust_bridge
- Без RGB/Anime Matrix

**Далее** расширять функционал итеративно по фазам.

---

## 📊 Оценка трудозатрат

| Фаза | Срок | Сложность |
|------|------|-----------|
| Фаза 1 (Инфраструктура) | 1-2 нед | Средняя |
| Фаза 2 (Мониторинг) | 2 нед | Низкая |
| Фаза 3 (Вентиляторы) | 2-3 нед | Высокая |
| Фаза 4 (GPU) | 1-2 нед | Средняя |
| Фаза 5 (RGB) | 2-3 нед | Высокая |
| Фаза 6 (Расширенное) | 2 нед | Средняя |
| Фаза 7 (Релиз) | 1-2 нед | Низкая |
| **ИТОГО** | **11-16 недель** | - |

**Один разработчик full-time**: ~3-4 месяца до полного релиза  
**MVP**: ~3 недели

---

## 🚀 Следующие шаги

1. Создать Git репозиторий
2. Инициализировать Flutter project
3. Добавить Rust library через `cargo new --lib rust_core`
4. Настроить flutter_rust_bridge codegen
5. Реализовать первый simple API (get_performance_mode)
6. Протестировать FFI интеграцию
7. Начать разработку UI

**Good luck!** 🎮
