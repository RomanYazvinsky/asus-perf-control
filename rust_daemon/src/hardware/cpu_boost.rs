use super::HardwareError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const CPUFREQ_ROOT: &str = "/sys/devices/system/cpu/cpufreq";
const CPU_BOOST_CONFIG_PATH: &str = "/var/lib/asus-linux-control/cpu-boost-mode.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBoostMode {
    Disabled,
    Efficient,
    EfficientEnabled,
    Enabled,
    EfficientAggressive,
    Aggressive,
}

impl CpuBoostMode {
    pub fn daemon_id(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Efficient => "efficient",
            Self::EfficientEnabled => "efficient_enabled",
            Self::Enabled => "enabled",
            Self::EfficientAggressive => "efficient_aggressive",
            Self::Aggressive => "aggressive",
        }
    }

    pub fn from_daemon_id(value: &str) -> Option<Self> {
        match value {
            "disabled" => Some(Self::Disabled),
            "efficient" => Some(Self::Efficient),
            "efficient_enabled" => Some(Self::EfficientEnabled),
            "enabled" => Some(Self::Enabled),
            "efficient_aggressive" => Some(Self::EfficientAggressive),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuBoostStatus {
    pub is_supported: bool,
    pub current_mode: String,
    pub boost_enabled: bool,
    pub epp: Option<String>,
    pub epp_writable: bool,
    pub available_modes: Vec<String>,
}

#[derive(Debug)]
struct CpuBoostDiscovery {
    boost_paths: Vec<PathBuf>,
    policy_paths: Vec<PathBuf>,
}

pub fn get_cpu_boost_status() -> Result<CpuBoostStatus, HardwareError> {
    get_cpu_boost_status_at(Path::new(CPUFREQ_ROOT))
}

pub fn get_cpu_boost_status_at(cpufreq_root: &Path) -> Result<CpuBoostStatus, HardwareError> {
    let discovery = discover(cpufreq_root)?;
    if !discovery.is_supported() {
        return Ok(unsupported_status());
    }
    capture_missing_epp_baselines(&discovery.policy_paths)?;

    let boost_enabled = read_boost_enabled(&discovery);
    let current_mode = read_saved_cpu_boost_mode()?
        .unwrap_or(if boost_enabled {
            CpuBoostMode::Enabled
        } else {
            CpuBoostMode::Disabled
        })
        .daemon_id()
        .to_string();

    Ok(status_from_discovery(
        &discovery,
        current_mode,
        boost_enabled,
        epp_writable_from_policies(&discovery.policy_paths),
    ))
}

pub fn apply_cpu_boost_mode(mode: CpuBoostMode) -> Result<CpuBoostStatus, HardwareError> {
    let status = apply_cpu_boost_mode_at(Path::new(CPUFREQ_ROOT), mode)?;
    if status.is_supported {
        save_cpu_boost_mode(mode)?;
    }
    Ok(status)
}

pub fn apply_cpu_boost_mode_at(
    cpufreq_root: &Path,
    mode: CpuBoostMode,
) -> Result<CpuBoostStatus, HardwareError> {
    let discovery = discover(cpufreq_root)?;
    if !discovery.is_supported() {
        return Ok(unsupported_status());
    }
    capture_missing_epp_baselines(&discovery.policy_paths)?;

    let available_modes = available_modes_from_discovery(
        &discovery,
        epp_writable_from_policies(&discovery.policy_paths),
    )?;
    if !available_modes
        .iter()
        .any(|available_mode| available_mode == mode.daemon_id())
    {
        return Err(unavailable_mode_error(mode));
    }

    let boost_value = if mode == CpuBoostMode::Disabled {
        "0"
    } else {
        "1"
    };
    for path in &discovery.boost_paths {
        write_string(path, boost_value)?;
    }

    let mut epp_writable = true;
    for policy_path in &discovery.policy_paths {
        if !apply_epp(policy_path, mode)? {
            epp_writable = false;
        }
    }

    Ok(status_from_discovery(
        &discovery,
        mode.daemon_id().to_string(),
        mode != CpuBoostMode::Disabled,
        epp_writable,
    ))
}

pub fn apply_saved_cpu_boost_mode_on_startup() -> Result<(), HardwareError> {
    if let Some(mode) = read_saved_cpu_boost_mode()? {
        apply_cpu_boost_mode_at(Path::new(CPUFREQ_ROOT), mode)?;
    }
    Ok(())
}

pub fn refresh_system_epp_baseline() -> Result<(), HardwareError> {
    refresh_system_epp_baseline_at(Path::new(CPUFREQ_ROOT))
}

pub fn refresh_system_epp_baseline_at(cpufreq_root: &Path) -> Result<(), HardwareError> {
    let discovery = discover(cpufreq_root)?;
    refresh_epp_baselines(&discovery.policy_paths)
}

pub fn mode_from_daemon_id(value: &str) -> Result<CpuBoostMode, HardwareError> {
    CpuBoostMode::from_daemon_id(value).ok_or_else(|| HardwareError::WriteError {
        path: "cpu boost mode".to_string(),
        source: io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown CPU boost mode: {value}"),
        ),
    })
}

pub fn status_to_json(status: &CpuBoostStatus) -> Result<String, HardwareError> {
    serde_json::to_string(status).map_err(|e| HardwareError::ReadError {
        path: "cpu boost status json".to_string(),
        source: io::Error::new(io::ErrorKind::InvalidData, e),
    })
}

fn discover(cpufreq_root: &Path) -> Result<CpuBoostDiscovery, HardwareError> {
    let mut boost_paths = Vec::new();
    let global_boost_path = cpufreq_root.join("boost");
    if global_boost_path.exists() {
        boost_paths.push(global_boost_path);
    }

    let policy_paths = discover_prefixed_dirs(cpufreq_root, "policy")?;
    if boost_paths.is_empty() {
        for policy_path in &policy_paths {
            let boost_path = policy_path.join("boost");
            if boost_path.exists() {
                boost_paths.push(boost_path);
            }
        }
    }

    if boost_paths.is_empty() {
        if let Some(cpu_root) = cpufreq_root.parent() {
            for cpu_path in discover_prefixed_dirs(cpu_root, "cpu")? {
                let boost_path = cpu_path.join("cpufreq").join("boost");
                if boost_path.exists() {
                    boost_paths.push(boost_path);
                }
            }
        }
    }

    Ok(CpuBoostDiscovery {
        boost_paths,
        policy_paths,
    })
}

fn discover_prefixed_dirs(root: &Path, prefix: &str) -> Result<Vec<PathBuf>, HardwareError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| HardwareError::ReadError {
        path: root.to_string_lossy().to_string(),
        source: e,
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| HardwareError::ReadError {
            path: root.to_string_lossy().to_string(),
            source: e,
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with(prefix)
            && name[prefix.len()..]
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn apply_epp(policy_path: &Path, mode: CpuBoostMode) -> Result<bool, HardwareError> {
    let epp_path = policy_path.join("energy_performance_preference");
    if !epp_path.exists() {
        return Ok(true);
    }
    if is_dynamic_epp_enabled(policy_path)? {
        return Ok(false);
    }

    let target = match mode {
        CpuBoostMode::Disabled | CpuBoostMode::Enabled => {
            baseline_epp_for_policy(policy_path).or_else(|| read_trimmed(&epp_path).ok())
        }
        CpuBoostMode::Efficient => Some("power".to_string()),
        CpuBoostMode::EfficientEnabled => Some("balance_power".to_string()),
        CpuBoostMode::EfficientAggressive => Some("balance_performance".to_string()),
        CpuBoostMode::Aggressive => Some("performance".to_string()),
    };
    if let Some(target) = target {
        write_string(&epp_path, &target)?;
    }
    Ok(true)
}

fn is_dynamic_epp_enabled(policy_path: &Path) -> Result<bool, HardwareError> {
    let path = policy_path.join("dynamic_epp");
    if !path.exists() {
        return Ok(false);
    }
    let value = read_trimmed(&path)?;
    Ok(matches!(
        value.as_str(),
        "1" | "true" | "enabled" | "enable" | "y" | "Y"
    ))
}

fn read_words_optional(path: &Path) -> Result<Vec<String>, HardwareError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(read_trimmed(path)?
        .split_whitespace()
        .map(str::to_string)
        .collect())
}

fn read_trimmed(path: &Path) -> Result<String, HardwareError> {
    fs::read_to_string(path)
        .map(|value| value.trim().to_string())
        .map_err(|e| HardwareError::ReadError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })
}

fn write_string(path: &Path, value: &str) -> Result<(), HardwareError> {
    fs::write(path, value).map_err(|e| HardwareError::WriteError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })
}

