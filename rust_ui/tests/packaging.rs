//! Repository packaging and integration-file checks (ported from the former
//! `test/install/*_test.dart`).

use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(path)
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo(path)).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn section<'a>(text: &'a str, start: &str, end: Option<&str>) -> &'a str {
    let from = text.find(start).unwrap_or_else(|| panic!("{start} missing"));
    let to = end.map_or(text.len(), |end| text.find(end).unwrap_or_else(|| panic!("{end} missing")));
    &text[from..to]
}

fn has_line_starting_with(text: &str, prefix: &str) -> bool {
    text.lines().any(|line| line.trim_start().starts_with(prefix))
}

const SPEC: &str = "packaging/rpm/asus-linux-control.spec";
const PKGBUILD: &str = "packaging/arch/PKGBUILD";
const GUI_HOOK: &str = "packaging/arch/asus-linux-control-gui.install";
const BINARY: &str = "rust_ui/target/release/asus-linux-control";

#[test]
fn rpm_spec_declares_meta_gui_and_daemon_packages() {
    let spec = read(SPEC);
    for needle in [
        "%package gui",
        "%package daemon",
        "Requires: asus-linux-control-gui",
        "Requires: asus-linux-control-daemon",
    ] {
        assert!(spec.contains(needle), "{needle}");
    }
}

#[test]
fn rpm_daemon_package_owns_system_integration_files() {
    let spec = read(SPEC);
    for needle in [
        "/usr/lib/systemd/system/asus-control-daemon.service",
        "/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service",
        "/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf",
        "/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy",
        "/usr/lib/udev/rules.d/99-asus-control.rules",
        "%systemd_post asus-control-daemon.service",
        "%systemd_preun asus-control-daemon.service",
        "%systemd_postun_with_restart asus-control-daemon.service",
    ] {
        assert!(spec.contains(needle), "{needle}");
    }
}

#[test]
fn build_targets_produce_native_package_artifacts() {
    let makefile = read("Makefile");
    for needle in [
        "package-rpm:",
        "package-arch:",
        "package-all:",
        "rpmbuild -ba",
        "makepkg",
        "build-ui:",
        "packaging/arch/asus-linux-control-gui.install",
    ] {
        assert!(makefile.contains(needle), "{needle}");
    }
    assert!(!makefile.contains("flutter"), "Makefile still calls flutter");
}

#[test]
fn packages_install_the_rust_ui_binary_under_the_desktop_name() {
    let cargo = read("rust_ui/Cargo.toml");
    assert!(cargo.contains("name = \"asus-linux-control\""));
    for path in [SPEC, PKGBUILD] {
        let text = read(path);
        assert!(text.contains(BINARY), "{path} must install {BINARY}");
        assert!(!text.contains("build/linux/x64/release/bundle"), "{path} still references the Flutter bundle");
    }
    let desktop = read("packaging/linux/asus-linux-control.desktop");
    assert!(desktop.contains("Exec=asus-linux-control"));
    assert!(desktop.contains("StartupWMClass=asus-linux-control"));
    assert!(read("rust_ui/src/main.rs").contains(".with_app_id(\"asus-linux-control\")"));
}

#[test]
fn desktop_metadata_assets_exist() {
    for path in [
        "packaging/linux/asus-linux-control.desktop",
        "packaging/linux/asus-linux-control.metainfo.xml",
        "packaging/linux/asus-linux-control.png",
    ] {
        assert!(repo(path).exists(), "{path}");
    }
}

#[test]
fn arch_pkgbuild_declares_split_packages() {
    let pkgbuild = read(PKGBUILD);
    for needle in [
        "pkgbase=asus-linux-control",
        "pkgname=('asus-linux-control' 'asus-linux-control-gui' 'asus-linux-control-daemon')",
        "package_asus-linux-control()",
        "package_asus-linux-control-gui()",
        "package_asus-linux-control-daemon()",
        "depends=('asus-linux-control-gui' 'asus-linux-control-daemon')",
        "install=asus-linux-control-daemon.install",
        "install=asus-linux-control-gui.install",
    ] {
        assert!(pkgbuild.contains(needle), "{needle}");
    }
}

#[test]
fn pacman_daemon_hook_reloads_systemd_without_enabling() {
    let hook = read("packaging/arch/asus-linux-control-daemon.install");
    assert!(hook.contains("systemctl daemon-reload"));
    assert!(!hook
        .lines()
        .any(|l| l.trim_start().starts_with("systemctl") && l.split_whitespace().nth(1) == Some("enable")));
}

