//! Native package updates through the daemon — former `PackageUpdateBloc`.

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use serde_json::Value;

macro_rules! wire_enum {
    ($name:ident, $what:literal { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn wire_name(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }

            pub fn from_wire(value: &str) -> Result<Self> {
                match value {
                    $($wire => Ok(Self::$variant),)+
                    other => bail!(concat!("Unknown ", $what, ": {}"), other),
                }
            }
        }
    };
}

wire_enum!(PackageUpdateBackend, "package backend" {
    FedoraDnf => "fedora-dnf",
    FedoraDnf5 => "fedora-dnf5",
    BazziteRpmOstree => "bazzite-rpm-ostree",
    ArchPacman => "arch-pacman",
    SteamOsPacman => "steamos-pacman",
    Unsupported => "unsupported",
});

wire_enum!(PackageUpdateRunState, "update state" {
    Idle => "idle",
    Checking => "checking",
    Available => "available",
    UpToDate => "upToDate",
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    Unsupported => "unsupported",
});

wire_enum!(PackageUpdateErrorCode, "update error code" {
    UnsupportedOs => "unsupported_os",
    MissingRepository => "missing_repository",
    PackageManagerUnavailable => "package_manager_unavailable",
    PackageDatabaseLocked => "package_database_locked",
    SignatureError => "signature_error",
    NetworkError => "network_error",
    AuthorizationDenied => "authorization_denied",
    UpdateFailed => "update_failed",
    RebootRequired => "reboot_required",
});

#[derive(Debug, Clone, PartialEq)]
pub struct PackageUpdateStatus {
    pub schema_version: i64,
    pub backend: PackageUpdateBackend,
    pub state: PackageUpdateRunState,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub reboot_required: Option<bool>,
    pub summary: String,
    pub error_code: Option<PackageUpdateErrorCode>,
    pub log_tail: Vec<String>,
}

impl PackageUpdateStatus {
    pub fn from_json_str(text: &str) -> Result<Self> {
        let Value::Object(json) = serde_json::from_str::<Value>(text)? else {
            bail!("Package update status must be a JSON object");
        };
        let schema_version = json.get("schemaVersion").and_then(Value::as_i64);
        if schema_version != Some(1) {
            bail!(
                "Unsupported update status schema: {}",
                json.get("schemaVersion").map_or("null".to_string(), Value::to_string)
            );
        }
        let string = |key: &str| -> Result<String> {
            json.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("Missing string field: {key}"))
        };
        let optional_string = |key: &str| json.get(key).and_then(Value::as_str).map(str::to_string);
        Ok(Self {
            schema_version: 1,
            backend: PackageUpdateBackend::from_wire(&string("backend")?)?,
            state: PackageUpdateRunState::from_wire(&string("state")?)?,
            installed_version: optional_string("installedVersion"),
            available_version: optional_string("availableVersion"),
            reboot_required: json.get("rebootRequired").and_then(Value::as_bool),
            summary: string("summary")?,
            error_code: match json.get("errorCode") {
                None | Some(Value::Null) => None,
                Some(value) => Some(PackageUpdateErrorCode::from_wire(
                    value.as_str().ok_or_else(|| anyhow!("errorCode must be a string"))?,
                )?),
            },
            log_tail: match json.get("logTail") {
                Some(Value::Array(lines)) => lines
                    .iter()
                    .map(|line| match line {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect(),
                _ => Vec::new(),
            },
        })
    }
}

#[async_trait]
pub trait PackageUpdateService: Send + Sync {
    async fn get_package_backend(&self) -> Result<PackageUpdateBackend>;
    async fn check_for_updates(&self) -> Result<PackageUpdateStatus>;
    async fn start_update(&self) -> Result<String>;
    async fn get_update_status(&self, update_id: &str) -> Result<PackageUpdateStatus>;
    async fn cancel_update(&self, update_id: &str) -> Result<bool>;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackageUpdateState {
    pub status: Option<PackageUpdateStatus>,
    pub backend: Option<PackageUpdateBackend>,
    pub update_id: Option<String>,
    pub is_checking: bool,
    pub daemon_unavailable: bool,
    pub last_error: Option<String>,
}

impl PackageUpdateState {
    pub fn is_update_running(&self) -> bool {
        self.run_state() == Some(PackageUpdateRunState::Running)
    }

