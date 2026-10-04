//! Settings and hardware profile persistence.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use super::json::JsonMap;
use super::model::*;
use crate::logging::log_event;

pub type SettingsUpdate<'a> = &'a mut dyn FnMut(AppSettings) -> AppSettings;

pub trait AppSettingsStore: Send + Sync {
    fn load(&self) -> AppSettings;
    fn save(&self, settings: &AppSettings) -> Result<()>;
    /// Serialised read-modify-write.
    fn update(&self, update: SettingsUpdate<'_>) -> Result<AppSettings>;
}

/// Convenience wrapper so call sites can pass a closure directly.
pub fn update_settings(
    store: &dyn AppSettingsStore,
    mut update: impl FnMut(AppSettings) -> AppSettings,
) -> Result<AppSettings> {
    store.update(&mut update)
}

/// Convenience wrapper for the common "change one hardware field" update.
pub fn update_hardware(
    store: &dyn AppSettingsStore,
    mut update: impl FnMut(&mut HardwareSettings),
) -> Result<AppSettings> {
    update_settings(store, |mut settings| {
        update(&mut settings.hardware);
        settings
    })
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

fn log_file_mismatch(file: &Path, issue: &str) {
    log_event(
        "Settings.readMismatch",
        &[("file", file.display().to_string()), ("issue", issue.to_string())],
    );
}

fn read_json_object(file: &Path) -> std::result::Result<JsonMap, String> {
    let text = fs::read_to_string(file).map_err(|e| format!("invalidJson error={e}"))?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(other) => Err(format!(
            "root invalidType actual={}",
            match other {
                Value::Array(_) => "List<dynamic>",
                Value::String(_) => "String",
                Value::Number(_) => "num",
                Value::Bool(_) => "bool",
                _ => "Null",
            }
        )),
        Err(e) => Err(format!("invalidJson error={e}")),
    }
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

pub struct MigrationResult {
    pub json: JsonMap,
    pub migrated: bool,
    pub issues: Vec<String>,
}

fn read_version(json: &JsonMap) -> i64 {
    json.get("schemaVersion").and_then(Value::as_i64).unwrap_or(1)
}

/// v1 -> v2 only bumps the version (v2 added `ui.activeHardwareProfileId`).
pub fn migrate_settings(source: JsonMap) -> MigrationResult {
    let initial = read_version(&source);
    if initial > CURRENT_SCHEMA_VERSION {
        return MigrationResult {
            json: source,
            migrated: false,
            issues: vec![format!(
                "schemaVersionUnsupported actual={initial} expected={CURRENT_SCHEMA_VERSION}"
            )],
        };
    }
    let mut json = source;
    let mut version = initial;
    let mut issues = Vec::new();
    while version < CURRENT_SCHEMA_VERSION {
        match version {
            1 => {
                json.insert("schemaVersion".into(), Value::from(2));
                version = 2;
            }
            other => {
                issues.push(format!("migrationMissing fromVersion={other}"));
                break;
            }
        }
    }
    MigrationResult {
        json,
        migrated: version != initial,
        issues,
    }
}

// ---------------------------------------------------------------------------
// Global settings file
// ---------------------------------------------------------------------------

pub struct FileAppSettingsStore {
    file: PathBuf,
    lock: Mutex<()>,
}

impl FileAppSettingsStore {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            lock: Mutex::new(()),
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    /// `$XDG_CONFIG_HOME/asus-linux-control/settings.json` (absolute XDG path only),
    /// otherwise `$HOME/.config/...`.
    pub fn default_settings_file() -> PathBuf {
        let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_default();
        let base = if !config_home.is_empty() && config_home.starts_with('/') {
            PathBuf::from(config_home)
        } else {
            let home = std::env::var("HOME").unwrap_or_default();
            let home = if home.is_empty() { ".".to_string() } else { home };
            PathBuf::from(home).join(".config")
        };
        base.join("asus-linux-control").join("settings.json")
    }

    fn load_unlocked(&self) -> AppSettings {
        if !self.file.exists() {
            return AppSettings::default();
        }
        let json = match read_json_object(&self.file) {
            Ok(json) => json,
            Err(issue) => {
                log_file_mismatch(&self.file, &issue);
                return AppSettings::default();
            }
        };
        let migration = migrate_settings(json);
        for issue in &migration.issues {
            log_file_mismatch(&self.file, issue);
        }
        if !migration.issues.is_empty() {
            return AppSettings::default();
        }
        let mut issues = Vec::new();
        let settings = AppSettings::from_json(&migration.json, &mut issues);
        for issue in &issues {
            log_file_mismatch(&self.file, issue);
        }
        if migration.migrated {
            if let Err(e) = self.save_unlocked(&settings) {
                log::warn!("failed to save migrated settings: {e:#}");
            }
        }
        settings
    }

    fn save_unlocked(&self, settings: &AppSettings) -> Result<()> {
        write_json(&self.file, &settings.to_json())
    }
}

impl AppSettingsStore for FileAppSettingsStore {
    fn load(&self) -> AppSettings {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.load_unlocked()
    }

    fn save(&self, settings: &AppSettings) -> Result<()> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.save_unlocked(settings)
    }

    fn update(&self, update: SettingsUpdate<'_>) -> Result<AppSettings> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let next = update(self.load_unlocked());
        self.save_unlocked(&next)?;
        Ok(next)
    }
}

// ---------------------------------------------------------------------------
// Hardware profiles (one JSON file per profile)
// ---------------------------------------------------------------------------

pub trait HardwareProfileStore: Send + Sync {
    fn list(&self) -> Vec<HardwareProfile>;
    fn load(&self, id: &str) -> Option<HardwareProfile>;
    fn create(&self, name: &str, hardware: HardwareSettings) -> Result<HardwareProfile>;
    fn save(&self, profile: &HardwareProfile) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
}

pub struct FileHardwareProfileStore {
    directory: PathBuf,
}

impl FileHardwareProfileStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Profiles live next to the settings file: `<settings dir>/profiles`.
    pub fn for_settings_file(settings_file: &Path) -> Self {
        let dir = settings_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::new(dir.join("profiles"))
    }

