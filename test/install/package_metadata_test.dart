import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  group('native package metadata', () {
    test('RPM spec declares meta, gui, and daemon packages', () {
      final spec = File(
        'packaging/rpm/asus-linux-control.spec',
      ).readAsStringSync();

      expect(spec, contains('%package gui'));
      expect(spec, contains('%package daemon'));
      expect(spec, contains('Requires: asus-linux-control-gui'));
      expect(spec, contains('Requires: asus-linux-control-daemon'));
    });

    test('RPM daemon package owns system integration files', () {
      final spec = File(
        'packaging/rpm/asus-linux-control.spec',
      ).readAsStringSync();

      expect(
        spec,
        contains('/usr/lib/systemd/system/asus-control-daemon.service'),
      );
      expect(
        spec,
        contains(
          '/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service',
        ),
      );
      expect(
        spec,
        contains('/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf'),
      );
      expect(
        spec,
        contains(
          '/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy',
        ),
      );
      expect(spec, contains('/usr/lib/udev/rules.d/99-asus-control.rules'));
    });

    test('RPM daemon package uses systemd scriptlet macros', () {
      final spec = File(
        'packaging/rpm/asus-linux-control.spec',
      ).readAsStringSync();

      expect(spec, contains('%systemd_post asus-control-daemon.service'));
      expect(spec, contains('%systemd_preun asus-control-daemon.service'));
      expect(
        spec,
        contains('%systemd_postun_with_restart asus-control-daemon.service'),
      );
    });

    test('build targets produce native package artifacts', () {
      final makefile = File('Makefile').readAsStringSync();

      expect(makefile, contains('package-rpm:'));
      expect(makefile, contains('package-arch:'));
      expect(makefile, contains('package-all:'));
      expect(makefile, contains('rpmbuild -ba'));
      expect(makefile, contains('makepkg'));
    });

    test('Linux release binary name matches package metadata', () {
      final cmake = File('linux/CMakeLists.txt').readAsStringSync();
      final spec = File(
        'packaging/rpm/asus-linux-control.spec',
      ).readAsStringSync();
      final pkgbuild = File('packaging/arch/PKGBUILD').readAsStringSync();

      expect(cmake, contains('set(BINARY_NAME "asus-linux-control")'));
      expect(spec, contains('build/linux/x64/release/bundle/asus-linux-control'));
      expect(
        pkgbuild,
        contains('build/linux/x64/release/bundle/asus-linux-control'),
      );
    });

    test('desktop metadata assets required by packages exist', () {
      expect(File('packaging/linux/asus-linux-control.desktop').existsSync(), isTrue);
      expect(
        File('packaging/linux/asus-linux-control.metainfo.xml').existsSync(),
        isTrue,
      );
      expect(File('packaging/linux/asus-linux-control.png').existsSync(), isTrue);
    });

    test('Arch PKGBUILD declares split packages and dependencies', () {
      final pkgbuild = File('packaging/arch/PKGBUILD').readAsStringSync();

      expect(pkgbuild, contains('pkgbase=asus-linux-control'));
      expect(
        pkgbuild,
        contains(
          "pkgname=('asus-linux-control' 'asus-linux-control-gui' 'asus-linux-control-daemon')",
        ),
      );
      expect(pkgbuild, contains('package_asus-linux-control()'));
      expect(pkgbuild, contains('package_asus-linux-control-gui()'));
      expect(pkgbuild, contains('package_asus-linux-control-daemon()'));
      expect(
        pkgbuild,
        contains(
          "depends=('asus-linux-control-gui' 'asus-linux-control-daemon')",
        ),
      );
      expect(pkgbuild, contains('install=asus-linux-control-daemon.install'));
    });

    test(
      'pacman daemon install hook reloads systemd without enabling service',
      () {
        final installScript = File(
          'packaging/arch/asus-linux-control-daemon.install',
        ).readAsStringSync();

        expect(installScript, contains('systemctl daemon-reload'));
        expect(
          installScript,
          isNot(matches(RegExp(r'^\s*systemctl\s+enable\b', multiLine: true))),
        );
      },
    );
  });
}
