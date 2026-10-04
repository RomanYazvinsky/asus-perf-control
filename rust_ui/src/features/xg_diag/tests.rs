use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use super::*;

struct FakeCurves {
    default_pwm: Vec<u8>,
    default_temps: Vec<u8>,
    readback: Mutex<Option<Vec<u8>>>,
    written: Mutex<Vec<Vec<i64>>>,
    restores: AtomicUsize,
    fail_writes: bool,
}

impl FakeCurves {
    fn new() -> Self {
        Self::with(vec![20, 43, 66, 81, 117, 193, 193, 193], vec![53, 58, 62, 65, 68, 73, 73, 73])
    }

    fn with(default_pwm: Vec<u8>, default_temps: Vec<u8>) -> Self {
        Self {
            default_pwm,
            default_temps,
            readback: Mutex::new(None),
            written: Mutex::default(),
            restores: AtomicUsize::new(0),
            fail_writes: false,
        }
    }

    fn written(&self) -> Vec<Vec<i64>> {
        self.written.lock().unwrap().clone()
    }
}

#[async_trait]
impl XgFanCurvePort for FakeCurves {
    async fn get_curves(&self, _profile: PerfProfile) -> Result<Vec<FanCurveData>> {
        Ok(vec![
            FanCurveData {
                name: "CPU".into(),
                pwm_values: self.readback.lock().unwrap().clone().unwrap_or(self.default_pwm.clone()),
                temp_values: self.default_temps.clone(),
                enabled: false,
            },
            FanCurveData {
                name: "GPU".into(),
                pwm_values: vec![15, 45, 66, 94, 155, 173, 173, 173],
                temp_values: vec![53, 58, 62, 65, 68, 73, 73, 73],
                enabled: false,
            },
        ])
    }

    async fn set_custom_curve(&self, _profile: PerfProfile, _fan: &str, pwm: &[i64], _temps: Option<&[i64]>) -> Result<()> {
        if self.fail_writes {
            anyhow::bail!("write failed");
        }
        self.written.lock().unwrap().push(pwm.to_vec());
        *self.readback.lock().unwrap() = Some(pwm.iter().map(|v| *v as u8).collect());
        Ok(())
    }

