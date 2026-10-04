use std::collections::BTreeMap;
use std::sync::Mutex;

use serde_json::{json, Value};

use super::*;

fn read_raw(path: &std::path::Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn loads_defaults_when_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileAppSettingsStore::new(dir.path().join("settings.json"));

    let settings = store.load();

    assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(settings.ui.selected_navigation_index, 0);
    assert_eq!(settings.hardware.performance_mode, None);
}

#[test]
fn saves_and_loads_formatted_json() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let store = FileAppSettingsStore::new(file.clone());
    let settings = AppSettings {
        ui: UiSettings {
            selected_navigation_index: 4,
            active_hardware_profile_id: None,
        },
        hardware: HardwareSettings {
            performance_mode: Some(2),
            tdp: Some(TdpDesiredSettings {
                power_limits_enabled: true,
                sustained_w: 25,
                boost_w: 35,
                temp_limit_enabled: true,
                temp_limit_c: 85,
            }),
            cpu_boost_mode: Some("efficient_enabled".into()),
            battery_charge_limit: Some(80),
            gpu_mode: Some("hybrid".into()),
            fan: Some(FanDesiredSettings {
                selected_profile: Some("balanced".into()),
                selected_fan: Some("cpu".into()),
                control_modes: BTreeMap::from([("cpu".into(), "asusctlCurve".into())]),
                dynamic_engine_config: None,
            }),
        },
        ..AppSettings::default()
    };

    store.save(&settings).unwrap();
    let loaded = store.load();
    let raw = read_raw(&file);

    assert_eq!(loaded, settings);
    assert_eq!(raw["schemaVersion"], json!(CURRENT_SCHEMA_VERSION));
    assert_eq!(raw["hardware"]["fan"]["controlModes"]["cpu"], json!("asusctlCurve"));
    assert!(std::fs::read_to_string(&file).unwrap().contains("\n  \"ui\""));
}

#[test]
fn omits_unset_hardware_fields() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let store = FileAppSettingsStore::new(file.clone());

    store
        .save(&AppSettings {
            ui: UiSettings {
                selected_navigation_index: 1,
                active_hardware_profile_id: None,
            },
            ..AppSettings::default()
        })
        .unwrap();
    let raw = read_raw(&file);

    assert_eq!(raw["ui"]["selectedNavigationIndex"], json!(1));
    assert_eq!(raw["hardware"], json!({}));
}

#[test]
fn falls_back_to_defaults_for_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    std::fs::write(&file, "{not-json").unwrap();

    assert_eq!(FileAppSettingsStore::new(file).load(), AppSettings::default());
}

#[test]
fn keeps_valid_values_when_some_fields_are_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    std::fs::write(
        &file,
        json!({
            "schemaVersion": 2,
            "ui": {"selectedNavigationIndex": "bad"},
            "hardware": {"performanceMode": 2, "batteryChargeLimit": 130}
        })
        .to_string(),
    )
    .unwrap();

    let settings = FileAppSettingsStore::new(file).load();

    assert_eq!(settings.ui.selected_navigation_index, 0);
    assert_eq!(settings.hardware.performance_mode, Some(2));
    assert_eq!(settings.hardware.battery_charge_limit, None);
}

#[test]
fn reports_issues_with_dart_compatible_paths() {
    let json = json!({
        "ui": {"selectedNavigationIndex": "bad"},
        "hardware": {"batteryChargeLimit": 130, "tdp": {"tempLimitC": 1.5}}
    });
    let mut issues = Vec::new();

    AppSettings::from_json(json.as_object().unwrap(), &mut issues);

    assert_eq!(
        issues,
        vec![
            "ui.selectedNavigationIndex invalidType actual=String",
            "hardware.tdp.tempLimitC invalidType actual=double",
            "hardware.batteryChargeLimit outOfRange actual=130 expected=0..100",
        ]
    );
}

#[test]
fn migrates_schema_v1_to_v2_and_rewrites_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    std::fs::write(
        &file,
        json!({"schemaVersion": 1, "ui": {"selectedNavigationIndex": 2}, "hardware": {"performanceMode": 1}})
            .to_string(),
    )
    .unwrap();

    let settings = FileAppSettingsStore::new(file.clone()).load();
    let raw = read_raw(&file);

    assert_eq!(settings.schema_version, 2);
    assert_eq!(settings.ui.selected_navigation_index, 2);
    assert_eq!(settings.ui.active_hardware_profile_id, None);
    assert_eq!(settings.hardware.performance_mode, Some(1));
    assert_eq!(raw["schemaVersion"], json!(2));
    assert!(raw["ui"].get("activeHardwareProfileId").is_none());
}