#[test]
fn hotkey_agent_user_service_has_graphical_session_lifecycle() {
    let service = read("install/systemd/asus-control-hotkey-agent.service");
    assert!(service.contains("ExecStart=/usr/lib/asus-linux-control/asus-control-hotkey-agent"));
    assert!(service.contains("Restart=on-failure"));
    assert!(service.contains("WantedBy=graphical-session.target"));
    assert!(read("rust_daemon/src/hotkey_agent.rs").contains("\"asus-linux-control\""));
}

#[test]
fn root_installer_installs_but_does_not_launch_user_agent() {
    let installer = read("install/install.sh");
    assert!(installer.contains("target/release/asus-control-hotkey-agent"));
    assert!(installer.contains("/usr/lib/systemd/user"));
    assert!(installer.contains("graphical-session.target.wants/asus-control-hotkey-agent.service"));
    assert!(!installer.contains("systemctl --user"));
}

#[test]
fn root_installer_resolves_paths_and_delegates_sudo_builds() {
    let installer = read("install/install.sh");
    for needle in [
        "SCRIPT_DIR=",
        "PROJECT_ROOT=",
        "runuser -u \"$SUDO_USER\"",
        "HOME=\"$BUILD_HOME\"",
        "cargo build --release --bins",
        "Build the release binaries as your regular user",
        "local BUILD_HOME BUILD_PATH CARGO_BIN",
        "mktemp",
        "install -m 0755",
        "mv -f",
        "install -d -m 0755",
        "install -m 0644",
    ] {
        assert!(installer.contains(needle), "{needle}");
    }
    assert!(!installer.contains("DAEMON_SOURCE=\"../rust_daemon\""));
    assert!(!has_line_starting_with(&installer, "cargo build --release --bins"));
    let guard = "if [ -z \"${SUDO_USER:-}\" ] || [ \"$SUDO_USER\" = \"root\" ]; then";
    let after = &installer[installer.find(guard).expect("sudo guard")..];
    let body: Vec<&str> = after.lines().skip(1).take(3).map(str::trim).collect();
    assert_eq!(body, ["print_prebuild_help", "return 1", "fi"]);
}

#[test]
fn gui_packages_own_hotkey_agent_and_rog_drop_in() {
    let pkgbuild = read(PKGBUILD);
    let spec = read(SPEC);
    let arch_gui = section(&pkgbuild, "package_asus-linux-control-gui()", Some("package_asus-linux-control-daemon()"));
    let arch_daemon = section(&pkgbuild, "package_asus-linux-control-daemon()", None);
    let rpm_gui = section(&spec, "%files gui", Some("%files daemon"));
    let rpm_daemon = section(&spec, "%files daemon", None);

    for owned in ["asus-control-hotkey-agent", "rog-hotkey.conf"] {
        assert!(arch_gui.contains(owned) && rpm_gui.contains(owned), "{owned} must be in the GUI package");
        assert!(!arch_daemon.contains(owned) && !rpm_daemon.contains(owned), "{owned} must not be in the daemon package");
    }
    for text in [&pkgbuild, &spec] {
        assert!(text.contains("graphical-session.target.wants"));
        assert!(text.contains("ln -s ../asus-control-hotkey-agent.service"));
    }
    assert!(spec.contains("/usr/lib/systemd/user/asus-control-hotkey-agent.service"));
    let drop_in = read("install/systemd/asus-control-daemon.service.d/rog-hotkey.conf");
    assert!(drop_in.contains("Environment=ASUS_ROG_HOTKEY_ENABLED=1"));
}

#[test]
fn rpm_gui_package_runs_systemd_user_scriptlets() {
    let spec = read(SPEC);
    for needle in [
        "Requires(post): systemd",
        "Requires(preun): systemd",
        "Requires(postun): systemd",
        "%systemd_user_post asus-control-hotkey-agent.service",
        "%systemd_user_preun asus-control-hotkey-agent.service",
        "%systemd_user_postun_with_restart asus-control-hotkey-agent.service",
        "systemctl daemon-reload",
        "systemctl try-restart asus-control-daemon.service",
    ] {
        assert!(spec.contains(needle), "{needle}");
    }
    assert!(!spec.contains("systemctl start asus-control-daemon"));
}