    pub fn can_start_update(&self) -> bool {
        !self.daemon_unavailable && !self.is_checking && self.run_state() == Some(PackageUpdateRunState::Available)
    }

    fn run_state(&self) -> Option<PackageUpdateRunState> {
        self.status.as_ref().map(|s| s.state)
    }

    pub fn summary(&self) -> String {
        if self.daemon_unavailable {
            return "Package update daemon is unavailable".into();
        }
        let Some(status) = &self.status else {
            return "Package update status is not loaded".into();
        };
        if status.reboot_required == Some(true) {
            return "Restart required to finish updating".into();
        }
        match status.state {
            PackageUpdateRunState::Running => "Package update is running".into(),
            PackageUpdateRunState::UpToDate => "ASUS Linux Control is up to date".into(),
            PackageUpdateRunState::Unsupported => "Package updates are not supported on this system".into(),
            PackageUpdateRunState::Failed => error_summary(status.error_code).into(),
            _ => status.summary.clone(),
        }
    }

    pub fn backend_label(&self) -> &'static str {
        self.status
            .as_ref()
            .map(|s| s.backend)
            .or(self.backend)
            .map_or("unknown", PackageUpdateBackend::wire_name)
    }

    fn unavailable(error: &anyhow::Error) -> Self {
        Self {
            daemon_unavailable: true,
            last_error: Some(format!("{error:#}")),
            ..Self::default()
        }
    }

    fn failed_status(&self, error: &anyhow::Error) -> PackageUpdateStatus {
        let message = format!("{error:#}");
        PackageUpdateStatus {
            schema_version: 1,
            backend: self.backend.unwrap_or(PackageUpdateBackend::Unsupported),
            state: PackageUpdateRunState::Failed,
            installed_version: None,
            available_version: None,
            reboot_required: None,
            summary: "Package update failed".into(),
            error_code: Some(if message.contains("authorization_denied") {
                PackageUpdateErrorCode::AuthorizationDenied
            } else {
                PackageUpdateErrorCode::UpdateFailed
            }),
            log_tail: vec![message],
        }
    }
}

fn error_summary(code: Option<PackageUpdateErrorCode>) -> &'static str {
    use PackageUpdateErrorCode as C;
    match code {
        Some(C::MissingRepository) => "Package repository is not configured",
        Some(C::SignatureError) => "Package signature verification failed",
        Some(C::PackageDatabaseLocked) => "Package database is locked",
        Some(C::UnsupportedOs) => "Package updates are not supported on this system",
        Some(C::AuthorizationDenied) => "Authorization was denied",
        Some(C::NetworkError) => "Network error while checking updates",
        Some(C::PackageManagerUnavailable) => "Package manager is unavailable",
        Some(C::RebootRequired) => "Restart required to finish updating",
        Some(C::UpdateFailed) | None => "Package update failed",
    }
}

/// Initial load: backend + status.
pub async fn load(service: &dyn PackageUpdateService) -> PackageUpdateState {
    let result: Result<PackageUpdateState> = async {
        let backend = service.get_package_backend().await?;
        let status = service.check_for_updates().await?;
        Ok(PackageUpdateState {
            status: Some(status),
            backend: Some(backend),
            ..Default::default()
        })
    }
    .await;
    result.unwrap_or_else(|e| PackageUpdateState::unavailable(&e))
}

pub async fn check(service: &dyn PackageUpdateService, current: PackageUpdateState) -> PackageUpdateState {
    match service.check_for_updates().await {
        Ok(status) => PackageUpdateState {
            status: Some(status),
            is_checking: false,
            ..current
        },
        Err(e) => PackageUpdateState::unavailable(&e),
    }
}

pub async fn start(service: &dyn PackageUpdateService, current: PackageUpdateState) -> PackageUpdateState {
    let result: Result<(String, PackageUpdateStatus)> = async {
        let update_id = service.start_update().await?;
        let status = service.get_update_status(&update_id).await?;
        Ok((update_id, status))
    }
    .await;
    match result {
        Ok((update_id, status)) => PackageUpdateState {
            status: Some(status),
            update_id: Some(update_id),
            ..Default::default()
        },
        Err(e) => PackageUpdateState {
            status: Some(current.failed_status(&e)),
            ..current
        },
    }
}

