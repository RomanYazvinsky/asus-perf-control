#![allow(dead_code)]

#[path = "../src/fan_engine/mod.rs"]
mod fan_engine;

use fan_engine::config::{EngineCurvePoint, FanEngineConfig};
use fan_engine::controller::FanEngineDriver;
use fan_engine::runtime::FanHardwareAdapter;
use fan_engine::threaded_driver::ThreadedFanEngineDriver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct SharedState {
    cpu_writes: Vec<u8>,
    gpu_writes: Vec<u8>,
    cpu_auto: usize,
    gpu_auto: usize,
}

struct FakeHardware {
    state: Arc<Mutex<SharedState>>,
    cpu_temp: i32,
    gpu_temp: i32,
    fail_cpu_auto: bool,
    fail_gpu_auto: bool,
    panic_on_read: bool,
}

impl FanHardwareAdapter for FakeHardware {
    fn read_cpu_temp(&mut self) -> Result<i32, String> {
        if self.panic_on_read {
            panic!("cpu temp read panic");
        }
        Ok(self.cpu_temp)
    }

    fn read_gpu_temp(&mut self) -> Result<i32, String> {
        Ok(self.gpu_temp)
    }

    fn write_cpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        self.state.lock().unwrap().cpu_writes.push(pwm);
        Ok(())
    }

    fn write_gpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        self.state.lock().unwrap().gpu_writes.push(pwm);
        Ok(())
    }

    fn set_cpu_auto_mode(&mut self) -> Result<(), String> {
        self.state.lock().unwrap().cpu_auto += 1;
        if self.fail_cpu_auto {
            return Err("cpu auto restore failed".to_string());
        }
        Ok(())
    }

    fn set_gpu_auto_mode(&mut self) -> Result<(), String> {
        self.state.lock().unwrap().gpu_auto += 1;
        if self.fail_gpu_auto {
            return Err("gpu auto restore failed".to_string());
        }
        Ok(())
    }
}

fn point(temp_c: i32, speed_percent: i32) -> EngineCurvePoint {
    EngineCurvePoint {
        temp_c,
        speed_percent,
    }
}

fn curve(speed_percent: i32) -> Vec<EngineCurvePoint> {
    vec![
        point(30, speed_percent),
        point(40, speed_percent),
        point(50, speed_percent),
        point(60, speed_percent),
        point(70, speed_percent),
        point(80, speed_percent.max(30)),
        point(90, speed_percent.max(30)),
        point(100, 100),
    ]
}

fn config(speed_percent: i32) -> fan_engine::config::ValidatedFanEngineConfig {
    FanEngineConfig {
        profile: "balanced".to_string(),
        tick_ms: 25,
        hysteresis_c: 2,
        ramp_down_delay_ms: 3000,
        emergency_temp_c: 95,
        cpu_curve: curve(speed_percent),
        gpu_curve: curve(speed_percent),
        cross_fan: false,
        cross_fan_curve: None,
    }
    .validate()
    .expect("config should validate")
}

#[test]
fn threaded_driver_ticks_and_stop_restores_auto() {
    let state = Arc::new(Mutex::new(SharedState::default()));
    let hardware_state = state.clone();
    let mut driver = ThreadedFanEngineDriver::new(Box::new(move || {
        Ok(Box::new(FakeHardware {
            state: hardware_state.clone(),
            cpu_temp: 60,
            gpu_temp: 60,
            fail_cpu_auto: false,
            fail_gpu_auto: false,
            panic_on_read: false,
        }))
    }));

    driver.start(config(50)).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(70));
    driver.stop().expect("stop should succeed");

    let state = state.lock().unwrap();
    assert!(!state.cpu_writes.is_empty());
    assert_eq!(state.cpu_auto, 1);
    assert_eq!(state.gpu_auto, 1);
}

#[test]
fn threaded_driver_update_changes_later_pwm() {
    let state = Arc::new(Mutex::new(SharedState::default()));
    let hardware_state = state.clone();
    let mut driver = ThreadedFanEngineDriver::new(Box::new(move || {
        Ok(Box::new(FakeHardware {
            state: hardware_state.clone(),
            cpu_temp: 60,
            gpu_temp: 60,
            fail_cpu_auto: false,
            fail_gpu_auto: false,
            panic_on_read: false,
        }))
    }));

    driver.start(config(40)).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(40));
    driver
        .update_config(config(80))
        .expect("update should succeed");
    std::thread::sleep(Duration::from_millis(70));
    driver.stop().expect("stop should succeed");

    let state = state.lock().unwrap();
    assert!(state.cpu_writes.iter().any(|pwm| *pwm >= 200));
}

#[test]
fn stop_reports_auto_restore_failure() {
    let state = Arc::new(Mutex::new(SharedState::default()));
    let hardware_state = state.clone();
    let mut driver = ThreadedFanEngineDriver::new(Box::new(move || {
        Ok(Box::new(FakeHardware {
            state: hardware_state.clone(),
            cpu_temp: 60,
            gpu_temp: 60,
            fail_cpu_auto: true,
            fail_gpu_auto: true,
            panic_on_read: false,
        }))
    }));

    driver.start(config(50)).expect("start should succeed");

    let err = driver
        .stop()
        .expect_err("stop should report restore failure");
    assert!(err.contains("cpu auto restore failed"));
    assert!(err.contains("gpu auto restore failed"));

    let state = state.lock().unwrap();
    assert_eq!(state.cpu_auto, 1);
    assert_eq!(state.gpu_auto, 1);
}

#[test]
fn active_clears_after_worker_panic() {
    let state = Arc::new(Mutex::new(SharedState::default()));
    let hardware_state = state.clone();
    let mut driver = ThreadedFanEngineDriver::new(Box::new(move || {
        Ok(Box::new(FakeHardware {
            state: hardware_state.clone(),
            cpu_temp: 60,
            gpu_temp: 60,
            fail_cpu_auto: false,
            fail_gpu_auto: false,
            panic_on_read: true,
        }))
    }));

    let mut test_config = config(50);
    test_config.tick_ms = 5;
    driver.start(test_config).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(30));

    assert!(!driver.is_active());
    let err = driver.stop().expect_err("stop should report worker panic");
    assert!(err.contains("panicked"));
}
