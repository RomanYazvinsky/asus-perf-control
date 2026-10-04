---
description: Пересборка и переустановка D-Bus daemon после изменений в rust_daemon/
---

# Обновление D-Bus Daemon

Используй этот workflow каждый раз после изменений в `rust_daemon/src/`.

## Шаг 1: Собрать daemon

```bash
cargo build --release
```

Выполнять из: `rust_daemon/`

Результат: `rust_daemon/target/release/asus-control-daemon`

## Шаг 2: Установить daemon

```bash
sudo ./install.sh
```

Выполнять из: `install/`

Скрипт скопирует бинарник в `/usr/lib/asus-linux-control/` и перезагрузит D-Bus.

## Шаг 3: Убедиться что старый процесс не висит

```bash
pkill -f asus-control-daemon || true
```

Daemon запускается через D-Bus activation — `systemctl start` не нужен, поднимется автоматически при первом вызове.

## Шаг 4: Проверить через busctl

```bash
busctl call com.asuslinuxcontrol.Daemon \
  /com/asuslinuxcontrol/Daemon \
  com.asuslinuxcontrol.Daemon GetTdpConfig
```

Ожидаемый вывод: `biii true <sustained_w> <boost_w> <temp_c>` (значения в ваттах)

## Быстрый способ (шаги 1–2 вместе)

```bash
make install
```

Выполнять из корня проекта. Делает `cargo build --release` + `sudo ./install.sh` автоматически.

## Диагностика при проблемах

```bash
journalctl -u asus-control-daemon -n 50
```