#[test]
fn rejects_newer_schema_versions() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    std::fs::write(
        &file,
        json!({"schemaVersion": 99, "hardware": {"performanceMode": 1}}).to_string(),
    )
    .unwrap();

    assert_eq!(FileAppSettingsStore::new(file).load(), AppSettings::default());
}

#[test]
fn update_is_read_modify_write() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileAppSettingsStore::new(dir.path().join("settings.json"));

    update_hardware(&store, |hw| hw.performance_mode = Some(2)).unwrap();
    update_hardware(&store, |hw| hw.battery_charge_limit = Some(80)).unwrap();
    let loaded = store.load();

    assert_eq!(loaded.hardware.performance_mode, Some(2));
    assert_eq!(loaded.hardware.battery_charge_limit, Some(80));
}

// ----------------------------------------------------------------------------
// Hardware profiles
// ----------------------------------------------------------------------------

#[test]
fn sanitizes_profile_ids() {
    assert_eq!(sanitize_profile_id("Gaming Mode"), "gaming-mode");
    assert_eq!(sanitize_profile_id("  --Quiet!! Night__ "), "quiet-night");
    assert_eq!(sanitize_profile_id("Тихий"), "");
}

#[test]
fn creates_one_json_file_per_profile_and_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    let profiles_dir = dir.path().join("profiles");
    let store = FileHardwareProfileStore::new(profiles_dir.clone());

    let profile = store
        .create(
            "Gaming Mode",
            HardwareSettings {
                performance_mode: Some(2),
                ..Default::default()
            },
        )
        .unwrap();
    let listed = store.list();
    let raw = read_raw(&profiles_dir.join("gaming-mode.json"));

    assert_eq!(profile.id, "gaming-mode");
    assert_eq!(profile.name, "Gaming Mode");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].hardware.performance_mode, Some(2));
    assert_eq!(raw["schemaVersion"], json!(PROFILE_SCHEMA_VERSION));
    assert_eq!(raw["id"], json!("gaming-mode"));
    assert_eq!(raw["name"], json!("Gaming Mode"));
    assert_eq!(raw["hardware"]["performanceMode"], json!(2));
}

#[test]
fn deduplicates_colliding_ids() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileHardwareProfileStore::new(dir.path().join("profiles"));

    let first = store.create("Gaming Mode", HardwareSettings::default()).unwrap();
    let second = store.create("Gaming Mode", HardwareSettings::default()).unwrap();
    let unnamed = store.create("!!!", HardwareSettings::default()).unwrap();

    assert_eq!(first.id, "gaming-mode");
    assert_eq!(second.id, "gaming-mode-2");
    assert_eq!(unnamed.id, "profile");
    assert!(store.create("   ", HardwareSettings::default()).is_err());
}

#[test]
fn loads_saves_and_deletes_profiles() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileHardwareProfileStore::new(dir.path().join("profiles"));
    let profile = store.create("Balanced", HardwareSettings::default()).unwrap();

    store
        .save(&HardwareProfile {
            hardware: HardwareSettings {
                performance_mode: Some(0),
                battery_charge_limit: Some(80),
                ..Default::default()
            },
            ..profile.clone()
        })
        .unwrap();
    let loaded = store.load(&profile.id).unwrap();
    assert_eq!(loaded.hardware.performance_mode, Some(0));
    assert_eq!(loaded.hardware.battery_charge_limit, Some(80));
    assert!(store.load("../etc/passwd").is_none());

    store.delete(&profile.id).unwrap();
    assert!(store.list().is_empty());
}

#[test]
fn skips_invalid_profile_files() {
    let dir = tempfile::tempdir().unwrap();
    let profiles_dir = dir.path().join("profiles");
    std::fs::create_dir_all(&profiles_dir).unwrap();
    std::fs::write(profiles_dir.join("broken.json"), "{not-json").unwrap();
    std::fs::write(profiles_dir.join("nameless.json"), r#"{"id":"nameless"}"#).unwrap();

    assert!(FileHardwareProfileStore::new(profiles_dir).list().is_empty());
}

#[test]
fn lists_profiles_sorted_case_insensitively() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileHardwareProfileStore::new(dir.path().join("profiles"));
    for name in ["zeta", "Alpha", "beta"] {
        store.create(name, HardwareSettings::default()).unwrap();
    }

    let names: Vec<String> = store.list().into_iter().map(|p| p.name).collect();

    assert_eq!(names, ["Alpha", "beta", "zeta"]);
}

// ----------------------------------------------------------------------------
// Profiled store
// ----------------------------------------------------------------------------

#[derive(Default)]
struct RecordingProfiles {
    profiles: Mutex<Vec<HardwareProfile>>,
    saved: Mutex<Vec<HardwareProfile>>,
}