    fn file_for_id(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{id}.json"))
    }

    fn load_file(&self, file: &Path) -> Option<HardwareProfile> {
        let json = match read_json_object(file) {
            Ok(json) => json,
            Err(issue) => {
                log_file_mismatch(file, &issue);
                return None;
            }
        };
        let mut issues = Vec::new();
        let profile = HardwareProfile::from_json(&json, &mut issues);
        for issue in &issues {
            log_file_mismatch(file, issue);
        }
        if issues
            .iter()
            .any(|issue| issue.starts_with("id ") || issue.starts_with("name "))
        {
            return None;
        }
        Some(profile)
    }

    fn next_available_id(&self, base: &str) -> String {
        let base = if base.is_empty() { "profile" } else { base };
        let mut candidate = base.to_string();
        let mut suffix = 2;
        while self.file_for_id(&candidate).exists() {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        candidate
    }
}

/// Lowercase ASCII letters/digits, other runs collapsed to a single `-`,
/// no leading or trailing separators.
pub fn sanitize_profile_id(source: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;
    for c in source.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            last_was_separator = false;
        } else if !last_was_separator && !out.is_empty() {
            out.push('-');
            last_was_separator = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn is_safe_id(id: &str) -> bool {
    let safe = sanitize_profile_id(id);
    !safe.is_empty() && safe == id
}

impl HardwareProfileStore for FileHardwareProfileStore {
    fn list(&self) -> Vec<HardwareProfile> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let mut profiles: Vec<HardwareProfile> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
            .filter_map(|path| self.load_file(&path))
            .collect();
        profiles.sort_by_key(|profile| profile.name.to_lowercase());
        profiles
    }

    fn load(&self, id: &str) -> Option<HardwareProfile> {
        if !is_safe_id(id) {
            return None;
        }
        let file = self.file_for_id(id);
        if !file.exists() {
            return None;
        }
        self.load_file(&file)
    }

    fn create(&self, name: &str, hardware: HardwareSettings) -> Result<HardwareProfile> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            bail!("Profile name cannot be blank");
        }
        let id = self.next_available_id(&sanitize_profile_id(trimmed));
        let profile = HardwareProfile::new(id, trimmed.to_string(), hardware);
        self.save(&profile)?;
        Ok(profile)
    }

    fn save(&self, profile: &HardwareProfile) -> Result<()> {
        if !is_safe_id(&profile.id) {
            bail!("Invalid profile id: {}", profile.id);
        }
        write_json(&self.file_for_id(&profile.id), &profile.to_json())
    }

    fn delete(&self, id: &str) -> Result<()> {
        if !is_safe_id(id) {
            return Ok(());
        }
        let file = self.file_for_id(id);
        if file.exists() {
            fs::remove_file(&file).with_context(|| format!("delete {}", file.display()))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Profiled store: UI settings in the global file, hardware in the active profile
// ---------------------------------------------------------------------------

pub struct ProfiledAppSettingsStore<G, P> {
    global: G,
    profiles: P,
    lock: Mutex<()>,
}

impl<G: AppSettingsStore, P: HardwareProfileStore> ProfiledAppSettingsStore<G, P> {
    pub fn new(global: G, profiles: P) -> Self {
        Self {
            global,
            profiles,
            lock: Mutex::new(()),
        }
    }

    pub fn global(&self) -> &G {
        &self.global
    }

    pub fn profiles(&self) -> &P {
        &self.profiles
    }

    fn with_active_profile_hardware(&self, global: AppSettings) -> AppSettings {
        let Some(id) = &global.ui.active_hardware_profile_id else {
            return global;
        };
        match self.profiles.load(id) {
            Some(profile) => AppSettings {
                hardware: profile.hardware,
                ..global
            },
            None => global,
        }
    }
}

impl<G: AppSettingsStore, P: HardwareProfileStore> AppSettingsStore for ProfiledAppSettingsStore<G, P> {
    fn load(&self) -> AppSettings {
        self.with_active_profile_hardware(self.global.load())
    }

    fn save(&self, settings: &AppSettings) -> Result<()> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let global = self.global.load();
        let profile = settings
            .ui
            .active_hardware_profile_id
            .as_deref()
            .and_then(|id| self.profiles.load(id));
        let Some(profile) = profile else {
            return self.global.save(settings);
        };
        self.global.save(&AppSettings {
            schema_version: settings.schema_version,
            ui: settings.ui.clone(),
            ..global
        })?;
        self.profiles.save(&HardwareProfile {
            hardware: settings.hardware.clone(),
            ..profile
        })
    }

    fn update(&self, update: SettingsUpdate<'_>) -> Result<AppSettings> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let global = self.global.load();
        let current = self.with_active_profile_hardware(global.clone());
        let next = update(current.clone());

        let profile = next
            .ui
            .active_hardware_profile_id
            .as_deref()
            .and_then(|id| self.profiles.load(id));
        let Some(profile) = profile else {
            self.global.save(&next)?;
            return Ok(next);
        };

        self.global.save(&AppSettings {
            schema_version: next.schema_version,
            ui: next.ui.clone(),
            ..global
        })?;

        let profile_changed = next.ui.active_hardware_profile_id != current.ui.active_hardware_profile_id;
        let hardware_changed = next.hardware != current.hardware;
        if !profile_changed || hardware_changed {
            self.profiles.save(&HardwareProfile {
                hardware: next.hardware.clone(),
                ..profile
            })?;
        }
        Ok(next)
    }
}

// ---------------------------------------------------------------------------
// In-memory store (tests and fallbacks)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct MemorySettingsStore {
    settings: Mutex<AppSettings>,
}

impl MemorySettingsStore {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
        }
    }
}

impl AppSettingsStore for MemorySettingsStore {
    fn load(&self) -> AppSettings {
        self.settings.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn save(&self, settings: &AppSettings) -> Result<()> {
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = settings.clone();
        Ok(())
    }

    fn update(&self, update: SettingsUpdate<'_>) -> Result<AppSettings> {
        let mut guard = self.settings.lock().unwrap_or_else(|e| e.into_inner());
        let next = update(guard.clone());
        *guard = next.clone();
        Ok(next)
    }
}
