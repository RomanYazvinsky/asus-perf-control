#![allow(dead_code)]

#[path = "../src/fan_engine/mod.rs"]
mod fan_engine;

use fan_engine::config::{EngineCurvePoint, FanEngineConfig};
use fan_engine::controller::{FanEngineController, FanEngineDriver};

#[derive(Default)]
struct FakeDriver {
    active: bool,
    starts: usize,
    stops: usize,
    updates: usize,
}

impl FanEngineDriver for FakeDriver {
    fn start(
        &mut self,
        _config: fan_engine::config::ValidatedFanEngineConfig,
    ) -> Result<(), String> {
        self.active = true;
        self.starts += 1;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.active = false;
        self.stops += 1;
        Ok(())
    }

    fn update_config(
        &mut self,
        _config: fan_engine::config::ValidatedFanEngineConfig,
    ) -> Result<(), String> {
        self.updates += 1;
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.active
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

fn valid_config() -> FanEngineConfig {
    FanEngineConfig {
        profile: "balanced".to_string(),
        tick_ms: 1000,
        hysteresis_c: 2,
        ramp_down_delay_ms: 3000,
        emergency_temp_c: 95,
        cpu_curve: curve(),
        gpu_curve: curve(),
        cross_fan: false,
        cross_fan_curve: None,
    }
}

#[test]
fn start_sets_running_state() {
    let mut controller = FanEngineController::new_with_driver(Box::new(FakeDriver::default()));

    controller
        .start(valid_config())
        .expect("start should succeed");

    assert!(controller.is_running());
}

#[test]
fn second_start_is_idempotent() {
    let mut controller = FanEngineController::new_with_driver(Box::new(FakeDriver::default()));

    controller
        .start(valid_config())
        .expect("first start should succeed");
    controller
        .start(valid_config())
        .expect("second start should be idempotent");

    assert!(controller.is_running());
}

#[test]
fn stop_resets_running_state() {
    let mut controller = FanEngineController::new_with_driver(Box::new(FakeDriver::default()));

    controller
        .start(valid_config())
        .expect("start should succeed");
    controller.stop().expect("stop should succeed");

    assert!(!controller.is_running());
}

#[test]
fn update_curve_requires_running_engine() {
    let mut controller = FanEngineController::new();

    let result = controller.update_curve("cpu".to_string(), curve());

    assert!(result.is_err());
}

#[test]
fn default_controller_does_not_report_running_when_runtime_is_unavailable() {
    let mut controller = FanEngineController::new();

    let result = controller.start(valid_config());

    assert!(result.is_err());
    assert!(!controller.is_running());
}

#[test]
fn update_curve_updates_running_driver_config() {
    let mut controller = FanEngineController::new_with_driver(Box::new(FakeDriver::default()));

    controller
        .start(valid_config())
        .expect("start should succeed");
    controller
        .update_curve("cpu".to_string(), curve())
        .expect("update should succeed");

    assert!(controller.is_running());
}

#[test]
fn current_config_reflects_updated_running_curve() {
    let mut controller = FanEngineController::new_with_driver(Box::new(FakeDriver::default()));
    let mut updated_curve = curve();
    updated_curve[0] = point(30, 25);

    controller
        .start(valid_config())
        .expect("start should succeed");
    controller
        .update_curve("cpu".to_string(), updated_curve)
        .expect("update should succeed");

    let config = controller
        .current_config()
        .expect("running controller should expose config");
    assert_eq!(config.cpu_curve[0].speed_percent, 25);
    assert_eq!(config.gpu_curve[0].speed_percent, 20);
}

#[test]
fn is_running_is_false_when_driver_is_inactive() {
    struct InactiveAfterStartDriver;

    impl FanEngineDriver for InactiveAfterStartDriver {
        fn start(
            &mut self,
            _config: fan_engine::config::ValidatedFanEngineConfig,
        ) -> Result<(), String> {
            Ok(())
        }

        fn stop(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn update_config(
            &mut self,
            _config: fan_engine::config::ValidatedFanEngineConfig,
        ) -> Result<(), String> {
            Ok(())
        }

        fn is_active(&self) -> bool {
            false
        }
    }

    let mut controller = FanEngineController::new_with_driver(Box::new(InactiveAfterStartDriver));
    controller
        .start(valid_config())
        .expect("start should succeed");

    assert!(!controller.is_running());
}
