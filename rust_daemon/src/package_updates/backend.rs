use super::status::PackageBackend;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct BackendDetectionInput<'a> {
    pub os_release: &'a HashMap<String, String>,
    pub executable_names: &'a HashSet<String>,
    pub pacman_db_writable: bool,
}

pub fn detect_backend_from_inputs(input: BackendDetectionInput<'_>) -> PackageBackend {
    if os_matches(input.os_release, &["bazzite"]) {
        return PackageBackend::BazziteRpmOstree;
    }

    if os_matches(input.os_release, &["steamos", "holo"]) {
        return if input.executable_names.contains("pacman") && input.pacman_db_writable {
            PackageBackend::SteamOsPacman
        } else {
            PackageBackend::Unsupported
        };
    }

    if os_matches(input.os_release, &["fedora"]) {
        if input.executable_names.contains("dnf5") {
            return PackageBackend::FedoraDnf5;
        }
        if input.executable_names.contains("dnf") {
            return PackageBackend::FedoraDnf;
        }
        return PackageBackend::Unsupported;
    }

    if os_matches(input.os_release, &["arch"]) {
        return if input.executable_names.contains("pacman") {
            PackageBackend::ArchPacman
        } else {
            PackageBackend::Unsupported
        };
    }

    PackageBackend::Unsupported
}

pub fn detect_backend() -> PackageBackend {
    let os_release = read_os_release(Path::new("/etc/os-release")).unwrap_or_default();
    let executable_names = known_executable_names(&["dnf5", "dnf", "rpm-ostree", "pacman"]);
    let pacman_db_writable = is_pacman_database_writable(Path::new("/var/lib/pacman"));

    detect_backend_from_inputs(BackendDetectionInput {
        os_release: &os_release,
        executable_names: &executable_names,
        pacman_db_writable,
    })
}

pub fn read_os_release(path: &Path) -> std::io::Result<HashMap<String, String>> {
    let contents = fs::read_to_string(path)?;
    Ok(parse_os_release(&contents))
}

pub fn parse_os_release(contents: &str) -> HashMap<String, String> {
    contents
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().to_string(),
                value.trim().trim_matches('"').to_lowercase(),
            )
        })
        .collect()
}

pub fn is_pacman_database_writable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

fn known_executable_names(candidates: &[&str]) -> HashSet<String> {
    candidates
        .iter()
        .filter(|candidate| executable_exists(candidate))
        .map(|candidate| (*candidate).to_string())
        .collect()
}

fn executable_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .any(|path| path.join(name).is_file())
}

fn os_matches(os_release: &HashMap<String, String>, needles: &[&str]) -> bool {
    ["ID", "ID_LIKE", "VARIANT_ID"]
        .iter()
        .filter_map(|key| os_release.get(*key))
        .flat_map(|value| value.split_whitespace())
        .any(|value| needles.iter().any(|needle| value.contains(needle)))
}
