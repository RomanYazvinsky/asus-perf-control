use std::sync::Mutex;

use serde_json::json;

use super::*;
use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

struct Fake {
    running: Mutex<bool>,
    calls: Mutex<Vec<String>>,
    curves: Vec<FanCurveData>,
}

impl Fake {
    fn new() -> Self {
        Self {
            running: Mutex::new(false),
            calls: Mutex::default(),
            curves: vec![
                FanCurveData {
                    name: "CPU".into(),
                    pwm_values: vec![0, 10, 20, 30, 40, 100, 200, 255],
                    temp_values: vec![30, 40, 50, 60, 70, 80, 90, 100],
                    enabled: false,
                },
                FanCurveData {
                    name: "gpu".into(),
                    pwm_values: vec![255; 8],
                    temp_values: vec![30, 40, 50, 60, 70, 80, 90, 100],
                    enabled: true,
                },
            ],
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn call(&self, call: impl Into<String>) {
        self.calls.lock().unwrap().push(call.into());
    }
}

#[async_trait]
impl FanService for Fake {
    async fn capabilities(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![
            FanInfo {
                id: FanId::Cpu,
                label: "CPU Fan".into(),
            },
            FanInfo {
                id: FanId::Gpu,
                label: "GPU Fan".into(),
            },
        ])
    }
    async fn fan_rpms(&self) -> Result<Vec<(FanId, Option<u32>)>> {
        Ok(vec![(FanId::Cpu, Some(2400)), (FanId::Gpu, None)])
    }
    async fn temperatures(&self) -> Result<FanTemperatures> {
        Ok(FanTemperatures {
            cpu_temp_c: 55,
            gpu_temp_c: 48,
        })
    }
    async fn active_profile(&self) -> Result<PerfProfile> {
        Ok(PerfProfile::Balanced)
    }
    async fn asusd_curves(&self, _profile: PerfProfile) -> Result<Vec<FanCurveData>> {
        Ok(self.curves.clone())
    }
    async fn set_asusd_curve(&self, profile: PerfProfile, fan: FanId, _points: &[FanCurvePoint]) -> Result<()> {
        self.call(format!("curve {} {}", profile.name(), fan.name()));
        Ok(())
    }
    async fn set_constant_speed(&self, _profile: PerfProfile, fan: FanId, speed_percent: i64) -> Result<()> {
        self.call(format!("constant {} {speed_percent}", fan.name()));
        Ok(())
    }
    async fn reset_to_defaults(&self, profile: PerfProfile) -> Result<()> {
        self.call(format!("reset {}", profile.name()));
        Ok(())
    }
    async fn start_dynamic_engine(&self, _config: &FanEngineConfig) -> Result<()> {
        *self.running.lock().unwrap() = true;
        self.call("start");
        Ok(())
    }
    async fn stop_dynamic_engine(&self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        self.call("stop");
        Ok(())
    }
    async fn update_dynamic_curve(&self, fan: FanId, _curve: &[FanCurvePoint]) -> Result<()> {
        self.call(format!("update {}", fan.name()));
        Ok(())
    }
    async fn is_dynamic_engine_running(&self) -> Result<bool> {
        Ok(*self.running.lock().unwrap())
    }
    async fn run_max_speed_test(&self, _profile: PerfProfile, _fan: FanId) -> Result<u32> {
        Ok(6100)
    }
}

fn store(fan: Option<FanDesiredSettings>) -> MemorySettingsStore {
    MemorySettingsStore::new(AppSettings {
        hardware: HardwareSettings {
            fan,
            ..Default::default()
        },
        ..Default::default()
    })
}

async fn loaded(service: &Fake, store: &MemorySettingsStore) -> FanLoaded {
    match load(service, store).await {
        FanState::Loaded(loaded) => loaded,
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn loads_fans_curves_and_saves_defaults() {
    let service = Fake::new();
    let store = store(None);

    let state = loaded(&service, &store).await;

    assert_eq!(state.fan_ids(), vec![FanId::Cpu, FanId::Gpu]);
    assert_eq!(state.fan_rpms[&FanId::Cpu], 2400);
    assert_eq!(state.fan_rpms[&FanId::Gpu], 0);
    assert_eq!(state.cpu_temp_c, 55);
    assert_eq!(state.selected_fan, FanId::Cpu);
    assert_eq!(state.selected_profile, PerfProfile::Balanced);
    assert_eq!(state.control_mode(FanId::Gpu), FanControlMode::AsusctlCurve);
    assert!(state.asusd_curves[&FanId::Gpu].contains_key(&PerfProfile::Quiet));
    let saved = store.load().hardware.fan.unwrap();
    assert_eq!(saved.selected_fan.as_deref(), Some("cpu"));
    assert_eq!(saved.control_modes["gpu"], "asusctlCurve");
    assert!(service.calls().is_empty());
}

#[tokio::test]
async fn restarts_saved_dynamic_engine_without_switching_profile() {
    let service = Fake::new();
    let config = FanEngineConfig::new(PerfProfile::Quiet, default_fan_curve(), default_fan_curve());
    let store = store(Some(FanDesiredSettings {
        selected_profile: Some("quiet".into()),
        selected_fan: Some("gpu".into()),
        control_modes: [("cpu".to_string(), "dynamicEngine".to_string())].into(),
        dynamic_engine_config: Some(config.to_json()),
    }));

    let state = loaded(&service, &store).await;

    assert_eq!(service.calls(), vec!["start"]);
    assert_eq!(state.selected_profile, PerfProfile::Quiet);
    assert_eq!(state.active_profile, Some(PerfProfile::Balanced));
    assert!(state.dynamic_engine_running);
    assert_eq!(state.selected_fan, FanId::Gpu);
    assert_eq!(state.engine_config.map(|c| c.to_json()), Some(config.to_json()));
}

#[tokio::test]
async fn ignores_invalid_saved_values() {
    let service = Fake::new();
    let store = store(Some(FanDesiredSettings {
        selected_profile: Some("ludicrous".into()),
        selected_fan: Some("pump".into()),
        control_modes: [("cpu".to_string(), "magic".to_string())].into(),
        dynamic_engine_config: None,
    }));

    let state = loaded(&service, &store).await;

    assert_eq!(state.selected_fan, FanId::Cpu);
    assert_eq!(state.selected_profile, PerfProfile::Balanced);
    assert_eq!(state.control_mode(FanId::Cpu), FanControlMode::AsusctlCurve);
}

#[tokio::test]
async fn refresh_drops_dynamic_mode_when_engine_stopped() {
    let service = Fake::new();
    let store = store(None);
    let state = loaded(&service, &store).await;
    let FanState::Loaded(mut state) = start_dynamic_engine(&service, &store, state.clone(), state.build_engine_config()).await
    else {
        panic!()
    };
    assert_eq!(state.control_mode(FanId::Cpu), FanControlMode::DynamicEngine);

    *service.running.lock().unwrap() = false;
    apply_refresh(&mut state, refresh(&service).await.unwrap());

    assert_eq!(state.control_mode(FanId::Cpu), FanControlMode::AsusctlCurve);
    assert!(state.engine_config.is_none());
}

#[tokio::test]
async fn curve_and_constant_speed_update_modes_and_curves() {
    let service = Fake::new();
    let store = store(None);
    let state = loaded(&service, &store).await;

    let FanState::Loaded(state) =
        set_constant_speed(&service, &store, state, FanId::Gpu, PerfProfile::Balanced, 50).await
    else {
        panic!()
    };
    assert_eq!(state.control_mode(FanId::Gpu), FanControlMode::ConstantSpeed);
    assert_eq!(state.asusd_curves[&FanId::Gpu][&PerfProfile::Balanced].pwm_values, vec![128; 8]);

    let curve = default_fan_curve();
    let FanState::Loaded(state) =
        set_asusctl_curve(&service, &store, state, FanId::Gpu, PerfProfile::Balanced, curve.clone()).await
    else {
        panic!()
    };
    assert_eq!(state.control_mode(FanId::Gpu), FanControlMode::AsusctlCurve);
    assert_eq!(state.editor_curve(FanId::Gpu).len(), 8);
    assert_eq!(service.calls(), vec!["constant gpu 50", "curve balanced gpu"]);
}

#[tokio::test]
async fn update_dynamic_curve_builds_config_when_missing() {
    let service = Fake::new();
    let store = store(None);
    let state = loaded(&service, &store).await;
    let curve: Vec<FanCurvePoint> = (0..8).map(|i| FanCurvePoint::new(30.0 + i as f64 * 5.0, 50.0)).collect();

    let FanState::Loaded(state) = update_dynamic_curve(&service, &store, state, FanId::Cpu, curve.clone()).await else {
        panic!()
    };

    let config = state.engine_config.unwrap();
    assert_eq!(config.cpu_curve, curve);
    assert_eq!(config.gpu_curve, with_safety_clamp(&config.gpu_curve));
    assert!(store.load().hardware.fan.unwrap().dynamic_engine_config.is_some());
}

#[test]
fn safety_clamp_raises_hot_points() {
    let curve = vec![FanCurvePoint::new(70.0, 10.0), FanCurvePoint::new(85.0, 10.0)];
    let clamped = with_safety_clamp(&curve);
    assert_eq!(clamped[0].speed_percent, 10.0);
    assert_eq!(clamped[1].speed_percent, 30.0);
}

#[test]
fn engine_config_json_roundtrip_matches_daemon_contract() {
    let config = FanEngineConfig::new(PerfProfile::Performance, default_fan_curve(), default_fan_curve());
    let json = config.to_json();

    assert_eq!(json["profile"], json!("performance"));
    assert_eq!(json["tickMs"], json!(1000));
    assert_eq!(json["cpuCurve"][0], json!([20, 30]));
    assert_eq!(json["cpuCurve"][7], json!([90, 100]));
    assert_eq!(json["crossFanCurve"], json!(null));
    let parsed = FanEngineConfig::from_json(&json);
    assert_eq!(parsed.profile, PerfProfile::Performance);
    assert_eq!(parsed.cpu_curve[1], FanCurvePoint::new(30.0, 40.0));
}

#[test]
fn invalid_curve_json_falls_back_to_default() {
    assert_eq!(curve_from_json(Some(&json!([[1, 2]]))), default_fan_curve());
    assert_eq!(curve_from_json(None), default_fan_curve());
}

#[test]
fn normalizes_temperatures_to_strictly_increasing() {
    let curve: Vec<FanCurvePoint> = [30.2, 30.4, 29.0, 50.0].iter().map(|t| FanCurvePoint::new(*t, 50.0)).collect();
    assert_eq!(normalized_rounded_temps(&curve), vec![30, 31, 32, 50]);
}

#[test]
fn converts_between_pwm_and_percent() {
    let data = FanCurveData::from_curve_points("CPU", &default_fan_curve(), true);
    assert_eq!(data.pwm_values[0], 77);
    assert_eq!(data.pwm_values[7], 255);
    assert_eq!(data.to_curve_points()[7], FanCurvePoint::new(90.0, 100.0));
    assert_eq!(FanCurveData::flat("GPU", 100).pwm_values, vec![255; 8]);
}

#[test]
fn point_updates_keep_curve_ordered_and_monotonic() {
    let mut points = default_fan_curve();

    update_point_position(&mut points, 3, 10.0, 90.0);
    assert_eq!(points[3].temp_c, 41.0);
    assert_eq!(points[3].speed_percent, 90.0);
    assert!(points[4..].iter().all(|p| p.speed_percent >= 90.0));

    update_point_position(&mut points, 5, 200.0, 5.0);
    assert_eq!(points[5].temp_c, 79.0);
    assert!(points[..5].iter().all(|p| p.speed_percent <= 5.0));
    assert!(points[6].speed_percent >= 90.0);
}

#[tokio::test]
async fn select_profile_only_changes_edited_profile() {
    let service = Fake::new();
    let store = store(None);
    let state = loaded(&service, &store).await;

    let FanState::Loaded(next) = select_profile(&store, state, PerfProfile::Performance) else {
        panic!()
    };

    assert_eq!(next.selected_profile, PerfProfile::Performance);
    assert_eq!(next.active_profile, Some(PerfProfile::Balanced));
    assert!(service.calls().is_empty());
    assert_eq!(store.load().hardware.fan.unwrap().selected_profile.as_deref(), Some("performance"));
}

#[tokio::test]
async fn apply_curves_sends_every_edited_fan_to_the_selected_profile() {
    let service = Fake::new();
    let store = store(None);
    let state = loaded(&service, &store).await;
    let state = match select_profile(&store, state, PerfProfile::Quiet) {
        FanState::Loaded(state) => state,
        other => panic!("{other:?}"),
    };

    let result = apply_curves(
        &service,
        &store,
        state,
        vec![(FanId::Cpu, default_fan_curve()), (FanId::Gpu, default_fan_curve())],
    )
    .await;

    assert!(matches!(result, FanState::Loaded(_)));
    assert_eq!(service.calls(), vec!["curve quiet cpu", "curve quiet gpu"]);
}