fn read_boost_enabled(discovery: &CpuBoostDiscovery) -> bool {
    discovery
        .boost_paths
        .first()
        .and_then(|path| read_trimmed(path).ok())
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn epp_writable_from_policies(policy_paths: &[PathBuf]) -> bool {
    policy_paths
        .iter()
        .all(|policy_path| !is_dynamic_epp_enabled(policy_path).unwrap_or(false))
}

fn available_modes_from_discovery(
    discovery: &CpuBoostDiscovery,
    epp_writable: bool,
) -> Result<Vec<String>, HardwareError> {
    let mut modes = Vec::new();

    if !discovery.boost_paths.is_empty() {
        modes.push(CpuBoostMode::Disabled.daemon_id().to_string());
        modes.push(CpuBoostMode::Enabled.daemon_id().to_string());
    }

    if epp_writable && !discovery.policy_paths.is_empty() {
        push_if_exact_epp_available(
            &mut modes,
            &discovery.policy_paths,
            CpuBoostMode::Efficient,
            "power",
        )?;
        push_if_exact_epp_available(
            &mut modes,
            &discovery.policy_paths,
            CpuBoostMode::EfficientEnabled,
            "balance_power",
        )?;
        push_if_exact_epp_available(
            &mut modes,
            &discovery.policy_paths,
            CpuBoostMode::EfficientAggressive,
            "balance_performance",
        )?;
        push_if_exact_epp_available(
            &mut modes,
            &discovery.policy_paths,
            CpuBoostMode::Aggressive,
            "performance",
        )?;
    }

    Ok(modes)
}

fn push_if_exact_epp_available(
    modes: &mut Vec<String>,
    policy_paths: &[PathBuf],
    mode: CpuBoostMode,
    target: &str,
) -> Result<(), HardwareError> {
    if epp_target_available_on_all_policies(policy_paths, target)? {
        modes.push(mode.daemon_id().to_string());
    }
    Ok(())
}

fn epp_target_available_on_all_policies(
    policy_paths: &[PathBuf],
    target: &str,
) -> Result<bool, HardwareError> {
    if policy_paths.is_empty() {
        return Ok(false);
    }
    for policy_path in policy_paths {
        let epp_path = policy_path.join("energy_performance_preference");
        if !epp_path.exists() {
            return Ok(false);
        }
        let available =
            read_words_optional(&policy_path.join("energy_performance_available_preferences"))?;
        if !available.iter().any(|value| value == target) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn status_from_discovery(
    discovery: &CpuBoostDiscovery,
    current_mode: String,
    boost_enabled: bool,
    epp_writable: bool,
) -> CpuBoostStatus {
    let first_policy = discovery.policy_paths.first();
    CpuBoostStatus {
        is_supported: discovery.is_supported(),
        current_mode,
        boost_enabled,
        epp: first_policy
            .and_then(|path| read_trimmed(&path.join("energy_performance_preference")).ok()),
        epp_writable,
        available_modes: available_modes_from_discovery(discovery, epp_writable)
            .unwrap_or_else(|_| Vec::new()),
    }
}

fn unsupported_status() -> CpuBoostStatus {
    CpuBoostStatus {
        is_supported: false,
        current_mode: CpuBoostMode::Enabled.daemon_id().to_string(),
        boost_enabled: false,
        epp: None,
        epp_writable: false,
        available_modes: Vec::new(),
    }
}

static EPP_BASELINES: OnceLock<Mutex<Vec<(String, String)>>> = OnceLock::new();

fn epp_baselines() -> &'static Mutex<Vec<(String, String)>> {
    EPP_BASELINES.get_or_init(|| Mutex::new(Vec::new()))
}

fn capture_missing_epp_baselines(policy_paths: &[PathBuf]) -> Result<(), HardwareError> {
    let mut baselines = epp_baselines()
        .lock()
        .map_err(|_| HardwareError::WriteError {
            path: "cpu boost epp baseline cache".to_string(),
            source: io::Error::new(io::ErrorKind::Other, "EPP baseline cache mutex poisoned"),
        })?;

    for policy_path in policy_paths {
        let epp_path = policy_path.join("energy_performance_preference");
        if !epp_path.exists() {
            continue;
        }
        let key = policy_path.to_string_lossy().to_string();
        if baselines.iter().any(|(path, _)| path == &key) {
            continue;
        }
        baselines.push((key, read_trimmed(&epp_path)?));
    }

    Ok(())
}

fn refresh_epp_baselines(policy_paths: &[PathBuf]) -> Result<(), HardwareError> {
    let mut baselines = epp_baselines()
        .lock()
        .map_err(|_| HardwareError::WriteError {
            path: "cpu boost epp baseline cache".to_string(),
            source: io::Error::new(io::ErrorKind::Other, "EPP baseline cache mutex poisoned"),
        })?;

    for policy_path in policy_paths {
        let epp_path = policy_path.join("energy_performance_preference");
        if !epp_path.exists() {
            continue;
        }
        let key = policy_path.to_string_lossy().to_string();
        let value = read_trimmed(&epp_path)?;
        if let Some((_, stored_value)) = baselines.iter_mut().find(|(path, _)| path == &key) {
            *stored_value = value;
        } else {
            baselines.push((key, value));
        }
    }

    Ok(())
}

fn baseline_epp_for_policy(policy_path: &Path) -> Option<String> {
    let key = policy_path.to_string_lossy().to_string();
    epp_baselines().lock().ok().and_then(|baselines| {
        baselines
            .iter()
            .find(|(path, _)| path == &key)
            .map(|(_, value)| value.clone())
    })
}

fn unavailable_mode_error(mode: CpuBoostMode) -> HardwareError {
    HardwareError::WriteError {
        path: "cpu boost mode".to_string(),
        source: io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unavailable CPU boost mode: {}", mode.daemon_id()),
        ),
    }
}

impl CpuBoostDiscovery {
    fn is_supported(&self) -> bool {
        !self.boost_paths.is_empty() || !self.policy_paths.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
struct SavedCpuBoostMode {
    mode: String,
}

fn read_saved_cpu_boost_mode() -> Result<Option<CpuBoostMode>, HardwareError> {
    let path = Path::new(CPU_BOOST_CONFIG_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| HardwareError::ReadError {
        path: CPU_BOOST_CONFIG_PATH.to_string(),
        source: e,
    })?;
    let saved: SavedCpuBoostMode =
        serde_json::from_str(&content).map_err(|e| HardwareError::ReadError {
            path: CPU_BOOST_CONFIG_PATH.to_string(),
            source: io::Error::new(io::ErrorKind::InvalidData, e),
        })?;
    Ok(CpuBoostMode::from_daemon_id(&saved.mode))
}

fn save_cpu_boost_mode(mode: CpuBoostMode) -> Result<(), HardwareError> {
    if let Some(dir) = Path::new(CPU_BOOST_CONFIG_PATH).parent() {
        fs::create_dir_all(dir).map_err(|e| HardwareError::WriteError {
            path: dir.to_string_lossy().to_string(),
            source: e,
        })?;
    }
    let content = serde_json::to_string(&SavedCpuBoostMode {
        mode: mode.daemon_id().to_string(),
    })
    .map_err(|e| HardwareError::WriteError {
        path: CPU_BOOST_CONFIG_PATH.to_string(),
        source: io::Error::new(io::ErrorKind::InvalidData, e),
    })?;
    fs::write(CPU_BOOST_CONFIG_PATH, content).map_err(|e| HardwareError::WriteError {
        path: CPU_BOOST_CONFIG_PATH.to_string(),
        source: e,
    })
}
