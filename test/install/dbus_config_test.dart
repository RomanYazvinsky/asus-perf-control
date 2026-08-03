import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('D-Bus config allows CPU boost daemon methods', () {
    final config = File(
      'install/dbus/com.asuslinuxcontrol.Daemon.conf',
    ).readAsStringSync();

    expect(config, contains('send_member="GetCpuBoostStatus"'));
    expect(config, contains('send_member="SetCpuBoostMode"'));
  });

  test('D-Bus config allows package update daemon methods', () {
    final config = File(
      'install/dbus/com.asuslinuxcontrol.Daemon.conf',
    ).readAsStringSync();

    expect(config, contains('send_member="GetPackageBackend"'));
    expect(config, contains('send_member="CheckForUpdates"'));
    expect(config, contains('send_member="StartUpdate"'));
    expect(config, contains('send_member="GetUpdateStatus"'));
    expect(config, contains('send_member="CancelUpdate"'));
  });

  test('D-Bus config exposes ROG hotkey status and press signals', () {
    final config = File(
      'install/dbus/com.asuslinuxcontrol.Daemon.conf',
    ).readAsStringSync();

    expect(config, contains('send_member="GetRogHotkeyStatus"'));
    expect(config, contains('receive_member="RogKeyPressed"'));
  });

  test('D-Bus activation delegates daemon startup to systemd', () {
    final service = File(
      'install/dbus/com.asuslinuxcontrol.Daemon.service',
    ).readAsStringSync();

    expect(service, contains('SystemdService=asus-control-daemon.service'));
    expect(
      service,
      contains('Exec=/usr/lib/asus-linux-control/asus-control-daemon'),
    );
  });

  test('polkit policy requires admin authentication for package updates', () {
    final policy = File(
      'install/dbus/com.asuslinuxcontrol.Daemon.policy',
    ).readAsStringSync();

    expect(policy, contains('id="com.asuslinuxcontrol.manage-updates"'));
    expect(
      policy,
      contains(
        'Authentication is required to install ASUS Linux Control package updates',
      ),
    );
    expect(policy, contains('<allow_any>auth_admin</allow_any>'));
    expect(policy, contains('<allow_inactive>auth_admin</allow_inactive>'));
    expect(policy, contains('<allow_active>auth_admin</allow_active>'));
  });

  test('systemd unit starts daemon at boot for reboot restore', () {
    final unit = File(
      'install/systemd/asus-control-daemon.service',
    ).readAsStringSync();

    expect(unit, contains('Type=dbus'));
    expect(unit, contains('BusName=com.asuslinuxcontrol.Daemon'));
    expect(
      unit,
      contains('ExecStart=/usr/lib/asus-linux-control/asus-control-daemon'),
    );
    expect(unit, contains('WantedBy=multi-user.target'));
  });
}
