import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('rebuild-daemon installs D-Bus and polkit integration files', () {
    final script = File('install/rebuild-daemon.sh').readAsStringSync();

    expect(
      script,
      contains(
        'install -m 644 dbus/com.asuslinuxcontrol.Daemon.conf '
        '/etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf',
      ),
    );
    expect(
      script,
      contains(
        'install -m 644 dbus/com.asuslinuxcontrol.Daemon.policy '
        '/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy',
      ),
    );
    expect(
      script,
      contains(
        'install -m 644 systemd/asus-control-daemon.service '
        '/etc/systemd/system/asus-control-daemon.service',
      ),
    );
    expect(
      script,
      contains('systemctl enable --now asus-control-daemon.service'),
    );
    expect(script, contains('GetCpuBoostStatus'));
    expect(script, isNot(contains('GetTdpConfig')));
  });
}
