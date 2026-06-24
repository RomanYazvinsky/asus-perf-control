use super::status::PackageBackend;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageCommandKind {
    Check,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageCommand {
    pub executable: &'static str,
    pub args: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageCommandError {
    UnsupportedBackend,
}

impl fmt::Display for PackageCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedBackend => write!(f, "unsupported package update backend"),
        }
    }
}

impl std::error::Error for PackageCommandError {}

const DNF5_CHECK: PackageCommand = PackageCommand {
    executable: "dnf5",
    args: &["check-upgrade", "asus-linux-control"],
};
const DNF5_UPDATE: PackageCommand = PackageCommand {
    executable: "dnf5",
    args: &["upgrade", "--assumeyes", "asus-linux-control"],
};
const DNF_CHECK: PackageCommand = PackageCommand {
    executable: "dnf",
    args: &["check-update", "asus-linux-control"],
};
const DNF_UPDATE: PackageCommand = PackageCommand {
    executable: "dnf",
    args: &["upgrade", "-y", "asus-linux-control"],
};
const RPM_OSTREE_CHECK: PackageCommand = PackageCommand {
    executable: "rpm-ostree",
    args: &["upgrade", "--check"],
};
const RPM_OSTREE_UPDATE: PackageCommand = PackageCommand {
    executable: "rpm-ostree",
    args: &["upgrade"],
};
const PACMAN_CHECK: PackageCommand = PackageCommand {
    executable: "pacman",
    args: &["-Qu", "asus-linux-control"],
};
const PACMAN_UPDATE: PackageCommand = PackageCommand {
    executable: "pacman",
    args: &["-Syu", "--needed", "--noconfirm", "asus-linux-control"],
};

pub fn command_for_backend(
    backend: PackageBackend,
    kind: PackageCommandKind,
) -> Result<PackageCommand, PackageCommandError> {
    match (backend, kind) {
        (PackageBackend::FedoraDnf5, PackageCommandKind::Check) => Ok(DNF5_CHECK),
        (PackageBackend::FedoraDnf5, PackageCommandKind::Update) => Ok(DNF5_UPDATE),
        (PackageBackend::FedoraDnf, PackageCommandKind::Check) => Ok(DNF_CHECK),
        (PackageBackend::FedoraDnf, PackageCommandKind::Update) => Ok(DNF_UPDATE),
        (PackageBackend::BazziteRpmOstree, PackageCommandKind::Check) => Ok(RPM_OSTREE_CHECK),
        (PackageBackend::BazziteRpmOstree, PackageCommandKind::Update) => Ok(RPM_OSTREE_UPDATE),
        (PackageBackend::ArchPacman | PackageBackend::SteamOsPacman, PackageCommandKind::Check) => {
            Ok(PACMAN_CHECK)
        }
        (
            PackageBackend::ArchPacman | PackageBackend::SteamOsPacman,
            PackageCommandKind::Update,
        ) => Ok(PACMAN_UPDATE),
        (PackageBackend::Unsupported, _) => Err(PackageCommandError::UnsupportedBackend),
    }
}
