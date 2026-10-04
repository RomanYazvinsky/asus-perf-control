//! XG Mobile fan diagnostic — sweeps GPU/CPU fan PWM through asusd and samples hwmon.
//! Every test restores the default curves when it finishes, also on error.

pub mod model;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;

pub use model::*;

use crate::features::fan::model::{FanCurveData, PerfProfile};

const DEFAULT_TEMPS: [u8; 8] = [30, 40, 50, 60, 70, 80, 90, 100];

#[async_trait]
pub trait XgFanCurvePort: Send + Sync {
    async fn get_curves(&self, profile: PerfProfile) -> Result<Vec<FanCurveData>>;
    /// Writes and enables a custom curve.
    async fn set_custom_curve(&self, profile: PerfProfile, fan: &str, pwm: &[i64], temps: Option<&[i64]>) -> Result<()>;
    async fn restore_defaults(&self, profile: PerfProfile) -> Result<()>;
}

#[async_trait]
pub trait XgPlatformProfilePort: Send + Sync {
    async fn get_active_profile(&self) -> Result<PerfProfile>;
    async fn set_active_profile(&self, profile: PerfProfile) -> Result<()>;
}

#[async_trait]
pub trait XgFanHwmonProbe: Send + Sync {
    async fn read_snapshot(&self) -> XgFanHwmonSnapshot;
}

/// Builds the asusd curve for a diagnostic write; values are clamped to a byte.
pub fn custom_curve(fan: &str, pwm: &[i64], temps: Option<&[i64]>) -> FanCurveData {
    let to_byte = |v: &i64| (*v).clamp(0, 255) as u8;
    FanCurveData {
        name: fan.to_string(),
        pwm_values: pwm.iter().map(to_byte).collect(),
        temp_values: temps.map_or_else(|| DEFAULT_TEMPS.to_vec(), |t| t.iter().map(to_byte).collect()),
        enabled: true,
    }
}

/// Reads the fixed hwmon paths used on the GV301 with XG Mobile.
pub struct SysfsXgFanHwmonProbe;

async fn read_string(path: &str) -> Option<String> {
    tokio::fs::read_to_string(path).await.ok().map(|s| s.trim().to_string())
}

async fn read_int(path: &str) -> Option<i64> {
    read_string(path).await?.parse().ok()
}

#[async_trait]
impl XgFanHwmonProbe for SysfsXgFanHwmonProbe {
    async fn read_snapshot(&self) -> XgFanHwmonSnapshot {
        XgFanHwmonSnapshot {
            fan1_rpm: read_int("/sys/class/hwmon/hwmon7/fan1_input").await,
            fan2_rpm: read_int("/sys/class/hwmon/hwmon7/fan2_input").await,
            pwm1_enable: read_int("/sys/class/hwmon/hwmon8/pwm1_enable").await,
            pwm2_enable: read_int("/sys/class/hwmon/hwmon8/pwm2_enable").await,
            pwm2_point8: read_int("/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm").await,
            cpu_temp_milli_c: read_int("/sys/class/hwmon/hwmon5/temp1_input").await,
            gpu_temp_milli_c: read_int("/sys/class/hwmon/hwmon6/temp1_input").await,
            labels: BTreeMap::from([
                ("fan1".to_string(), read_string("/sys/class/hwmon/hwmon7/fan1_label").await),
                ("fan2".to_string(), read_string("/sys/class/hwmon/hwmon7/fan2_label").await),
            ]),
        }
    }
}

pub struct XgMobileFanDiagnosticService<C, P, H> {
    pub curves: C,
    pub platform: P,
    pub probe: H,
    pub sample_interval: Duration,
    pub samples_per_pwm: usize,
    pub sweep_pwm_values: Vec<i64>,
}