impl RecordingProfiles {
    fn with(profiles: Vec<HardwareProfile>) -> Self {
        Self {
            profiles: Mutex::new(profiles),
            saved: Mutex::default(),
        }
    }
}

impl HardwareProfileStore for RecordingProfiles {
    fn list(&self) -> Vec<HardwareProfile> {
        self.profiles.lock().unwrap().clone()
    }
    fn load(&self, id: &str) -> Option<HardwareProfile> {
        self.profiles.lock().unwrap().iter().find(|p| p.id == id).cloned()
    }
    fn create(&self, name: &str, hardware: HardwareSettings) -> anyhow::Result<HardwareProfile> {
        let profile = HardwareProfile::new(sanitize_profile_id(name), name.into(), hardware);
        self.save(&profile)?;
        Ok(profile)
    }
    fn save(&self, profile: &HardwareProfile) -> anyhow::Result<()> {
        let mut profiles = self.profiles.lock().unwrap();
        profiles.retain(|p| p.id != profile.id);
        profiles.push(profile.clone());
        self.saved.lock().unwrap().push(profile.clone());
        Ok(())
    }
    fn delete(&self, id: &str) -> anyhow::Result<()> {
        self.profiles.lock().unwrap().retain(|p| p.id != id);
        Ok(())
    }
}

fn gaming_profile(performance_mode: i64) -> HardwareProfile {
    HardwareProfile::new(
        "gaming".into(),
        "Gaming".into(),
        HardwareSettings {
            performance_mode: Some(performance_mode),
            battery_charge_limit: Some(80),
            ..Default::default()
        },
    )
}

fn global_with_profile(profile: Option<&str>) -> MemorySettingsStore {
    MemorySettingsStore::new(AppSettings {
        ui: UiSettings {
            selected_navigation_index: 3,
            active_hardware_profile_id: profile.map(Into::into),
        },
        hardware: HardwareSettings {
            performance_mode: Some(0),
            ..Default::default()
        },
        ..AppSettings::default()
    })
}

#[test]
fn profiled_store_merges_global_ui_with_profile_hardware() {
    let store = ProfiledAppSettingsStore::new(
        global_with_profile(Some("gaming")),
        RecordingProfiles::with(vec![gaming_profile(2)]),
    );

    let settings = store.load();

    assert_eq!(settings.ui.selected_navigation_index, 3);
    assert_eq!(settings.ui.active_hardware_profile_id.as_deref(), Some("gaming"));
    assert_eq!(settings.hardware.performance_mode, Some(2));
    assert_eq!(settings.hardware.battery_charge_limit, Some(80));
}

#[test]
fn profiled_store_updates_ui_and_profile_hardware_separately() {
    let store = ProfiledAppSettingsStore::new(
        global_with_profile(Some("gaming")),
        RecordingProfiles::with(vec![gaming_profile(2)]),
    );

    let updated = update_settings(&store, |mut s| {
        s.ui.selected_navigation_index = 7;
        s.hardware.performance_mode = Some(1);
        s
    })
    .unwrap();

    assert_eq!(updated.ui.selected_navigation_index, 7);
    assert_eq!(updated.hardware.performance_mode, Some(1));
    let global = store_global(&store);
    assert_eq!(global.ui.selected_navigation_index, 7);
    assert_eq!(global.hardware.performance_mode, Some(0));
    let saved = store.profiles().saved.lock().unwrap().clone();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].hardware.performance_mode, Some(1));
}

#[test]
fn profiled_store_without_active_profile_behaves_like_global_store() {
    let store = ProfiledAppSettingsStore::new(global_with_profile(None), RecordingProfiles::default());

    let updated = update_settings(&store, |mut s| {
        s.ui.selected_navigation_index = 4;
        s.hardware.performance_mode = Some(2);
        s
    })
    .unwrap();

    assert_eq!(updated.hardware.performance_mode, Some(2));
    assert_eq!(store_global(&store).hardware.performance_mode, Some(2));
    assert!(store.profiles().saved.lock().unwrap().is_empty());
}

#[test]
fn switching_profile_does_not_overwrite_new_profile_hardware() {
    let store = ProfiledAppSettingsStore::new(
        global_with_profile(None),
        RecordingProfiles::with(vec![gaming_profile(2)]),
    );

    update_settings(&store, |mut s| {
        s.ui.active_hardware_profile_id = Some("gaming".into());
        s
    })
    .unwrap();

    assert!(store.profiles().saved.lock().unwrap().is_empty());
    assert_eq!(store.load().hardware.performance_mode, Some(2));
}

fn store_global(store: &ProfiledAppSettingsStore<MemorySettingsStore, RecordingProfiles>) -> AppSettings {
    store.global().load()
}
