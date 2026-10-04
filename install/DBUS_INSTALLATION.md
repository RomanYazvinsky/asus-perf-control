# D-Bus + Polkit Installation Guide

## Архитектура

```
asus-linux-control (rust_ui, egui)
    ├─ Performance mode, TDP, battery limit, fan curves ── D-Bus ──► asusd (xyz.ljones.Asusd)
    │                                         │ персистентно, знает min/max с железа
    │
    ├─ Temp limit, CPU boost, fan engine, updates ── D-Bus ──► asus-control-daemon (root)
    │                                         │ Polkit authorization
    │                                         ▼ sysfs write
    └─ Fan RPM, temperatures ── rust_core ──► sysfs (read-only)
```

## Что установится

1. **asus-control-daemon** — Rust daemon (работает от root)
   - Скомпилируется автоматически при установке
   - Нативный бинарник (~1.5 MB, 2-5 MB памяти)
   - Отвечает за: **battery charge limit** (`charge_control_end_threshold`)
   - TDP управляется через `asusd` — не через этот daemon
   - Проверяет права через **polkit** для всех операций записи
   - Операции чтения не требуют авторизации

2. **Polkit policy** (`com.asuslinuxcontrol.Daemon.policy`)
   - `allow_active=yes` — активный пользователь за экраном может писать без пароля
   - `allow_inactive=auth_admin` — удалённые пользователи должны ввести admin пароль

3. **D-Bus configuration** (`com.asuslinuxcontrol.Daemon.conf`)
   - Только root может владеть сервисом
   - Пользователи могут вызывать только определённые методы
   - Polkit проверяет права на запись

## Установка

```bash
cd install
sudo ./install.sh
```

Это установит:
- `/usr/lib/asus-linux-control/asus-control-daemon`
- `/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service`
- `/etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf`
- `/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy`

После установки:
```bash
# Перезагрузить D-Bus
sudo systemctl reload dbus

# Daemon запустится автоматически при первом вызове
```

## Ручное тестирование

### 1. Проверить, что daemon запускается

```bash
# Вызвать метод для запуска daemon
dbus-send --system --print-reply \
  --dest=com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon.GetPerformanceMode
```

### 2. Проверить polkit авторизацию

```bash
# Попробовать установить режим (должно запросить пароль или разрешить)
dbus-send --system --print-reply \
  --dest=com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon.SetPerformanceMode \
  int32:2
```

**Ожидаемое поведение:**
- Если вы активный пользователь (за экраном) — операция выполнится без пароля
- Если удалённый — запросит admin пароль через polkit GUI
- Если у вас нет прав — получите ошибку `NotAuthorized`

### 3. Проверить логи daemon

```bash
# Смотреть логи D-Bus daemon
journalctl -f | grep asus-control-daemon
```

## Безопасность

### ✅ Что защищено

- **Polkit авторизация** — все операции записи требуют прав через polkit
- **D-Bus ограничения** — пользователи могут вызывать только разрешённые методы
- **Валидация входных данных** — mode проверяется на диапазон 0-2
- **Root isolation** — только root может владеть D-Bus сервисом
- **Чтение без авторизации** — Get* методы не требуют polkit (read-only sysfs)

### Polkit Policy

```xml
<allow_active>yes</allow_active>          <!-- Активный пользователь: без пароля -->
<allow_inactive>auth_admin</allow_inactive> <!-- Удалённый: требует admin пароль -->
<allow_any>auth_admin</allow_any>          <!-- Остальные: требует admin пароль -->
```

## Отладка

### Daemon не запускается

```bash
# Проверить установку
ls -la /usr/lib/asus-linux-control/asus-control-daemon
ls -la /usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service

# Запустить вручную для диагностики
sudo /usr/lib/asus-linux-control/asus-control-daemon
```

### Ошибка авторизации

```bash
# Проверить polkit policy
ls -la /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy

# Проверить, что polkit работает
pkcheck --action-id com.asuslinuxcontrol.set-performance-mode \
  --process $$ -u
```

### D-Bus ошибки

```bash
# Проверить D-Bus config
ls -la /etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf

# Проверить синтаксис
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep asus
```

## Обновление daemon после изменений кода

Используй этот порядок каждый раз, когда менялся код в `rust_daemon/src/`.

### 1. Пересобрать бинарник

```bash
cd rust_daemon
cargo build --release
```

### 2. Переустановить

```bash
cd install
sudo ./install.sh
```

Скрипт скопирует новый бинарник в `/usr/lib/asus-linux-control/` и перезагрузит D-Bus.

### 3. Убедиться что старый процесс не висит

```bash
pkill -f asus-control-daemon || true
```

Daemon запускается автоматически через D-Bus activation при первом обращении — отдельный `systemctl start` не нужен.

### 4. Проверить что новый daemon работает

```bash
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetTdpConfig
```

Ожидается: `biii true <sustained_w> <boost_w> <temp_c>` (значения в ваттах и °C, не в милливаттах).

### Быстрый способ (шаги 1–2 вместе)

```bash
make install
```

Выполнять из корня проекта.

---

## Удаление

```bash
sudo rm /usr/lib/asus-linux-control/asus-control-daemon
sudo rm /usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service
sudo rm /etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf
sudo rm /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy
sudo rmdir /usr/lib/asus-linux-control
sudo systemctl reload dbus
```