#[test]
fn arch_gui_hook_refreshes_user_managers_safely() {
    let hook = read(GUI_HOOK);
    for needle in [
        "timeout 5s runuser -u",
        "XDG_RUNTIME_DIR=",
        "graphical-session.target",
        "systemctl --user daemon-reload",
        "systemctl --user restart asus-control-hotkey-agent.service",
        "systemctl --user stop asus-control-hotkey-agent.service",
        "pre_remove()",
        "post_remove()",
        "|| true",
        "systemctl daemon-reload",
        "systemctl try-restart asus-control-daemon.service",
    ] {
        assert!(hook.contains(needle), "{needle}");
    }
    assert!(!has_line_starting_with(&hook, "systemctl --user"));
    assert!(!hook.contains("gtk-launch"));
    assert!(!hook.contains("systemctl start asus-control-daemon"));
}

#[test]
fn direct_installer_installs_rog_opt_in_before_daemon_start() {
    let installer = read("install/install.sh");
    let drop_in = installer.find("rog-hotkey.conf").expect("rog-hotkey.conf");
    let start = installer
        .find("systemctl restart asus-control-daemon.service")
        .expect("daemon start");
    assert!(drop_in < start);
}

#[test]
fn dbus_config_allows_daemon_methods_and_signals() {
    let config = read("install/dbus/com.asuslinuxcontrol.Daemon.conf");
    for member in [
        "send_member=\"GetCpuBoostStatus\"",
        "send_member=\"SetCpuBoostMode\"",
        "send_member=\"GetPackageBackend\"",
        "send_member=\"CheckForUpdates\"",
        "send_member=\"StartUpdate\"",
        "send_member=\"GetUpdateStatus\"",
        "send_member=\"CancelUpdate\"",
        "send_member=\"GetRogHotkeyStatus\"",
        "receive_member=\"RogKeyPressed\"",
    ] {
        assert!(config.contains(member), "{member}");
    }
}

#[test]
fn dbus_activation_and_systemd_unit() {
    let service = read("install/dbus/com.asuslinuxcontrol.Daemon.service");
    assert!(service.contains("SystemdService=asus-control-daemon.service"));
    assert!(service.contains("Exec=/usr/lib/asus-linux-control/asus-control-daemon"));

    let unit = read("install/systemd/asus-control-daemon.service");
    for needle in [
        "Type=dbus",
        "BusName=com.asuslinuxcontrol.Daemon",
        "ExecStart=/usr/lib/asus-linux-control/asus-control-daemon",
        "WantedBy=multi-user.target",
    ] {
        assert!(unit.contains(needle), "{needle}");
    }
}

#[test]
fn polkit_policy_requires_admin_for_package_updates() {
    let policy = read("install/dbus/com.asuslinuxcontrol.Daemon.policy");
    for needle in [
        "id=\"com.asuslinuxcontrol.manage-updates\"",
        "Authentication is required to install ASUS Linux Control package updates",
        "<allow_any>auth_admin</allow_any>",
        "<allow_inactive>auth_admin</allow_inactive>",
        "<allow_active>auth_admin</allow_active>",
    ] {
        assert!(policy.contains(needle), "{needle}");
    }
}

#[test]
fn rebuild_daemon_installs_dbus_and_polkit_files() {
    let script = read("install/rebuild-daemon.sh");
    for needle in [
        "install -m 644 dbus/com.asuslinuxcontrol.Daemon.conf /etc/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf",
        "install -m 644 dbus/com.asuslinuxcontrol.Daemon.policy /usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy",
        "install -m 644 systemd/asus-control-daemon.service /etc/systemd/system/asus-control-daemon.service",
        "systemctl enable asus-control-daemon.service",
        "systemctl restart asus-control-daemon.service",
        "GetCpuBoostStatus",
    ] {
        assert!(script.contains(needle), "{needle}");
    }
    assert!(!script.contains("GetTdpConfig"));
}

const SELINUX_MODULE: &str = "install/selinux/asus_control.cil";
const DAEMON_EXEC_CONTEXT: &str = "(filecon \"/usr/lib/asus-linux-control/asus-control-daemon\" file (system_u object_r asus_control_exec_t";

