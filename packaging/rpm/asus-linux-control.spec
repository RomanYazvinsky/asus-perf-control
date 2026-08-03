Name:           asus-linux-control
Version:        0.0.1
Release:        1%{?dist}
Summary:        ASUS Linux Control GUI and privileged daemon

License:        MIT
URL:            https://github.com/asus-linux-control/asus-linux-control
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  systemd
%{?systemd_requires}

%global debug_package %{nil}

Requires: asus-linux-control-gui = %{version}-%{release}
Requires: asus-linux-control-daemon = %{version}-%{release}

%description
Meta package that installs the ASUS Linux Control Flutter GUI and privileged
system daemon.

%package gui
Summary:        ASUS Linux Control Flutter desktop application
Requires:       asus-linux-control-daemon = %{version}-%{release}
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description gui
Flutter desktop application for ASUS Linux Control.

%package daemon
Summary:        ASUS Linux Control privileged system daemon
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description daemon
Privileged Rust daemon and system integration files for ASUS Linux Control.

%prep
%autosetup

%build
# Release artifacts are produced by the project build pipeline.

%install
install -dm0755 %{buildroot}/usr/lib/asus-linux-control/gui
install -dm0755 %{buildroot}/usr/lib/asus-linux-control
install -dm0755 %{buildroot}/usr/bin
install -dm0755 %{buildroot}/usr/share/applications
install -dm0755 %{buildroot}/usr/share/icons/hicolor/256x256/apps
install -dm0755 %{buildroot}/usr/share/metainfo
install -dm0755 %{buildroot}/usr/lib/systemd/system
install -dm0755 %{buildroot}/usr/lib/systemd/system/asus-control-daemon.service.d
install -dm0755 %{buildroot}/usr/lib/systemd/user/graphical-session.target.wants
install -dm0755 %{buildroot}/usr/share/dbus-1/system-services
install -dm0755 %{buildroot}/usr/share/dbus-1/system.d
install -dm0755 %{buildroot}/usr/share/polkit-1/actions
install -dm0755 %{buildroot}/usr/lib/udev/rules.d

install -Dm0755 build/linux/x64/release/bundle/asus-linux-control \
  %{buildroot}/usr/lib/asus-linux-control/gui/asus-linux-control
cp -a build/linux/x64/release/bundle/data \
  %{buildroot}/usr/lib/asus-linux-control/gui/
cp -a build/linux/x64/release/bundle/lib \
  %{buildroot}/usr/lib/asus-linux-control/gui/
ln -s ../lib/asus-linux-control/gui/asus-linux-control \
  %{buildroot}/usr/bin/asus-linux-control

install -Dm0644 packaging/linux/asus-linux-control.desktop \
  %{buildroot}/usr/share/applications/asus-linux-control.desktop
install -Dm0644 packaging/linux/asus-linux-control.png \
  %{buildroot}/usr/share/icons/hicolor/256x256/apps/asus-linux-control.png
install -Dm0644 packaging/linux/asus-linux-control.metainfo.xml \
  %{buildroot}/usr/share/metainfo/asus-linux-control.metainfo.xml
install -Dm0755 rust_daemon/target/release/asus-control-hotkey-agent \
  %{buildroot}/usr/lib/asus-linux-control/asus-control-hotkey-agent
install -Dm0644 install/systemd/asus-control-hotkey-agent.service \
  %{buildroot}/usr/lib/systemd/user/asus-control-hotkey-agent.service
ln -s ../asus-control-hotkey-agent.service \
  %{buildroot}/usr/lib/systemd/user/graphical-session.target.wants/asus-control-hotkey-agent.service
install -Dm0644 install/systemd/asus-control-daemon.service.d/rog-hotkey.conf \
  %{buildroot}/usr/lib/systemd/system/asus-control-daemon.service.d/rog-hotkey.conf

install -Dm0755 rust_daemon/target/release/asus-control-daemon \
  %{buildroot}/usr/lib/asus-linux-control/asus-control-daemon
install -Dm0644 install/systemd/asus-control-daemon.service \
  %{buildroot}/usr/lib/systemd/system/asus-control-daemon.service
install -Dm0644 install/dbus/com.asuslinuxcontrol.Daemon.service \
  %{buildroot}/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service
install -Dm0644 install/dbus/com.asuslinuxcontrol.Daemon.conf \
  %{buildroot}/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf
install -Dm0644 install/dbus/com.asuslinuxcontrol.Daemon.policy \
  %{buildroot}/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy
install -Dm0644 install/99-asus-control.rules \
  %{buildroot}/usr/lib/udev/rules.d/99-asus-control.rules

%post daemon
%systemd_post asus-control-daemon.service
echo "Run 'sudo systemctl enable --now asus-control-daemon.service' to start ASUS Linux Control at boot."

%preun daemon
%systemd_preun asus-control-daemon.service

%postun daemon
%systemd_postun_with_restart asus-control-daemon.service

%post gui
%systemd_user_post asus-control-hotkey-agent.service
systemctl daemon-reload >/dev/null 2>&1 || :
systemctl try-restart asus-control-daemon.service >/dev/null 2>&1 || :

%preun gui
%systemd_user_preun asus-control-hotkey-agent.service

%postun gui
%systemd_user_postun_with_restart asus-control-hotkey-agent.service
systemctl daemon-reload >/dev/null 2>&1 || :
systemctl try-restart asus-control-daemon.service >/dev/null 2>&1 || :

%files
%doc README.md

%files gui
/usr/bin/asus-linux-control
/usr/lib/asus-linux-control/gui/
/usr/share/applications/asus-linux-control.desktop
/usr/share/icons/hicolor/256x256/apps/asus-linux-control.png
/usr/share/metainfo/asus-linux-control.metainfo.xml
/usr/lib/asus-linux-control/asus-control-hotkey-agent
/usr/lib/systemd/user/asus-control-hotkey-agent.service
/usr/lib/systemd/user/graphical-session.target.wants/asus-control-hotkey-agent.service
/usr/lib/systemd/system/asus-control-daemon.service.d/rog-hotkey.conf

%files daemon
/usr/lib/asus-linux-control/asus-control-daemon
/usr/lib/systemd/system/asus-control-daemon.service
/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service
/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf
/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy
/usr/lib/udev/rules.d/99-asus-control.rules

%changelog
* Wed Jun 24 2026 ASUS Linux Control <maintainers@example.invalid> - 0.0.1-1
- Initial semantic-versioned package release.
