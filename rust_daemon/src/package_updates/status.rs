use serde::ser::SerializeStruct;
use serde::Serialize;

pub const UPDATE_STATUS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBackend {
    FedoraDnf,
    FedoraDnf5,
    BazziteRpmOstree,
    ArchPacman,
    SteamOsPacman,
    Unsupported,
}

impl PackageBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FedoraDnf => "fedora-dnf",
            Self::FedoraDnf5 => "fedora-dnf5",
            Self::BazziteRpmOstree => "bazzite-rpm-ostree",
            Self::ArchPacman => "arch-pacman",
            Self::SteamOsPacman => "steamos-pacman",
            Self::Unsupported => "unsupported",
        }
    }
}

impl Serialize for PackageBackend {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UpdateState {
    Idle,
    Checking,
    Available,
    UpToDate,
    Running,
    Succeeded,
    Failed,
    Unsupported,
}

impl UpdateState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Checking => "checking",
            Self::Available => "available",
            Self::UpToDate => "upToDate",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        }
    }
}

impl Serialize for UpdateState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UpdateErrorCode {
    UnsupportedOs,
    MissingRepository,
    PackageManagerUnavailable,
    PackageDatabaseLocked,
    SignatureError,
    NetworkError,
    AuthorizationDenied,
    UpdateFailed,
    RebootRequired,
}

impl UpdateErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedOs => "unsupported_os",
            Self::MissingRepository => "missing_repository",
            Self::PackageManagerUnavailable => "package_manager_unavailable",
            Self::PackageDatabaseLocked => "package_database_locked",
            Self::SignatureError => "signature_error",
            Self::NetworkError => "network_error",
            Self::AuthorizationDenied => "authorization_denied",
            Self::UpdateFailed => "update_failed",
            Self::RebootRequired => "reboot_required",
        }
    }
}

impl Serialize for UpdateErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateStatus {
    pub backend: PackageBackend,
    pub state: UpdateState,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub reboot_required: Option<bool>,
    pub summary: String,
    pub error_code: Option<UpdateErrorCode>,
    pub log_tail: Vec<String>,
}

impl UpdateStatus {
    pub fn new(backend: PackageBackend, state: UpdateState, summary: impl Into<String>) -> Self {
        Self {
            backend,
            state,
            installed_version: None,
            available_version: None,
            reboot_required: None,
            summary: summary.into(),
            error_code: None,
            log_tail: Vec::new(),
        }
    }

    pub fn unsupported(summary: impl Into<String>) -> Self {
        Self {
            error_code: Some(UpdateErrorCode::UnsupportedOs),
            ..Self::new(
                PackageBackend::Unsupported,
                UpdateState::Unsupported,
                summary,
            )
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

impl Serialize for UpdateStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("UpdateStatus", 9)?;
        state.serialize_field("schemaVersion", &UPDATE_STATUS_SCHEMA_VERSION)?;
        state.serialize_field("backend", &self.backend)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("installedVersion", &self.installed_version)?;
        state.serialize_field("availableVersion", &self.available_version)?;
        state.serialize_field("rebootRequired", &self.reboot_required)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("errorCode", &self.error_code)?;
        state.serialize_field("logTail", &self.log_tail)?;
        state.end()
    }
}
