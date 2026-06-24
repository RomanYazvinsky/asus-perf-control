#![allow(dead_code)]

#[path = "../src/package_updates/mod.rs"]
mod package_updates;

use package_updates::backend::{detect_backend_from_inputs, BackendDetectionInput};
use package_updates::commands::{command_for_backend, PackageCommandKind};
use package_updates::job::UpdateJobManager;
use package_updates::parser::parse_check_output;
use package_updates::status::{PackageBackend, UpdateErrorCode, UpdateState, UpdateStatus};
use std::collections::{HashMap, HashSet};

fn os_release(values: &[(&str, &str)]) -> HashMap<String, String> {
    values
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn executables(names: &[&str]) -> HashSet<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn detect(
    os_release: HashMap<String, String>,
    executable_names: &[&str],
    pacman_db_writable: bool,
) -> PackageBackend {
    detect_backend_from_inputs(BackendDetectionInput {
        os_release: &os_release,
        executable_names: &executables(executable_names),
        pacman_db_writable,
    })
}

#[test]
fn bazzite_is_selected_before_generic_fedora() {
    let backend = detect(
        os_release(&[
            ("ID", "bazzite"),
            ("ID_LIKE", "fedora"),
            ("VARIANT_ID", "bazzite"),
        ]),
        &["dnf5", "rpm-ostree"],
        false,
    );

    assert_eq!(backend, PackageBackend::BazziteRpmOstree);
}

#[test]
fn steamos_requires_pacman_and_writable_package_database() {
    let os = os_release(&[("ID", "steamos"), ("ID_LIKE", "arch")]);

    assert_eq!(
        detect(os.clone(), &["pacman"], true),
        PackageBackend::SteamOsPacman
    );
    assert_eq!(detect(os.clone(), &[], true), PackageBackend::Unsupported);
    assert_eq!(detect(os, &["pacman"], false), PackageBackend::Unsupported);
}

#[test]
fn fedora_prefers_dnf5_before_dnf() {
    let os = os_release(&[("ID", "fedora")]);

    assert_eq!(
        detect(os.clone(), &["dnf", "dnf5"], false),
        PackageBackend::FedoraDnf5
    );
    assert_eq!(detect(os, &["dnf"], false), PackageBackend::FedoraDnf);
}

#[test]
fn arch_returns_pacman_backend() {
    assert_eq!(
        detect(os_release(&[("ID", "arch")]), &["pacman"], true),
        PackageBackend::ArchPacman
    );
}

#[test]
fn unknown_os_is_unsupported() {
    assert_eq!(
        detect(os_release(&[("ID", "debian")]), &["apt"], false),
        PackageBackend::Unsupported
    );
}

#[test]
fn backend_commands_are_executable_plus_fixed_args() {
    let command = command_for_backend(PackageBackend::FedoraDnf5, PackageCommandKind::Check)
        .expect("dnf5 check command should exist");

    assert_eq!(command.executable, "dnf5");
    assert_eq!(command.args, ["check-upgrade", "asus-linux-control"]);

    let command = command_for_backend(PackageBackend::ArchPacman, PackageCommandKind::Update)
        .expect("pacman update command should exist");

    assert_eq!(command.executable, "pacman");
    assert_eq!(
        command.args,
        ["-Syu", "--needed", "--noconfirm", "asus-linux-control"]
    );
}

#[test]
fn unsupported_backend_command_lookup_returns_error() {
    let error = command_for_backend(PackageBackend::Unsupported, PackageCommandKind::Check)
        .expect_err("unsupported backend must not fall back to a shell command");

    assert!(error.to_string().contains("unsupported"));
}

#[test]
fn serialized_status_uses_versioned_schema() {
    let status = UpdateStatus {
        backend: PackageBackend::FedoraDnf5,
        state: UpdateState::Available,
        installed_version: Some("0.0.1".to_string()),
        available_version: Some("0.0.2".to_string()),
        reboot_required: Some(false),
        summary: "Update available".to_string(),
        error_code: None,
        log_tail: vec!["checked repository metadata".to_string()],
    };

    let json = status.to_json().expect("status should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["backend"], "fedora-dnf5");
    assert_eq!(value["state"], "available");
    assert_eq!(value["installedVersion"], "0.0.1");
    assert_eq!(value["availableVersion"], "0.0.2");
    assert_eq!(value["rebootRequired"], false);
    assert_eq!(value["summary"], "Update available");
    assert!(value["errorCode"].is_null());
    assert_eq!(value["logTail"][0], "checked repository metadata");
}

#[test]
fn dnf5_json_with_matching_package_maps_to_available() {
    let status = parse_check_output(
        PackageBackend::FedoraDnf5,
        r#"{"packages":[{"name":"asus-linux-control","evr":"0.0.2-1.fc40"}]}"#,
        "",
        100,
    );

    assert_eq!(status.state, UpdateState::Available);
    assert_eq!(status.available_version.as_deref(), Some("0.0.2-1.fc40"));
}

#[test]
fn dnf4_check_output_with_matching_package_maps_to_available() {
    let status = parse_check_output(
        PackageBackend::FedoraDnf,
        "asus-linux-control.x86_64 0.0.2-1.fc40 updates\n",
        "",
        100,
    );

    assert_eq!(status.state, UpdateState::Available);
    assert_eq!(status.available_version.as_deref(), Some("0.0.2-1.fc40"));
}

#[test]
fn dnf_exit_code_zero_maps_to_up_to_date() {
    let status = parse_check_output(PackageBackend::FedoraDnf5, "", "", 0);

    assert_eq!(status.state, UpdateState::UpToDate);
}

#[test]
fn pacman_update_line_maps_to_available() {
    let status = parse_check_output(
        PackageBackend::ArchPacman,
        "asus-linux-control 0.0.1-1 -> 0.0.2-1\n",
        "",
        0,
    );

    assert_eq!(status.state, UpdateState::Available);
    assert_eq!(status.installed_version.as_deref(), Some("0.0.1-1"));
    assert_eq!(status.available_version.as_deref(), Some("0.0.2-1"));
}

#[test]
fn rpm_ostree_no_changes_maps_to_up_to_date() {
    let status = parse_check_output(
        PackageBackend::BazziteRpmOstree,
        "No upgrade available.\n",
        "",
        0,
    );

    assert_eq!(status.state, UpdateState::UpToDate);
}

#[test]
fn parser_maps_specific_package_manager_errors() {
    let cases = [
        (
            "Could not get lock /var/lib/rpm/.rpm.lock",
            "package_database_locked",
        ),
        ("repomd.xml: Cannot download", "missing_repository"),
        ("GPG signature verification failed", "signature_error"),
        ("Could not resolve host: mirrors.example", "network_error"),
    ];

    for (stderr, expected) in cases {
        let status = parse_check_output(PackageBackend::FedoraDnf, "", stderr, 1);
        let json: serde_json::Value =
            serde_json::from_str(&status.to_json().expect("status should serialize")).unwrap();

        assert_eq!(status.state, UpdateState::Failed);
        assert_eq!(json["errorCode"], expected);
    }
}

#[test]
fn start_update_creates_one_update_id_when_idle() {
    let manager = UpdateJobManager::new();

    let update_id = manager.start_update_for_test(PackageBackend::FedoraDnf5);
    let status = manager
        .get_status(&update_id)
        .expect("started update should have status");

    assert!(!update_id.is_empty());
    assert_eq!(status.state, UpdateState::Running);
}

#[test]
fn second_start_while_running_returns_existing_id() {
    let manager = UpdateJobManager::new();

    let first = manager.start_update_for_test(PackageBackend::FedoraDnf5);
    let second = manager.start_update_for_test(PackageBackend::FedoraDnf5);

    assert_eq!(first, second);
}

#[test]
fn successful_update_transitions_to_succeeded() {
    let manager = UpdateJobManager::new();
    let update_id = manager.start_update_for_test(PackageBackend::FedoraDnf5);

    manager.complete_for_test(&update_id, 0, "");
    let status = manager.get_status(&update_id).unwrap();

    assert_eq!(status.state, UpdateState::Succeeded);
    assert_eq!(status.error_code, None);
}

#[test]
fn failed_update_maps_to_specific_error_when_possible() {
    let manager = UpdateJobManager::new();
    let update_id = manager.start_update_for_test(PackageBackend::ArchPacman);

    manager.complete_for_test(&update_id, 1, "database is locked");
    let status = manager.get_status(&update_id).unwrap();

    assert_eq!(status.state, UpdateState::Failed);
    assert_eq!(
        status.error_code,
        Some(UpdateErrorCode::PackageDatabaseLocked)
    );
}

#[test]
fn failed_update_defaults_to_update_failed() {
    let manager = UpdateJobManager::new();
    let update_id = manager.start_update_for_test(PackageBackend::FedoraDnf);

    manager.complete_for_test(&update_id, 1, "unknown failure");
    let status = manager.get_status(&update_id).unwrap();

    assert_eq!(status.state, UpdateState::Failed);
    assert_eq!(status.error_code, Some(UpdateErrorCode::UpdateFailed));
}

#[test]
fn bazzite_successful_update_requires_reboot() {
    let manager = UpdateJobManager::new();
    let update_id = manager.start_update_for_test(PackageBackend::BazziteRpmOstree);

    manager.complete_for_test(&update_id, 0, "");
    let status = manager.get_status(&update_id).unwrap();

    assert_eq!(status.state, UpdateState::Succeeded);
    assert_eq!(status.reboot_required, Some(true));
}

#[test]
fn cancel_marks_running_job_failed_without_starting_another_job() {
    let manager = UpdateJobManager::new();
    let update_id = manager.start_update_for_test(PackageBackend::FedoraDnf5);

    assert!(manager.cancel_update(&update_id));

    let status = manager.get_status(&update_id).unwrap();
    assert_eq!(status.state, UpdateState::Failed);
    assert_eq!(status.summary, "Package update canceled");
}
