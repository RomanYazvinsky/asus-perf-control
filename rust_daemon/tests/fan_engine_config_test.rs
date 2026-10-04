#![allow(dead_code)]

#[path = "../src/fan_engine/mod.rs"]
mod fan_engine;

use fan_engine::config::{
    clear_saved_fan_engine_config_at, read_saved_fan_engine_config_at, save_fan_engine_config_at,
    EngineCurvePoint, FanEngineConfig,
};
use std::fs;
use std::path::PathBuf;

fn config_path(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("asus-control-daemon-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp config dir should be created");
    dir.join("fan-engine-config.json")
}

fn point(temp_c: i32, speed_percent: i32) -> EngineCurvePoint {
    EngineCurvePoint {
        temp_c,
        speed_percent,
    }
}

fn valid_curve() -> Vec<EngineCurvePoint> {
    vec![
        point(30, 20),
        point(40, 30),
        point(50, 40),
        point(60, 50),
        point(70, 60),
        point(80, 70),
        point(90, 80),
        point(100, 100),
    ]
}

fn valid_config() -> FanEngineConfig {
    FanEngineConfig {
        profile: "balanced".to_string(),
        tick_ms: 1000,
        hysteresis_c: 2,
        ramp_down_delay_ms: 3000,
        emergency_temp_c: 95,
        cpu_curve: valid_curve(),
        gpu_curve: valid_curve(),
        cross_fan: false,
        cross_fan_curve: None,
    }
}

#[test]
fn validate_rejects_curve_length_not_8() {
    let mut cfg = valid_config();
    cfg.cpu_curve.pop();

    assert!(cfg.validate().is_err());
}

#[test]
fn validate_rejects_unsorted_temp_points() {
    let mut cfg = valid_config();
    cfg.cpu_curve[4] = point(65, 60);
    cfg.cpu_curve[5] = point(60, 70);

    assert!(cfg.validate().is_err());
}

#[test]
fn validate_rejects_speed_out_of_range() {
    let mut cfg = valid_config();
    cfg.cpu_curve[0] = point(30, 110);

    assert!(cfg.validate().is_err());
}

#[test]
fn validate_rejects_unsafe_speed_for_high_temp() {
    let mut cfg = valid_config();
    cfg.cpu_curve[5] = point(80, 20);

    assert!(cfg.validate().is_err());
}

#[test]
fn validate_accepts_valid_config() {
    let cfg = valid_config();
    let validated = cfg.validate().expect("config should be valid");

    assert_eq!(validated.cpu_curve.len(), 8);
    assert_eq!(validated.gpu_curve.len(), 8);
}

#[test]
fn missing_saved_fan_engine_config_returns_none() {
    let path = config_path("fan-engine-missing");
    fs::remove_file(&path).ok();

    let restored = read_saved_fan_engine_config_at(&path)
        .expect("missing fan engine config should be readable");

    assert!(restored.is_none());
}

#[test]
fn saved_fan_engine_config_round_trips_as_json() {
    let path = config_path("fan-engine-round-trip");
    let config = valid_config();

    save_fan_engine_config_at(&path, &config).expect("fan engine config should be saved");
    let restored = read_saved_fan_engine_config_at(&path)
        .expect("fan engine config should be read")
        .expect("fan engine config should exist");

    assert_eq!(restored.profile, config.profile);
    assert_eq!(restored.tick_ms, config.tick_ms);
    assert_eq!(restored.cpu_curve.len(), 8);
    assert_eq!(restored.cpu_curve[0].temp_c, 30);
    assert_eq!(restored.cpu_curve[0].speed_percent, 20);
}

#[test]
fn clear_saved_fan_engine_config_removes_file() {
    let path = config_path("fan-engine-clear");
    save_fan_engine_config_at(&path, &valid_config()).expect("fan engine config should be saved");

    clear_saved_fan_engine_config_at(&path).expect("fan engine config should be cleared");

    assert!(!path.exists());
}