    async fn restore_defaults(&self, _profile: PerfProfile) -> Result<()> {
        self.restores.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct FakePlatform(Mutex<PerfProfile>);

#[async_trait]
impl XgPlatformProfilePort for FakePlatform {
    async fn get_active_profile(&self) -> Result<PerfProfile> {
        Ok(*self.0.lock().unwrap())
    }
    async fn set_active_profile(&self, profile: PerfProfile) -> Result<()> {
        *self.0.lock().unwrap() = profile;
        Ok(())
    }
}

struct FakeProbe {
    snapshots: Vec<XgFanHwmonSnapshot>,
    index: AtomicUsize,
}

impl FakeProbe {
    fn new(snapshots: Vec<XgFanHwmonSnapshot>) -> Self {
        Self {
            snapshots,
            index: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl XgFanHwmonProbe for FakeProbe {
    async fn read_snapshot(&self) -> XgFanHwmonSnapshot {
        let i = self.index.fetch_add(1, Ordering::SeqCst);
        self.snapshots[i.min(self.snapshots.len() - 1)].clone()
    }
}

fn snap(fan1: i64, fan2: i64, pwm2_point8: i64, cpu_milli: i64) -> XgFanHwmonSnapshot {
    XgFanHwmonSnapshot {
        fan1_rpm: Some(fan1),
        fan2_rpm: Some(fan2),
        pwm1_enable: Some(2),
        pwm2_enable: Some(1),
        pwm2_point8: Some(pwm2_point8),
        cpu_temp_milli_c: Some(cpu_milli),
        gpu_temp_milli_c: Some(62000),
        labels: BTreeMap::from([
            ("fan1".to_string(), Some("cpu_fan".to_string())),
            ("fan2".to_string(), Some("gpu_fan".to_string())),
        ]),
    }
}

fn default_snap() -> XgFanHwmonSnapshot {
    snap(3600, 3300, 220, 56000)
}

fn fan2(rpm: i64) -> XgFanHwmonSnapshot {
    snap(3600, rpm, 220, 56000)
}

fn cpu(rpm: i64, milli: i64) -> XgFanHwmonSnapshot {
    snap(rpm, 3300, 220, milli)
}

fn service(
    curves: FakeCurves,
    probe: Vec<XgFanHwmonSnapshot>,
    samples: usize,
) -> XgMobileFanDiagnosticService<FakeCurves, FakePlatform, FakeProbe> {
    XgMobileFanDiagnosticService {
        sample_interval: Duration::ZERO,
        samples_per_pwm: samples,
        ..XgMobileFanDiagnosticService::new(curves, FakePlatform(Mutex::new(PerfProfile::Balanced)), FakeProbe::new(probe))
    }
}

fn run(label: &str, baseline: XgFanHwmonSnapshot, samples: Vec<XgFanHwmonSnapshot>) -> XgFanDiagnosticRun {
    XgFanDiagnosticRun {
        label: label.into(),
        started_at: "2026-05-21T00:00:00.000".into(),
        finished_at: "2026-05-21T00:00:00.000".into(),
        baseline_curves: vec![],
        baseline_snapshot: baseline,
        steps: vec![XgFanSweepStep {
            requested_pwm: 220,
            samples,
        }],
    }
}

fn observation(label: &str, default: &[i64], requested: &[i64], readback: &[i64], temps: &[i64], before: i64, after: i64) -> CurveRuleCaseObservation {
    CurveRuleCaseObservation {
        label: label.into(),
        default_pwm_values: default.to_vec(),
        requested_pwm_values: requested.to_vec(),
        readback_pwm_values: readback.to_vec(),
        temp_values: temps.to_vec(),
        before: cpu(before, 60000),
        samples: vec![cpu(after, 60000)],
    }
}

#[tokio::test]
async fn sweep_restores_balanced_defaults() {
    let mut svc = service(FakeCurves::new(), vec![default_snap(); 8], 2);
    svc.sweep_pwm_values = vec![180, 220];

    let run = svc.run_sweep("current").await.unwrap();

    assert_eq!(svc.curves.written(), vec![vec![180; 8], vec![220; 8]]);
    assert_eq!(svc.curves.restores.load(Ordering::SeqCst), 1);
    assert_eq!(run.steps.len(), 2);
}

#[tokio::test]
async fn sweep_restores_defaults_after_write_failure() {
    let mut curves = FakeCurves::new();
    curves.fail_writes = true;
    let svc = service(curves, vec![default_snap()], 1);

    assert!(svc.run_sweep("current").await.is_err());
    assert_eq!(svc.curves.restores.load(Ordering::SeqCst), 1);
}

#[test]
fn accepted_curve_without_rpm_change() {
    let step = XgFanSweepStep {
        requested_pwm: 220,
        samples: vec![default_snap(), fan2(3400)],
    };

    assert_eq!(step.classification(), XgFanStepClassification::AcceptedNoResponse);
    assert_eq!(step.fan2_min_rpm(), Some(3300));
    assert_eq!(step.fan2_max_rpm(), Some(3400));
    assert_eq!(step.fan2_delta_rpm(), Some(100));
}

#[test]
fn route_changed_when_only_one_run_responds() {
    let no_response = run("connected", default_snap(), vec![default_snap(), fan2(3400)]);
    let responded = run("disconnected", default_snap(), vec![default_snap(), fan2(4300)]);

    assert_eq!(
        XgFanDiagnosis::compare(&no_response, &responded).kind,
        XgFanDiagnosisKind::RouteChanged
    );
}

#[test]
fn baseline_change_means_xg_mobile_dependency() {
    let first = run("connected", fan2(3200), vec![fan2(3200), fan2(3200)]);
    let second = run("disconnected", fan2(4900), vec![fan2(4900), fan2(5000)]);

    assert_eq!(
        XgFanDiagnosis::compare(&first, &second).kind,
        XgFanDiagnosisKind::GpuDependsOnXgMobile
    );
}

#[test]
fn session_json_is_compatible_with_flutter_files() {
    let flutter_json = r#"{
      "currentRun": {
        "label": "Current State",
        "startedAt": "2026-05-21T10:00:00.000Z",
        "finishedAt": "2026-05-21T10:01:00.000Z",
        "baselineCurves": [{"name": "GPU", "pwmValues": [15,45,66,94,155,173,173,173],
                            "tempValues": [53,58,62,65,68,73,73,73], "enabled": false}],
        "baselineSnapshot": {"fan1Rpm": 3600, "fan2Rpm": 3300, "pwm1Enable": 2, "pwm2Enable": 1,
                             "pwm2Point8": 220, "cpuTempMilliC": 56000, "gpuTempMilliC": 62000,
                             "labels": {"fan1": "cpu_fan", "fan2": null}},
        "steps": [{"requestedPwm": 220, "samples": []}]
      },
      "changedRun": null
    }"#;

    let session: XgFanDiagnosticSession = serde_json::from_str(flutter_json).unwrap();
    let run = session.current_run.as_ref().unwrap();
    assert_eq!(run.label, "Current State");
    assert_eq!(run.baseline_curves[0].name, "GPU");
    assert_eq!(run.baseline_snapshot.labels["fan2"], None);
    assert_eq!(run.steps[0].requested_pwm, 220);

    let dir = tempfile::tempdir().unwrap();
    let store = XgFanDiagnosticStore::new(dir.path().join("state").join("diag.json"));
    assert_eq!(store.load().unwrap(), XgFanDiagnosticSession::default());
    store.save(&session).unwrap();
    assert_eq!(store.load().unwrap(), session);
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.path().join("state/diag.json")).unwrap()).unwrap();
    assert_eq!(raw["currentRun"]["baselineSnapshot"]["cpuTempMilliC"], 56000);
}

#[test]
fn gpu_fan_coupled_to_cpu_sweep() {
    let result = FanHypothesisDiagnosis::cpu_gpu_coupling(
        &[FanHypothesisStep::simple("CPU 255".into(), "CPU", 255, vec![fan2(3200), fan2(4300)])],
        &[FanHypothesisStep::simple("GPU 255".into(), "GPU", 255, vec![fan2(3200), fan2(3300)])],
    );

    assert_eq!(result.kind, FanHypothesisKind::GpuDependsOnCpuFan);
}

#[test]
fn profile_floor_or_cap() {
    let step = |rpms: [i64; 2]| FanHypothesisStep::simple("s".into(), "CPU", 40, vec![fan2(rpms[0]), fan2(rpms[1])]);
    let result = FanHypothesisDiagnosis::profile_bounds(&[
        ProfileFanBoundary {
            profile: PerfProfile::Quiet,
            low_pwm_step: step([3000, 3100]),
            high_pwm_step: step([5000, 5200]),
        },
        ProfileFanBoundary {
            profile: PerfProfile::Performance,
            low_pwm_step: step([4300, 4400]),
            high_pwm_step: step([7800, 8200]),
        },
    ]);

    assert_eq!(result.kind, FanHypothesisKind::ProfileFloorOrCap);
}

#[test]
fn interpolates_curve_at_cpu_temperature() {
    let o = CurveRuleCaseObservation {
        label: "low".into(),
        default_pwm_values: vec![100, 200],
        requested_pwm_values: vec![0, 255],
        readback_pwm_values: vec![0, 255],
        temp_values: vec![50, 100],
        before: fan2(3200),
        samples: vec![snap(3600, 3300, 220, 75000)],
    };

    assert_eq!(o.requested_pwm_at_temp().unwrap().round(), 128.0);
    assert_eq!(o.readback_pwm_at_temp().unwrap().round(), 128.0);
    assert_eq!(o.default_pwm_at_temp().unwrap().round(), 150.0);
}

#[test]
fn below_default_runtime_clamped() {
    let result = FanHypothesisDiagnosis::curve_rule_discovery(&[observation(
        "flat_below_default_large_delta",
        &[100, 180],
        &[40, 80],
        &[40, 80],
        &[50, 70],
        6000,
        5900,
    )]);
    assert_eq!(result.kind, FanHypothesisKind::DownwardStoredButRuntimeClamped);
}

#[test]
fn below_default_applied_with_numbers() {
    let result = FanHypothesisDiagnosis::curve_rule_discovery(&[observation(
        "flat_below_default_small_delta",
        &[100, 180],
        &[80, 160],
        &[80, 160],
        &[50, 70],
        6000,
        5400,
    )]);

    assert_eq!(result.kind, FanHypothesisKind::DownwardApplied);
    for needle in ["temp=60C", "PWM default=140", "requested=120", "readback=120", "CPU RPM 6000->5400", "delta=-600"] {
        assert!(result.summary.contains(needle), "{needle} missing in {}", result.summary);
    }
    assert!(result.next_action.contains("drop threshold=500 RPM"));
    assert!(result.next_action.contains("below-default threshold=20 PWM"));
}

#[test]
fn boundary_case_has_priority() {
    let result = FanHypothesisDiagnosis::curve_rule_discovery(&[
        observation("flat_below_default_small_delta", &[100, 180], &[80, 160], &[80, 160], &[50, 70], 6600, 5800),
        observation("zero_until_80c", &[100, 180, 220], &[0, 0, 220], &[0, 0, 220], &[50, 70, 90], 6600, 5200),
    ]);

    assert_eq!(result.kind, FanHypothesisKind::DownwardApplied);
    assert!(result.summary.starts_with("zero_until_80c:"));
}

#[test]
fn below_default_transformed_to_default() {
    let result = FanHypothesisDiagnosis::curve_rule_discovery(&[observation(
        "current_bucket_below_default",
        &[100, 180],
        &[40, 80],
        &[100, 180],
        &[50, 70],
        6000,
        5900,
    )]);
    assert_eq!(result.kind, FanHypothesisKind::DownwardTransformedToDefault);
}

#[tokio::test]
async fn curve_discovery_restores_each_case() {
    let svc = service(FakeCurves::new(), vec![cpu(3600, 66000); 32], 1);

    let result = svc.run_curve_validation_test().await.unwrap();

    let written = svc.curves.written();
    assert_eq!(result.name, "Curve Rule Discovery");
    assert!(written.iter().any(|c| c.iter().all(|p| *p < 117)));
    assert_eq!(svc.curves.restores.load(Ordering::SeqCst), written.len() + 1);
}

#[tokio::test]
async fn curve_discovery_writes_range_cases_up_to_80c() {
    let svc = service(FakeCurves::with(vec![50, 100, 150, 200], vec![40, 60, 80, 90]), vec![cpu(3600, 70000); 48], 1);

    svc.run_curve_validation_test().await.unwrap();

    let written = svc.curves.written();
    for expected in [[0, 0, 0, 200], [255, 255, 255, 200], [0, 20, 70, 200], [130, 180, 230, 200]] {
        assert!(written.contains(&expected.to_vec()), "{expected:?} not written");
    }
}

#[tokio::test]
async fn curve_discovery_exposes_per_case_deltas() {
    let mut probe = vec![cpu(5000, 60000); 4];
    probe.push(cpu(4500, 60000));
    probe.extend(vec![cpu(5000, 60000); 64]);
    let svc = service(FakeCurves::with(vec![50, 100, 150, 200], vec![40, 60, 80, 90]), probe, 1);

    let result = svc.run_curve_validation_test().await.unwrap();
    let step = result
        .steps
        .iter()
        .find(|s| s.label == "flat_below_default_small_delta")
        .unwrap();

    assert_eq!(step.current_temp_c, Some(60));
    assert_eq!(step.default_pwm, Some(100));
    assert_eq!(step.requested_pwm, 80);
    assert_eq!(step.readback_pwm, Some(80));
    assert_eq!(step.cpu_rpm_before, Some(5000));
    assert_eq!(step.cpu_rpm_after, Some(4500));
    assert_eq!(step.cpu_rpm_delta, Some(-500));
}

#[tokio::test]
async fn curve_discovery_restores_after_case_failure() {
    let mut curves = FakeCurves::new();
    curves.fail_writes = true;
    let svc = service(curves, vec![cpu(3600, 66000); 4], 1);

    assert!(svc.run_curve_validation_test().await.is_err());
    assert_eq!(svc.curves.restores.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn profile_bounds_restores_original_profile() {
    let svc = service(FakeCurves::new(), vec![default_snap(); 16], 1);
    *svc.platform.0.lock().unwrap() = PerfProfile::Performance;

    let result = svc.run_profile_bounds_test().await.unwrap();

    assert_eq!(result.steps.len(), 6);
    assert_eq!(*svc.platform.0.lock().unwrap(), PerfProfile::Performance);
    assert_eq!(svc.curves.restores.load(Ordering::SeqCst), 6);
}
