#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use hardware::fan::{set_fan_auto_mode, write_flat_fan_curve, FanChannel};
use std::fs;
use std::path::PathBuf;

fn temp_curve_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("asus-control-daemon-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

#[test]
fn write_flat_fan_curve_writes_channel_specific_points() {
    let dir = temp_curve_dir("gpu-flat");
    for point in 1..=8 {
        fs::write(dir.join(format!("pwm2_auto_point{point}_temp")), "").unwrap();
        fs::write(dir.join(format!("pwm2_auto_point{point}_pwm")), "").unwrap();
    }

    write_flat_fan_curve(&dir, FanChannel::Gpu, 180).expect("flat curve write should succeed");

    for point in 1..=8 {
        assert_eq!(
            fs::read_to_string(dir.join(format!("pwm2_auto_point{point}_pwm"))).unwrap(),
            "180"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_flat_fan_curve_enables_channel_custom_curve() {
    let dir = temp_curve_dir("gpu-enable");
    fs::write(dir.join("pwm2_enable"), "2").unwrap();
    for point in 1..=8 {
        fs::write(dir.join(format!("pwm2_auto_point{point}_temp")), "").unwrap();
        fs::write(dir.join(format!("pwm2_auto_point{point}_pwm")), "").unwrap();
    }

    write_flat_fan_curve(&dir, FanChannel::Gpu, 180).expect("flat curve write should succeed");

    assert_eq!(fs::read_to_string(dir.join("pwm2_enable")).unwrap(), "1");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn set_fan_auto_mode_writes_channel_specific_enable_file() {
    let dir = temp_curve_dir("cpu-auto");
    fs::write(dir.join("pwm1_enable"), "").unwrap();

    set_fan_auto_mode(&dir, FanChannel::Cpu).expect("auto mode write should succeed");

    assert_eq!(fs::read_to_string(dir.join("pwm1_enable")).unwrap(), "2");

    let _ = fs::remove_dir_all(&dir);
}