fn now() -> String {
    chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

fn find_curve(curves: &[FanCurveData], name: &str) -> FanCurveData {
    curves.iter().find(|c| c.name == name).cloned().unwrap_or(FanCurveData {
        name: name.to_string(),
        pwm_values: Vec::new(),
        temp_values: Vec::new(),
        enabled: false,
    })
}

fn bytes_to_i64(values: &[u8]) -> Vec<i64> {
    values.iter().map(|v| i64::from(*v)).collect()
}

/// Run `body`, then always `cleanup`; the body's error wins over the cleanup's.
async fn with_cleanup<T>(
    body: impl std::future::Future<Output = Result<T>>,
    cleanup: impl std::future::Future<Output = Result<()>>,
) -> Result<T> {
    let result = body.await;
    let cleaned = cleanup.await;
    let value = result?;
    cleaned?;
    Ok(value)
}

impl<C: XgFanCurvePort, P: XgPlatformProfilePort, H: XgFanHwmonProbe> XgMobileFanDiagnosticService<C, P, H> {
    pub fn new(curves: C, platform: P, probe: H) -> Self {
        Self {
            curves,
            platform,
            probe,
            sample_interval: Duration::from_secs(1),
            samples_per_pwm: 15,
            sweep_pwm_values: vec![180, 220, 255],
        }
    }

    async fn collect_samples(&self) -> Vec<XgFanHwmonSnapshot> {
        let mut samples = Vec::with_capacity(self.samples_per_pwm);
        for i in 0..self.samples_per_pwm {
            samples.push(self.probe.read_snapshot().await);
            if !self.sample_interval.is_zero() && i + 1 < self.samples_per_pwm {
                tokio::time::sleep(self.sample_interval).await;
            }
        }
        samples
    }

    async fn set_flat(&self, profile: PerfProfile, fan: &str, pwm: i64) -> Result<()> {
        self.curves.set_custom_curve(profile, fan, &[pwm.clamp(0, 255); 8], None).await
    }

    async fn restore_balanced(&self) -> Result<()> {
        self.curves.restore_defaults(PerfProfile::Balanced).await
    }

    /// GPU flat-PWM sweep on the balanced profile.
    pub async fn run_sweep(&self, label: &str) -> Result<XgFanDiagnosticRun> {
        let started_at = now();
        let baseline_curves = self.curves.get_curves(PerfProfile::Balanced).await?;
        let baseline_snapshot = self.probe.read_snapshot().await;
        let steps = with_cleanup(
            async {
                let mut steps = Vec::new();
                for &pwm in &self.sweep_pwm_values {
                    self.set_flat(PerfProfile::Balanced, "GPU", pwm).await?;
                    steps.push(XgFanSweepStep {
                        requested_pwm: pwm,
                        samples: self.collect_samples().await,
                    });
                }
                Ok(steps)
            },
            self.restore_balanced(),
        )
        .await?;
        Ok(XgFanDiagnosticRun {
            label: label.to_string(),
            started_at,
            finished_at: now(),
            baseline_curves: baseline_curves
                .iter()
                .map(|c| CurveJson {
                    name: c.name.clone(),
                    pwm_values: bytes_to_i64(&c.pwm_values),
                    temp_values: bytes_to_i64(&c.temp_values),
                    enabled: c.enabled,
                })
                .collect(),
            baseline_snapshot,
            steps,
        })
    }

    pub async fn run_cpu_gpu_coupling_test(&self) -> Result<FanHypothesisResult> {
        let (cpu_steps, gpu_steps) = with_cleanup(
            async {
                let mut cpu_steps = Vec::new();
                for &pwm in &self.sweep_pwm_values {
                    self.set_flat(PerfProfile::Balanced, "CPU", pwm).await?;
                    cpu_steps.push(FanHypothesisStep::simple(format!("CPU {pwm}"), "CPU", pwm, self.collect_samples().await));
                }
                self.restore_balanced().await?;
                let mut gpu_steps = Vec::new();
                for &pwm in &self.sweep_pwm_values {
                    self.set_flat(PerfProfile::Balanced, "GPU", pwm).await?;
                    gpu_steps.push(FanHypothesisStep::simple(format!("GPU {pwm}"), "GPU", pwm, self.collect_samples().await));
                }
                Ok((cpu_steps, gpu_steps))
            },
            self.restore_balanced(),
        )
        .await?;
        Ok(FanHypothesisResult {
            name: "CPU/GPU Coupling".into(),
            diagnosis: FanHypothesisDiagnosis::cpu_gpu_coupling(&cpu_steps, &gpu_steps),
            steps: cpu_steps.into_iter().chain(gpu_steps).collect(),
        })
    }

    pub async fn run_profile_bounds_test(&self) -> Result<FanHypothesisResult> {
        let original = self.platform.get_active_profile().await?;
        let profiles = [PerfProfile::Quiet, PerfProfile::Balanced, PerfProfile::Performance];
        let boundaries = with_cleanup(
            async {
                let mut boundaries = Vec::new();
                for profile in profiles {
                    self.platform.set_active_profile(profile).await?;
                    self.set_flat(profile, "CPU", 40).await?;
                    let low = FanHypothesisStep::simple(
                        format!("{} CPU 40", profile.name()),
                        "CPU",
                        40,
                        self.collect_samples().await,
                    );
                    self.set_flat(profile, "CPU", 255).await?;
                    let high = FanHypothesisStep::simple(
                        format!("{} CPU 255", profile.name()),
                        "CPU",
                        255,
                        self.collect_samples().await,
                    );
                    boundaries.push(ProfileFanBoundary {
                        profile,
                        low_pwm_step: low,
                        high_pwm_step: high,
                    });
                    self.curves.restore_defaults(profile).await?;
                }
                Ok(boundaries)
            },
            async {
                for profile in profiles {
                    self.curves.restore_defaults(profile).await?;
                }
                self.platform.set_active_profile(original).await
            },
        )
        .await?;
        Ok(FanHypothesisResult {
            name: "Profile Bounds".into(),
            diagnosis: FanHypothesisDiagnosis::profile_bounds(&boundaries),
            steps: boundaries
                .into_iter()
                .flat_map(|b| [b.low_pwm_step, b.high_pwm_step])
                .collect(),
        })
    }

    pub async fn run_curve_validation_test(&self) -> Result<FanHypothesisResult> {
        let default_curve = find_curve(&self.curves.get_curves(PerfProfile::Balanced).await?, "CPU");
        let temps = bytes_to_i64(&default_curve.temp_values);
        let default_pwm = bytes_to_i64(&default_curve.pwm_values);
        let baseline = self.probe.read_snapshot().await;
        let cases = build_curve_rule_cases(&default_pwm, &temps, baseline.cpu_temp_milli_c);

        let observations = with_cleanup(
            async {
                let mut observations = Vec::new();
                for (label, requested) in cases {
                    let observation = with_cleanup(
                        async {
                            let before = self.probe.read_snapshot().await;
                            self.curves
                                .set_custom_curve(PerfProfile::Balanced, "CPU", &requested, Some(&temps))
                                .await?;
                            let readback = find_curve(&self.curves.get_curves(PerfProfile::Balanced).await?, "CPU");
                            Ok(CurveRuleCaseObservation {
                                label: label.to_string(),
                                default_pwm_values: default_pwm.clone(),
                                requested_pwm_values: requested.clone(),
                                readback_pwm_values: bytes_to_i64(&readback.pwm_values),
                                temp_values: temps.clone(),
                                before,
                                samples: self.collect_samples().await,
                            })
                        },
                        self.restore_balanced(),
                    )
                    .await?;
                    observations.push(observation);
                }
                Ok(observations)
            },
            self.restore_balanced(),
        )
        .await?;

        Ok(FanHypothesisResult {
            name: "Curve Rule Discovery".into(),
            diagnosis: FanHypothesisDiagnosis::curve_rule_discovery(&observations),
            steps: observations.iter().map(CurveRuleCaseObservation::to_step).collect(),
        })
    }
}

fn nearest_curve_point_index(temps: &[i64], cpu_temp_milli_c: Option<i64>) -> usize {
    let (Some(first), Some(milli)) = (temps.first(), cpu_temp_milli_c) else {
        return 0;
    };
    let temp_c = (milli as f64 / 1000.0).round() as i64;
    let mut best = 0;
    let mut best_delta = (first - temp_c).abs();
    for (i, t) in temps.iter().enumerate().skip(1) {
        let delta = (t - temp_c).abs();
        if delta < best_delta {
            best = i;
            best_delta = delta;
        }
    }
    best
}

/// The 13 curve shapes probed by "Curve Rule Discovery".
pub fn build_curve_rule_cases(default_pwm: &[i64], temps: &[i64], cpu_temp_milli_c: Option<i64>) -> Vec<(&'static str, Vec<i64>)> {
    let shift_all = |delta: i64| default_pwm.iter().map(|v| (v + delta).clamp(0, 255)).collect::<Vec<_>>();
    let shift_current_bucket = |delta: i64| {
        let mut values = default_pwm.to_vec();
        let index = nearest_curve_point_index(temps, cpu_temp_milli_c) as i64;
        let mut points = vec![index - 1, index];
        points.dedup();
        for point in points {
            if point >= 0 && (point as usize) < values.len() {
                values[point as usize] = (values[point as usize] + delta).clamp(0, 255);
            }
        }
        values
    };
    let until_temp = |max_temp: i64, transform: &dyn Fn(i64) -> i64| {
        let mut values = default_pwm.to_vec();
        for i in 0..values.len().min(temps.len()) {
            if temps[i] <= max_temp {
                values[i] = transform(values[i]).clamp(0, 255);
            }
        }
        values
    };
    let mut non_monotonic = default_pwm.to_vec();
    if non_monotonic.len() >= 4 {
        non_monotonic[1] = 220;
        non_monotonic[2] = 60;
        non_monotonic[3] = 200;
    }
    vec![
        ("default", default_pwm.to_vec()),
        ("flat_below_default_small_delta", shift_all(-20)),
        ("flat_below_default_large_delta", shift_all(-80)),
        ("current_bucket_below_default", shift_current_bucket(-80)),
        ("current_bucket_above_default", shift_current_bucket(40)),
        ("all_above_default", shift_all(40)),
        ("zero_until_80c", until_temp(80, &|_| 0)),
        ("max_until_80c", until_temp(80, &|_| 255)),
        ("below_default_until_80c", until_temp(80, &|v| v - 80)),
        ("above_default_until_80c", until_temp(80, &|v| v + 80)),
        ("flat_low", vec![40; default_pwm.len()]),
        ("flat_high", vec![255; default_pwm.len()]),
        ("non_monotonic_pwm", non_monotonic),
    ]
}

// ----------------------------------------------------------------------------
// Session store
// ----------------------------------------------------------------------------

pub struct XgFanDiagnosticStore {
    file: PathBuf,
}

impl XgFanDiagnosticStore {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    /// `$XDG_STATE_HOME/asus-linux-control/xg_mobile_fan_diagnostic.json`.
    pub fn default_file() -> PathBuf {
        let state_home = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
            format!("{}/.local/state", std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        });
        PathBuf::from(state_home)
            .join("asus-linux-control")
            .join("xg_mobile_fan_diagnostic.json")
    }

    pub fn load(&self) -> Result<XgFanDiagnosticSession> {
        if !self.file.exists() {
            return Ok(XgFanDiagnosticSession::default());
        }
        let text = std::fs::read_to_string(&self.file).with_context(|| format!("read {}", self.file.display()))?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, session: &XgFanDiagnosticSession) -> Result<()> {
        if let Some(parent) = self.file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.file, serde_json::to_string_pretty(session)?)
            .with_context(|| format!("write {}", self.file.display()))
    }
}

#[cfg(test)]
mod tests;