pub async fn poll(service: &dyn PackageUpdateService, current: PackageUpdateState) -> PackageUpdateState {
    let Some(update_id) = current.update_id.clone() else {
        return current;
    };
    match service.get_update_status(&update_id).await {
        Ok(status) => PackageUpdateState {
            status: Some(status),
            ..current
        },
        Err(e) => PackageUpdateState::unavailable(&e),
    }
}

pub async fn cancel(service: &dyn PackageUpdateService, current: PackageUpdateState) -> PackageUpdateState {
    let Some(update_id) = current.update_id.clone() else {
        return current;
    };
    let result: Result<PackageUpdateStatus> = async {
        service.cancel_update(&update_id).await?;
        service.get_update_status(&update_id).await
    }
    .await;
    match result {
        Ok(status) => PackageUpdateState {
            status: Some(status),
            update_id: None,
            ..current
        },
        Err(e) => PackageUpdateState {
            status: Some(current.failed_status(&e)),
            ..current
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_json(state: &str, extra: &str) -> String {
        format!(
            r#"{{"schemaVersion":1,"backend":"fedora-dnf5","state":"{state}","installedVersion":"0.0.1",
                "availableVersion":"0.0.2","rebootRequired":false,"summary":"Update available",
                "errorCode":null,"logTail":["line 1"]{extra}}}"#
        )
    }

    #[test]
    fn parses_status_payload() {
        let status = PackageUpdateStatus::from_json_str(&status_json("available", "")).unwrap();

        assert_eq!(status.backend, PackageUpdateBackend::FedoraDnf5);
        assert_eq!(status.state, PackageUpdateRunState::Available);
        assert_eq!(status.installed_version.as_deref(), Some("0.0.1"));
        assert_eq!(status.reboot_required, Some(false));
        assert_eq!(status.log_tail, vec!["line 1"]);
    }

    #[test]
    fn rejects_unknown_schema_and_values() {
        assert!(PackageUpdateStatus::from_json_str(r#"{"schemaVersion":2}"#).is_err());
        assert!(PackageUpdateStatus::from_json_str("[]").is_err());
        assert!(PackageUpdateStatus::from_json_str(&status_json("exploded", "")).is_err());
        assert!(PackageUpdateBackend::from_wire("apt").is_err());
    }

    #[test]
    fn summaries_follow_state() {
        let mut state = PackageUpdateState::default();
        assert_eq!(state.summary(), "Package update status is not loaded");

        state.status = Some(PackageUpdateStatus::from_json_str(&status_json("upToDate", "")).unwrap());
        assert_eq!(state.summary(), "ASUS Linux Control is up to date");

        let mut failed = PackageUpdateStatus::from_json_str(&status_json("failed", "")).unwrap();
        failed.error_code = Some(PackageUpdateErrorCode::SignatureError);
        state.status = Some(failed);
        assert_eq!(state.summary(), "Package signature verification failed");

        state.daemon_unavailable = true;
        assert_eq!(state.summary(), "Package update daemon is unavailable");
    }

    #[test]
    fn can_start_only_when_available() {
        let mut state = PackageUpdateState {
            status: Some(PackageUpdateStatus::from_json_str(&status_json("available", "")).unwrap()),
            ..Default::default()
        };
        assert!(state.can_start_update());
        state.is_checking = true;
        assert!(!state.can_start_update());
    }

    struct Failing;

    #[async_trait]
    impl PackageUpdateService for Failing {
        async fn get_package_backend(&self) -> Result<PackageUpdateBackend> {
            bail!("Package update daemon unavailable: no bus")
        }
        async fn check_for_updates(&self) -> Result<PackageUpdateStatus> {
            bail!("no bus")
        }
        async fn start_update(&self) -> Result<String> {
            bail!("authorization_denied")
        }
        async fn get_update_status(&self, _: &str) -> Result<PackageUpdateStatus> {
            bail!("no bus")
        }
        async fn cancel_update(&self, _: &str) -> Result<bool> {
            bail!("no bus")
        }
    }

    #[tokio::test]
    async fn failures_map_to_unavailable_or_failed_status() {
        let loaded = load(&Failing).await;
        assert!(loaded.daemon_unavailable);

        let started = start(&Failing, PackageUpdateState::default()).await;
        let status = started.status.unwrap();
        assert_eq!(status.state, PackageUpdateRunState::Failed);
        assert_eq!(status.error_code, Some(PackageUpdateErrorCode::AuthorizationDenied));
    }
}
