#![allow(dead_code)]

#[path = "../src/fan_engine/mod.rs"]
mod fan_engine;

use fan_engine::config::{EngineCurvePoint, FanEngineConfig};
use fan_engine::runtime::{FanEngineRuntime, FanHardwareAdapter};

#[derive(Default)]
struct FakeFanHardware {
    cpu_temp: i32,
    gpu_temp: i32,
    cpu_pwm_writes: Vec<u8>,
    gpu_pwm_writes: Vec<u8>,
    cpu_auto_mode_calls: usize,
    gpu_auto_mode_calls: usize,
}

impl FakeFanHardware {
    fn with_temps(cpu_temp: i32, gpu_temp: i32) -> Self {
        Self {
            cpu_temp,
            gpu_temp,
            ..Self::default()
        }
    }

    fn last_cpu_pwm(&self) -> Option<u8> {
        self.cpu_pwm_writes.last().copied()
    }

    fn last_gpu_pwm(&self) -> Option<u8> {
        self.gpu_pwm_writes.last().copied()
    }
}

impl FanHardwareAdapter for FakeFanHardware {
    fn read_cpu_temp(&mut self) -> Result<i32, String> {
        Ok(self.cpu_temp)
    }

    fn read_gpu_temp(&mut self) -> Result<i32, String> {
        Ok(self.gpu_temp)
    }

    fn write_cpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        self.cpu_pwm_writes.push(pwm);
        Ok(())
    }

    fn write_gpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        self.gpu_pwm_writes.push(pwm);
        Ok(())
    }

    fn set_cpu_auto_mode(&mut self) -> Result<(), String> {
        self.cpu_auto_mode_calls += 1;
        Ok(())
    }

    fn set_gpu_auto_mode(&mut self) -> Result<(), String> {
        self.gpu_auto_mode_calls += 1;
        Ok(())
    }
}

fn point(temp_c: i32, speed_percent: i32) -> EngineCurvePoint {
    EngineCurvePoint {
        temp_c,
        speed_percent,
    }
}

fn curve() -> Vec<EngineCurvePoint> {
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

fn validated_config() -> fan_engine::config::ValidatedFanEngineConfig {
    FanEngineConfig {
        profile: "balanced".to_string(),
        tick_ms: 1000,
        hysteresis_c: 0,
        ramp_down_delay_ms: 0,
        emergency_temp_c: 95,
        cpu_curve: curve(),
        gpu_curve: curve(),
        cross_fan: false,
        cross_fan_curve: None,
    }
    .validate()
    .expect("valid config")
}

#[test]
fn tick_flattens_non_lead_fan_using_lead_curve_pwm() {
    let hw = FakeFanHardware::with_temps(82, 65);
    let mut engine = FanEngineRuntime::new(validated_config(), hw);

    engine.tick_once().expect("tick should pass");

    let hw = engine.hardware();
    assert_eq!(hw.last_cpu_pwm(), Some(184));
    assert_eq!(hw.last_gpu_pwm(), Some(184));
}

#[test]
fn tick_uses_gpu_as_lead_when_gpu_hotter() {
    let hw = FakeFanHardware::with_temps(62, 88);
    let mut engine = FanEngineRuntime::new(validated_config(), hw);

    engine.tick_once().expect("tick should pass");

    let hw = engine.hardware();
    assert_eq!(hw.last_gpu_pwm(), Some(199));
    assert_eq!(hw.last_cpu_pwm(), Some(199));
}

#[test]
fn tick_applies_emergency_override() {
    let hw = FakeFanHardware::with_temps(99, 70);
    let mut engine = FanEngineRuntime::new(validated_config(), hw);

    engine.tick_once().expect("tick should pass");

    let hw = engine.hardware();
    assert_eq!(hw.last_cpu_pwm(), Some(255));
    assert_eq!(hw.last_gpu_pwm(), Some(255));
}

#[test]
fn stop_restores_auto_mode_for_both_fans() {
    let hw = FakeFanHardware::with_temps(70, 70);
    let mut engine = FanEngineRuntime::new(validated_config(), hw);

    engine.stop().expect("stop should pass");

    let hw = engine.hardware();
    assert_eq!(hw.cpu_auto_mode_calls, 1);
    assert_eq!(hw.gpu_auto_mode_calls, 1);
}

#[test]
fn update_config_changes_pwm_used_by_next_tick() {
    let hw = FakeFanHardware {
        cpu_temp: 60,
        gpu_temp: 60,
        ..FakeFanHardware::default()
    };
    let mut engine = FanEngineRuntime::new(validated_config(), hw);

    let mut next = validated_config();
    for point in &mut next.cpu_curve {
        point.pwm = 77;
    }
    for point in &mut next.gpu_curve {
        point.pwm = 77;
    }

    engine.update_config(next);
    engine.tick_once().expect("tick should succeed");

    assert_eq!(engine.hardware().cpu_pwm_writes, vec![77]);
    assert_eq!(engine.hardware().gpu_pwm_writes, vec![77]);
}
