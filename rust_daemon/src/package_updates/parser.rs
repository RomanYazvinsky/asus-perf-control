use super::status::{PackageBackend, UpdateErrorCode, UpdateState, UpdateStatus};

const PACKAGE_NAME: &str = "asus-linux-control";

pub fn parse_check_output(
    backend: PackageBackend,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> UpdateStatus {
    if let Some(error_code) = classify_error(stderr) {
        return failed_status(backend, error_code, stderr);
    }

    match backend {
        PackageBackend::FedoraDnf5 => parse_dnf5(stdout, exit_code),
        PackageBackend::FedoraDnf => parse_dnf4(stdout, exit_code),
        PackageBackend::BazziteRpmOstree => parse_rpm_ostree(stdout, stderr, exit_code),
        PackageBackend::ArchPacman | PackageBackend::SteamOsPacman => {
            parse_pacman(backend, stdout, stderr, exit_code)
        }
        PackageBackend::Unsupported => UpdateStatus::unsupported("Package updates are unsupported"),
    }
}

pub fn classify_error(output: &str) -> Option<UpdateErrorCode> {
    let lower = output.to_lowercase();
    if lower.contains("lock") || lower.contains("database is locked") {
        return Some(UpdateErrorCode::PackageDatabaseLocked);
    }
    if lower.contains("repomd.xml")
        || lower.contains("no repositories")
        || lower.contains("repository")
        || lower.contains("failed retrieving file")
    {
        return Some(UpdateErrorCode::MissingRepository);
    }
    if lower.contains("signature") || lower.contains("gpg") || lower.contains("keyring") {
        return Some(UpdateErrorCode::SignatureError);
    }
    if lower.contains("could not resolve")
        || lower.contains("network")
        || lower.contains("timed out")
        || lower.contains("connection")
    {
        return Some(UpdateErrorCode::NetworkError);
    }
    None
}

fn parse_dnf5(stdout: &str, exit_code: i32) -> UpdateStatus {
    if exit_code == 0 {
        return up_to_date(PackageBackend::FedoraDnf5);
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(stdout) {
        if let Some(version) = value
            .get("packages")
            .and_then(|packages| packages.as_array())
            .and_then(|packages| {
                packages.iter().find_map(|package| {
                    let name = package.get("name").and_then(|name| name.as_str())?;
                    if name == PACKAGE_NAME {
                        package
                            .get("evr")
                            .or_else(|| package.get("version"))
                            .and_then(|version| version.as_str())
                    } else {
                        None
                    }
                })
            })
        {
            return available(PackageBackend::FedoraDnf5, None, Some(version.to_string()));
        }
    }

    failed_status(
        PackageBackend::FedoraDnf5,
        UpdateErrorCode::UpdateFailed,
        stdout,
    )
}

fn parse_dnf4(stdout: &str, exit_code: i32) -> UpdateStatus {
    if exit_code == 0 {
        return up_to_date(PackageBackend::FedoraDnf);
    }

    if let Some(version) = stdout.lines().find_map(parse_dnf4_line) {
        return available(PackageBackend::FedoraDnf, None, Some(version));
    }

    failed_status(
        PackageBackend::FedoraDnf,
        UpdateErrorCode::UpdateFailed,
        stdout,
    )
}

fn parse_dnf4_line(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let name = parts.next()?;
    if !name.starts_with(PACKAGE_NAME) {
        return None;
    }
    parts.next().map(str::to_string)
}

fn parse_pacman(
    backend: PackageBackend,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> UpdateStatus {
    if let Some((installed, available_version)) = stdout.lines().find_map(parse_pacman_line) {
        return available(backend, Some(installed), Some(available_version));
    }

    if exit_code == 0 || stdout.trim().is_empty() {
        return up_to_date(backend);
    }

    failed_status(backend, UpdateErrorCode::UpdateFailed, stderr)
}

fn parse_pacman_line(line: &str) -> Option<(String, String)> {
    let mut parts = line.split_whitespace();
    if parts.next()? != PACKAGE_NAME {
        return None;
    }
    let installed = parts.next()?.to_string();
    if parts.next()? != "->" {
        return None;
    }
    let available = parts.next()?.to_string();
    Some((installed, available))
}

fn parse_rpm_ostree(stdout: &str, stderr: &str, exit_code: i32) -> UpdateStatus {
    let combined = format!("{stdout}\n{stderr}").to_lowercase();
    if combined.contains("no upgrade available") || combined.contains("no update available") {
        return up_to_date(PackageBackend::BazziteRpmOstree);
    }
    if exit_code == 0 && !stdout.trim().is_empty() {
        return available(PackageBackend::BazziteRpmOstree, None, None);
    }
    if exit_code == 0 {
        return up_to_date(PackageBackend::BazziteRpmOstree);
    }
    failed_status(
        PackageBackend::BazziteRpmOstree,
        UpdateErrorCode::UpdateFailed,
        stderr,
    )
}

fn available(
    backend: PackageBackend,
    installed_version: Option<String>,
    available_version: Option<String>,
) -> UpdateStatus {
    UpdateStatus {
        backend,
        state: UpdateState::Available,
        installed_version,
        available_version,
        reboot_required: Some(false),
        summary: "Update available".to_string(),
        error_code: None,
        log_tail: Vec::new(),
    }
}

fn up_to_date(backend: PackageBackend) -> UpdateStatus {
    UpdateStatus {
        backend,
        state: UpdateState::UpToDate,
        installed_version: None,
        available_version: None,
        reboot_required: Some(false),
        summary: "ASUS Linux Control is up to date".to_string(),
        error_code: None,
        log_tail: Vec::new(),
    }
}

fn failed_status(
    backend: PackageBackend,
    error_code: UpdateErrorCode,
    output: &str,
) -> UpdateStatus {
    UpdateStatus {
        backend,
        state: UpdateState::Failed,
        installed_version: None,
        available_version: None,
        reboot_required: None,
        summary: "Package update check failed".to_string(),
        error_code: Some(error_code),
        log_tail: output
            .lines()
            .rev()
            .take(20)
            .map(str::to_string)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
    }
}
