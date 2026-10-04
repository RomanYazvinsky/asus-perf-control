#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use hardware::temp::{
    apply_profile_temp_setting, normalize_platform_profile, read_per_mode_settings_at,
    write_per_mode_settings_at, CpuTempProfileBucket, CpuTempProfileSetting, PerModeSettings,
};
use std::fs;
use std::path::PathBuf;

fn temp_config_path(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("asus-control-daemon-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp config dir should be created");
    dir.join("per-mode-settings.json")
}

#[test]
fn missing_per_mode_settings_file_returns_empty_defaults() {
    let path = temp_config_path("per-mode-missing");
    fs::remove_file(&path).ok();

    let settings = read_per_mode_settings_at(&path).expect("missing file should be readable");

    assert_eq!(settings.version, 1);
    assert!(settings.cpu_temp.quiet.is_none());
    assert!(settings.cpu_temp.balanced.is_none());
    assert!(settings.cpu_temp.performance.is_none());
    assert!(settings.cpu_temp.custom.is_none());
}

#[test]
fn per_mode_settings_round_trip_as_json() {
    let path = temp_config_path("per-mode-round-trip");
    let mut settings = PerModeSettings::default();
    settings.cpu_temp.quiet = Some(CpuTempProfileSetting {
        enabled: true,
        limit_c: 75,
    });
    settings.cpu_temp.performance = Some(CpuTempProfileSetting {
        enabled: false,
        limit_c: 95,
    });

    write_per_mode_settings_at(&path, &settings).expect("settings should be written");
    let restored = read_per_mode_settings_at(&path).expect("settings should be read");

    assert_eq!(restored, settings);
}

#[test]
fn quiet_and_low_power_normalize_to_quiet_bucket() {
    assert_eq!(
        normalize_platform_profile("Quiet").expect("quiet should normalize"),
        CpuTempProfileBucket::Quiet
    );
    assert_eq!(
        normalize_platform_profile("LowPower").expect("low power should normalize"),
        CpuTempProfileBucket::Quiet
    );
}

#[test]
fn platform_profiles_normalize_to_temp_buckets() {
    assert_eq!(
        normalize_platform_profile("Balanced").expect("balanced should normalize"),
        CpuTempProfileBucket::Balanced
    );
    assert_eq!(
        normalize_platform_profile("Performance").expect("performance should normalize"),
        CpuTempProfileBucket::Performance
    );
    assert_eq!(
        normalize_platform_profile("Custom").expect("custom should normalize"),
        CpuTempProfileBucket::Custom
    );
    assert!(normalize_platform_profile("Unknown").is_err());
}

#[test]
fn apply_profile_temp_setting_applies_enabled_non_zero_limit_only() {
    let mut applied = Vec::new();
    let mut settings = PerModeSettings::default();
    settings.cpu_temp.balanced = Some(CpuTempProfileSetting {
        enabled: true,
        limit_c: 85,
    });

    apply_profile_temp_setting(&settings, "Balanced", |limit| {
        applied.push(limit);
        Ok(())
    })
    .expect("enabled setting should apply");

    assert_eq!(applied, vec![85]);
}

#[test]
fn apply_profile_temp_setting_skips_disabled_zero_or_missing_settings() {
    let mut applied = Vec::new();
    let mut settings = PerModeSettings::default();
    settings.cpu_temp.quiet = Some(CpuTempProfileSetting {
        enabled: false,
        limit_c: 75,
    });
    settings.cpu_temp.performance = Some(CpuTempProfileSetting {
        enabled: true,
        limit_c: 0,
    });

    apply_profile_temp_setting(&settings, "Quiet", |limit| {
        applied.push(limit);
        Ok(())
    })
    .expect("disabled setting should be skipped");
    apply_profile_temp_setting(&settings, "Performance", |limit| {
        applied.push(limit);
        Ok(())
    })
    .expect("zero setting should be skipped");
    apply_profile_temp_setting(&settings, "Custom", |limit| {
        applied.push(limit);
        Ok(())
    })
    .expect("missing setting should be skipped");

    assert!(applied.is_empty());
}

#[test]
fn apply_profile_temp_setting_rejects_invalid_non_zero_limits() {
    let mut settings = PerModeSettings::default();
    settings.cpu_temp.balanced = Some(CpuTempProfileSetting {
        enabled: true,
        limit_c: 101,
    });

    let error = apply_profile_temp_setting(&settings, "Balanced", |_| Ok(()))
        .expect_err("invalid enabled limit should fail");

    assert!(error.to_string().contains("out of range"));
}