#[test]
fn selinux_module_gives_daemon_its_own_domain_with_rog_device_access() {
    let module = read(SELINUX_MODULE);
    for needle in [
        DAEMON_EXEC_CONTEXT,
        "(filecon \"/var/lib/asus-linux-control(/.*)?\" any (system_u object_r asus_control_var_lib_t",
        "(typetransition init_t asus_control_exec_t process asus_control_t)",
        "(allow asus_control_t usb_device_t (chr_file (getattr open read write ioctl lock)))",
        "(allow asus_control_t event_device_t (chr_file (getattr open read write ioctl lock)))",
        "(typetransition asus_control_t rpm_exec_t process rpm_t)",
    ] {
        assert!(module.contains(needle), "{needle}");
    }
    // The generic service domain must not be widened: only asus_control_t gets device access.
    assert!(!module.contains("(allow init_t usb_device_t"));
    assert!(!module.contains("(allow init_t event_device_t"));

    let unit = read("install/systemd/asus-control-daemon.service");
    assert!(unit.contains("ExecStart=/usr/lib/asus-linux-control/asus-control-daemon"));
}

#[test]
fn rpm_daemon_package_loads_and_removes_selinux_module() {
    let spec = read(SPEC);
    let daemon_files = section(&spec, "%files daemon", None);
    assert!(daemon_files.contains("%{_datadir}/selinux/packages/%{selinuxtype}/asus_control.cil"));
    for needle in [
        "install -Dm0644 install/selinux/asus_control.cil",
        "%selinux_relabel_pre -s %{selinuxtype}",
        "%selinux_modules_install -s %{selinuxtype} %{_datadir}/selinux/packages/%{selinuxtype}/asus_control.cil",
        "%selinux_modules_uninstall -s %{selinuxtype} asus_control",
        "%selinux_relabel_post -s %{selinuxtype}",
    ] {
        assert!(spec.contains(needle), "{needle}");
    }
    // Relabel happens after the files are on disk, then a running daemon moves into its domain.
    let posttrans = section(&spec, "%posttrans daemon", Some("%preun daemon"));
    let relabel = posttrans.find("%selinux_relabel_post").expect("relabel");
    let restart = posttrans.find("try-restart asus-control-daemon.service").expect("restart");
    assert!(relabel < restart);
}

#[test]
fn direct_installers_load_selinux_module_and_restore_labels_before_start() {
    // The module is loaded before the binary appears and the binary is labelled for its final
    // path while still temporary: D-Bus activation can start it the moment it is moved in.
    // A restart at the end replaces any daemon activated during the install.
    // (script, marker of the module load, marker of the daemon binary install)
    for (path, module_step, binary_step) in [
        ("install/install.sh", "    install_selinux_policy\n", "install_binary_atomically \"$DAEMON_BINARY\""),
        ("install/rebuild-daemon.sh", "semodule -i selinux/asus_control.cil", "asus-control-daemon.new"),
    ] {
        let script = read(path);
        assert!(script.contains("semodule -i"), "{path}: semodule");
        assert!(script.contains("selinux/asus_control.cil"), "{path}: module path");
        assert!(script.contains("restorecon -RF"), "{path}: restorecon");
        assert!(script.contains("matchpathcon -n"), "{path}: label lookup");
        assert!(script.contains("chcon "), "{path}: label before move");
        let module = script.find(module_step).unwrap_or_else(|| panic!("{path}: SELinux step"));
        let binary = script.find(binary_step).unwrap_or_else(|| panic!("{path}: binary install"));
        let start = script
            .find("systemctl restart asus-control-daemon.service")
            .unwrap_or_else(|| panic!("{path}: daemon start"));
        assert!(module < binary && binary < start, "{path}: order");
        assert!(!script.contains("systemctl enable --now asus-control-daemon"), "{path}: restart, not enable --now");
    }
}

#[test]
fn root_installer_installs_desktop_app_launched_by_hotkey_agent() {
    // The agent runs `gtk-launch asus-linux-control`, which needs the binary and desktop entry.
    let installer = read("install/install.sh");
    for needle in [
        "build_release_bins \"$UI_SOURCE/Cargo.toml\"",
        "install_binary_atomically \"$UI_BINARY\" /usr/bin/asus-linux-control",
        "/usr/share/applications/asus-linux-control.desktop",
        "/usr/share/icons/hicolor/256x256/apps/asus-linux-control.png",
        "/usr/share/metainfo/asus-linux-control.metainfo.xml",
    ] {
        assert!(installer.contains(needle), "{needle}");
    }
    assert!(read("rust_daemon/src/hotkey_agent.rs").contains("\"gtk-launch\""));
    assert!(read("packaging/linux/asus-linux-control.desktop").contains("Exec=asus-linux-control"));

    let makefile = read("Makefile");
    assert!(makefile.contains("install: build-daemon build-ui"));
}
