//! XG Mobile fan diagnostic data: hwmon snapshots, sweep runs and persisted sessions.
//! JSON field names match the former Flutter session file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::features::fan::model::PerfProfile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XgFanHwmonSnapshot {
    pub fan1_rpm: Option<i64>,
    pub fan2_rpm: Option<i64>,
    pub pwm1_enable: Option<i64>,
    pub pwm2_enable: Option<i64>,
    pub pwm2_point8: Option<i64>,
    #[serde(rename = "cpuTempMilliC")]
    pub cpu_temp_milli_c: Option<i64>,
    #[serde(rename = "gpuTempMilliC")]
    pub gpu_temp_milli_c: Option<i64>,
    pub labels: BTreeMap<String, Option<String>>,
}

fn min_max_last(values: &[i64]) -> (Option<i64>, Option<i64>, Option<i64>) {
    (
        values.iter().copied().min(),
        values.iter().copied().max(),
        values.last().copied(),
    )
}

fn fan2_values(samples: &[XgFanHwmonSnapshot]) -> Vec<i64> {
    samples.iter().filter_map(|s| s.fan2_rpm).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XgFanStepClassification {
    Responded,
    AcceptedNoResponse,
    Inconclusive,
}

impl XgFanStepClassification {
    pub fn label(self) -> &'static str {
        match self {
            Self::Responded => "Responded",
            Self::AcceptedNoResponse => "No response",
            Self::Inconclusive => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XgFanSweepStep {
    pub requested_pwm: i64,
    pub samples: Vec<XgFanHwmonSnapshot>,
}

impl XgFanSweepStep {
    pub fn fan2_min_rpm(&self) -> Option<i64> {
        min_max_last(&fan2_values(&self.samples)).0
    }

    pub fn fan2_max_rpm(&self) -> Option<i64> {
        min_max_last(&fan2_values(&self.samples)).1
    }

    pub fn fan2_last_rpm(&self) -> Option<i64> {
        min_max_last(&fan2_values(&self.samples)).2
    }

    pub fn fan2_delta_rpm(&self) -> Option<i64> {
        Some(self.fan2_max_rpm()? - self.fan2_min_rpm()?)
    }

    /// Driver kept pwm2 enabled and stored the requested value for every sample.
    pub fn accepted_by_driver(&self) -> bool {
        !self.samples.is_empty()
            && self
                .samples
                .iter()
                .all(|s| s.pwm2_enable == Some(1) && s.pwm2_point8 == Some(self.requested_pwm))
    }

    pub fn classification(&self) -> XgFanStepClassification {
        match self.fan2_delta_rpm() {
            None => XgFanStepClassification::Inconclusive,
            Some(delta) if delta >= 500 => XgFanStepClassification::Responded,
            Some(_) if self.accepted_by_driver() => XgFanStepClassification::AcceptedNoResponse,
            Some(_) => XgFanStepClassification::Inconclusive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurveJson {
    pub name: String,
    pub pwm_values: Vec<i64>,
    pub temp_values: Vec<i64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XgFanDiagnosticRun {
    pub label: String,
    /// ISO-8601 local timestamps, kept as text.
    pub started_at: String,
    pub finished_at: String,
    pub baseline_curves: Vec<CurveJson>,
    pub baseline_snapshot: XgFanHwmonSnapshot,
    pub steps: Vec<XgFanSweepStep>,
}

impl XgFanDiagnosticRun {
    pub fn has_fan_response(&self) -> bool {
        self.steps
            .iter()
            .any(|s| s.classification() == XgFanStepClassification::Responded)
    }

    pub fn has_missing_fan2_rpm(&self) -> bool {
        self.baseline_snapshot.fan2_rpm.is_none() || self.steps.iter().any(|s| s.fan2_last_rpm().is_none())
    }

    pub fn has_different_labels_from(&self, other: &Self) -> bool {
        self.baseline_snapshot.labels != other.baseline_snapshot.labels
    }

    pub fn fan2_baseline_delta_from(&self, other: &Self) -> Option<i64> {
        Some((self.baseline_snapshot.fan2_rpm? - other.baseline_snapshot.fan2_rpm?).abs())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XgFanDiagnosticSession {
    pub current_run: Option<XgFanDiagnosticRun>,
    pub changed_run: Option<XgFanDiagnosticRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XgFanDiagnosisKind {
    Responded,
    AcceptedNoResponse,
    RouteChanged,
    GpuDependsOnXgMobile,
    NewHwmonLikely,
    Inconclusive,
}

impl XgFanDiagnosisKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Responded => "Fan responded",
            Self::AcceptedNoResponse => "Accepted, no RPM response",
            Self::RouteChanged => "XG Mobile changed fan routing",
            Self::GpuDependsOnXgMobile => "GPU fan depends on XG Mobile state",
            Self::NewHwmonLikely => "New hwmon path likely",
            Self::Inconclusive => "Inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XgFanDiagnosis {
    pub kind: XgFanDiagnosisKind,
    pub summary: &'static str,
    pub next_action: &'static str,
}

impl XgFanDiagnosis {
    const fn new(kind: XgFanDiagnosisKind, summary: &'static str, next_action: &'static str) -> Self {
        Self {
            kind,
            summary,
            next_action,
        }
    }

    pub fn single(run: &XgFanDiagnosticRun) -> Self {
        use XgFanDiagnosisKind as K;
        if run.has_missing_fan2_rpm() {
            return Self::new(
                K::Inconclusive,
                "fan2 RPM was not available for this run.",
                "Check hwmon labels and repeat the sweep after reboot.",
            );
        }
        if run.has_fan_response() {
            return Self::new(
                K::Responded,
                "fan2 RPM changed during the GPU PWM sweep.",
                "Compare this result with the opposite XG Mobile state.",
            );
        }
        Self::new(
            K::AcceptedNoResponse,
            "The GPU curve was accepted, but fan2 RPM did not meaningfully change.",
            "Run the changed-state sweep to see whether XG Mobile changes routing.",
        )
    }

    pub fn compare(first: &XgFanDiagnosticRun, second: &XgFanDiagnosticRun) -> Self {
        use XgFanDiagnosisKind as K;
        if first.has_different_labels_from(second) {
            return Self::new(
                K::NewHwmonLikely,
                "Fan labels changed between runs.",
                "Inspect the new hwmon device paths before trusting fan2.",
            );
        }
        if first.has_missing_fan2_rpm() || second.has_missing_fan2_rpm() {
            return Self::new(
                K::Inconclusive,
                "At least one run did not expose fan2 RPM.",
                "Record hwmon labels and repeat after a clean reboot.",
            );
        }
        if first.has_fan_response() != second.has_fan_response() {
            return Self::new(
                K::RouteChanged,
                "Only one XG Mobile state produced a fan2 RPM response.",
                "Treat XG Mobile state as changing the GPU fan control route.",
            );
        }
        if first.fan2_baseline_delta_from(second).is_some_and(|d| d >= 500) {
            return Self::new(
                K::GpuDependsOnXgMobile,
                "fan2 baseline RPM changed between XG Mobile states.",
                "Treat XG Mobile state as changing firmware fan policy even if pwm2 is not direct control.",
            );
        }
        if first.has_fan_response() && second.has_fan_response() {
            return Self::new(
                K::Responded,
                "Both runs produced fan2 RPM response.",
                "GPU fan control appears independent of XG Mobile state.",
            );
        }
        Self::new(
            K::AcceptedNoResponse,
            "Both runs accepted GPU curve writes without a meaningful RPM change.",
            "Look for a separate XG Mobile fan interface or EC/SAIO control path.",
        )
    }

    /// Diagnosis for whatever runs are present.
    pub fn for_session(session: &XgFanDiagnosticSession) -> Option<Self> {
        match (&session.current_run, &session.changed_run) {
            (Some(current), Some(changed)) => Some(Self::compare(current, changed)),
            (_, Some(changed)) => Some(Self::single(changed)),
            (Some(current), None) => Some(Self::single(current)),
            (None, None) => None,
        }
    }
}

// ----------------------------------------------------------------------------
// Hypothesis tests
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanHypothesisKind {
    GpuDependsOnCpuFan,
    ProfileFloorOrCap,
    DownwardApplied,
    DownwardStoredButRuntimeClamped,
    DownwardTransformedToDefault,
    UpwardApplied,
    CurveNoRuntimeEffect,
    CurveFollowsRuntime,
    CurveTransformed,
    CurveStoredButIgnored,
    CurveInconclusive,
    NoEvidence,
    Inconclusive,
}

impl FanHypothesisKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::GpuDependsOnCpuFan => "GPU fan follows CPU fan policy",
            Self::ProfileFloorOrCap => "Profile floor or cap detected",
            Self::DownwardApplied => "Below-default curve applied",
            Self::DownwardStoredButRuntimeClamped => "Below-default curve stored, runtime clamped",
            Self::DownwardTransformedToDefault => "Below-default curve transformed to default",
            Self::UpwardApplied => "Above-default curve applied",
            Self::CurveNoRuntimeEffect => "Curve stored with no runtime effect",
            Self::CurveFollowsRuntime => "Curve follows interpolated runtime expectation",
            Self::CurveTransformed => "Curve transformed before use",
            Self::CurveStoredButIgnored => "Curve stored but ignored at runtime",
            Self::CurveInconclusive => "Runtime curve result inconclusive",
            Self::NoEvidence => "No evidence in this run",
            Self::Inconclusive => "Inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanHypothesisDiagnosis {
    pub kind: FanHypothesisKind,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FanHypothesisStep {
    pub label: String,
    pub requested_fan: String,
    pub requested_pwm: i64,
    pub current_temp_c: Option<i64>,
    pub default_pwm: Option<i64>,
    pub readback_pwm: Option<i64>,
    pub cpu_rpm_before: Option<i64>,
    pub cpu_rpm_after: Option<i64>,
    pub cpu_rpm_delta: Option<i64>,
    pub samples: Vec<XgFanHwmonSnapshot>,
}

impl FanHypothesisStep {
    pub fn simple(label: String, fan: &str, pwm: i64, samples: Vec<XgFanHwmonSnapshot>) -> Self {
        Self {
            label,
            requested_fan: fan.to_string(),
            requested_pwm: pwm,
            samples,
            ..Default::default()
        }
    }

    pub fn fan2_last_rpm(&self) -> Option<i64> {
        min_max_last(&fan2_values(&self.samples)).2
    }

    pub fn fan2_delta_rpm(&self) -> Option<i64> {
        let (min, max, _) = min_max_last(&fan2_values(&self.samples));
        Some(max? - min?)
    }

    pub fn cpu_last_rpm(&self) -> Option<i64> {
        self.cpu_rpm_after
            .or_else(|| self.samples.iter().filter_map(|s| s.fan1_rpm).next_back())
    }

    pub fn describe(&self) -> String {
        let fmt = |v: Option<i64>| v.map_or("n/a".to_string(), |v| v.to_string());
        let signed = |v: Option<i64>| v.map_or("n/a".to_string(), |v| if v > 0 { format!("+{v}") } else { v.to_string() });
        if self.default_pwm.is_some()
            || self.readback_pwm.is_some()
            || self.cpu_rpm_before.is_some()
            || self.cpu_rpm_after.is_some()
            || self.cpu_rpm_delta.is_some()
        {
            format!(
                "{}: temp {}C, PWM default={} requested={} readback={}, CPU RPM {}->{} delta={}",
                self.label,
                fmt(self.current_temp_c),
                fmt(self.default_pwm),
                self.requested_pwm,
                fmt(self.readback_pwm),
                fmt(self.cpu_rpm_before),
                fmt(self.cpu_rpm_after),
                signed(self.cpu_rpm_delta),
            )
        } else {
            format!(
                "{}: {} PWM {}, CPU RPM {}",
                self.label,
                self.requested_fan,
                self.requested_pwm,
                fmt(self.cpu_last_rpm())
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFanBoundary {
    pub profile: PerfProfile,
    pub low_pwm_step: FanHypothesisStep,
    pub high_pwm_step: FanHypothesisStep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanHypothesisResult {
    pub name: String,
    pub diagnosis: FanHypothesisDiagnosis,
    pub steps: Vec<FanHypothesisStep>,
}

/// One curve-rule case: requested vs. readback curve and the RPM it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurveRuleCaseObservation {
    pub label: String,
    pub default_pwm_values: Vec<i64>,
    pub requested_pwm_values: Vec<i64>,
    pub readback_pwm_values: Vec<i64>,
    pub temp_values: Vec<i64>,
    pub before: XgFanHwmonSnapshot,
    pub samples: Vec<XgFanHwmonSnapshot>,
}

const PWM_THRESHOLD: f64 = 20.0;
const RPM_THRESHOLD: i64 = 500;

impl CurveRuleCaseObservation {
    pub fn current_temp_c(&self) -> Option<i64> {
        let milli = match self.samples.last() {
            Some(sample) => sample.cpu_temp_milli_c,
            None => self.before.cpu_temp_milli_c,
        }?;
        Some((milli as f64 / 1000.0).round() as i64)
    }

    fn interpolate(&self, pwm_values: &[i64]) -> Option<f64> {
        interpolate_pwm(&self.temp_values, pwm_values, self.current_temp_c()?)
    }

    pub fn default_pwm_at_temp(&self) -> Option<f64> {
        self.interpolate(&self.default_pwm_values)
    }

    pub fn requested_pwm_at_temp(&self) -> Option<f64> {
        self.interpolate(&self.requested_pwm_values)
    }

    pub fn readback_pwm_at_temp(&self) -> Option<f64> {
        self.interpolate(&self.readback_pwm_values)
    }

    pub fn cpu_rpm_before(&self) -> Option<i64> {
        self.before.fan1_rpm
    }

    pub fn cpu_rpm_after(&self) -> Option<i64> {
        self.samples.last()?.fan1_rpm
    }

    pub fn cpu_rpm_delta(&self) -> Option<i64> {
        Some(self.cpu_rpm_after()? - self.cpu_rpm_before()?)
    }

    fn requests_below_default(&self) -> bool {
        matches!((self.requested_pwm_at_temp(), self.default_pwm_at_temp()), (Some(r), Some(d)) if r <= d - PWM_THRESHOLD)
    }

    fn requests_above_default(&self) -> bool {
        matches!((self.requested_pwm_at_temp(), self.default_pwm_at_temp()), (Some(r), Some(d)) if r >= d + PWM_THRESHOLD)
    }

    fn readback_near_default(&self) -> bool {
        matches!((self.readback_pwm_at_temp(), self.default_pwm_at_temp()), (Some(r), Some(d)) if (r - d).abs() <= PWM_THRESHOLD)
    }

    fn readback_near_requested(&self) -> bool {
        matches!((self.readback_pwm_at_temp(), self.requested_pwm_at_temp()), (Some(r), Some(q)) if (r - q).abs() <= PWM_THRESHOLD)
    }

    pub fn downward_applied(&self) -> bool {
        self.requests_below_default()
            && self.readback_near_requested()
            && self.cpu_rpm_delta().is_some_and(|d| d <= -RPM_THRESHOLD)
    }

    pub fn downward_stored_but_runtime_clamped(&self) -> bool {
        self.requests_below_default()
            && self.readback_near_requested()
            && self.cpu_rpm_delta().is_some_and(|d| d > -RPM_THRESHOLD)
    }

    pub fn downward_transformed_to_default(&self) -> bool {
        self.requests_below_default() && self.readback_near_default() && !self.readback_near_requested()
    }

    pub fn upward_applied(&self) -> bool {
        self.requests_above_default()
            && self.readback_near_requested()
            && self.cpu_rpm_delta().is_some_and(|d| d >= RPM_THRESHOLD)
    }

    pub fn curve_no_runtime_effect(&self) -> bool {
        (self.requests_below_default() || self.requests_above_default())
            && self.readback_near_requested()
            && self.cpu_rpm_delta().is_some_and(|d| d.abs() < RPM_THRESHOLD)
    }

    pub fn curve_transformed(&self) -> bool {
        self.requested_pwm_values != self.readback_pwm_values
    }

    fn priority(&self) -> i64 {
        let boundary = if self.label.contains("until_80c") { 100_000 } else { 0 };
        let rpm = self.cpu_rpm_delta().unwrap_or(0).abs();
        let pwm = (self.default_pwm_at_temp().unwrap_or(0.0) - self.requested_pwm_at_temp().unwrap_or(0.0))
            .abs()
            .round() as i64;
        boundary + rpm + pwm
    }

    pub fn to_step(&self) -> FanHypothesisStep {
        FanHypothesisStep {
            label: self.label.clone(),
            requested_fan: "CPU".into(),
            requested_pwm: self.requested_pwm_at_temp().map_or(0, |v| v.round() as i64),
            current_temp_c: self.current_temp_c(),
            default_pwm: self.default_pwm_at_temp().map(|v| v.round() as i64),
            readback_pwm: self.readback_pwm_at_temp().map(|v| v.round() as i64),
            cpu_rpm_before: self.cpu_rpm_before(),
            cpu_rpm_after: self.cpu_rpm_after(),
            cpu_rpm_delta: self.cpu_rpm_delta(),
            samples: self.samples.clone(),
        }
    }
}

/// Linear interpolation of a curve at `temp_c`, clamped to the end points.
pub fn interpolate_pwm(temp_values: &[i64], pwm_values: &[i64], temp_c: i64) -> Option<f64> {
    if temp_values.is_empty() || pwm_values.is_empty() || temp_values.len() != pwm_values.len() {
        return None;
    }
    if temp_c <= temp_values[0] {
        return Some(pwm_values[0] as f64);
    }
    for i in 1..temp_values.len() {
        let upper = temp_values[i];
        if temp_c <= upper {
            let lower = temp_values[i - 1];
            let span = upper - lower;
            if span <= 0 {
                return Some(pwm_values[i] as f64);
            }
            let ratio = (temp_c - lower) as f64 / span as f64;
            return Some(pwm_values[i - 1] as f64 + (pwm_values[i] - pwm_values[i - 1]) as f64 * ratio);
        }
    }
    pwm_values.last().map(|v| *v as f64)
}

fn diagnosis(kind: FanHypothesisKind, summary: impl Into<String>, next_action: impl Into<String>) -> FanHypothesisDiagnosis {
    FanHypothesisDiagnosis {
        kind,
        summary: summary.into(),
        next_action: next_action.into(),
    }
}

impl FanHypothesisDiagnosis {
    pub fn cpu_gpu_coupling(cpu_only: &[FanHypothesisStep], gpu_only: &[FanHypothesisStep]) -> Self {
        let max_delta = |steps: &[FanHypothesisStep]| steps.iter().filter_map(FanHypothesisStep::fan2_delta_rpm).max();
        match (max_delta(cpu_only), max_delta(gpu_only)) {
            (Some(cpu), Some(gpu)) if cpu >= RPM_THRESHOLD && gpu < RPM_THRESHOLD => diagnosis(
                FanHypothesisKind::GpuDependsOnCpuFan,
                "fan2 changed during CPU-only sweep but not GPU-only sweep.",
                "Treat fan2 as coupled to CPU/common EC policy, not direct GPU pwm2 control.",
            ),
            (Some(_), Some(_)) => diagnosis(
                FanHypothesisKind::NoEvidence,
                "CPU-only sweep did not show stronger fan2 response than GPU-only sweep.",
                "Use profile bounds and curve validation tests next.",
            ),
            _ => diagnosis(
                FanHypothesisKind::Inconclusive,
                "fan2 RPM data was missing during coupling test.",
                "Repeat after verifying fan2_input is readable.",
            ),
        }
    }

    pub fn profile_bounds(boundaries: &[ProfileFanBoundary]) -> Self {
        if boundaries.len() < 2 {
            return diagnosis(
                FanHypothesisKind::Inconclusive,
                "At least two profiles are needed to compare floor/cap.",
                "Run the bounds test for Quiet, Balanced, and Performance.",
            );
        }
        let lows: Vec<i64> = boundaries.iter().filter_map(|b| b.low_pwm_step.fan2_last_rpm()).collect();
        let highs: Vec<i64> = boundaries.iter().filter_map(|b| b.high_pwm_step.fan2_last_rpm()).collect();
        if lows.len() < 2 || highs.len() < 2 {
            return diagnosis(
                FanHypothesisKind::Inconclusive,
                "RPM data was missing for profile bounds.",
                "Repeat after verifying fan RPM sensors are readable.",
            );
        }
        let spread = |v: &[i64]| v.iter().max().unwrap() - v.iter().min().unwrap();
        if spread(&lows) >= RPM_THRESHOLD || spread(&highs) >= RPM_THRESHOLD {
            return diagnosis(
                FanHypothesisKind::ProfileFloorOrCap,
                "Different profiles produced different RPM floors or caps.",
                "Model fan percentages as profile-limited requests, not absolute speeds.",
            );
        }
        diagnosis(
            FanHypothesisKind::NoEvidence,
            "Profile RPM floors/caps were not distinguishable in this run.",
            "Repeat with higher thermal load if needed.",
        )
    }

    pub fn curve_rule_discovery(observations: &[CurveRuleCaseObservation]) -> Self {
        use FanHypothesisKind as K;
        if observations.is_empty() {
            return diagnosis(K::Inconclusive, "No curve rule cases were collected.", "Repeat the curve rule discovery test.");
        }
        let first = |pred: fn(&CurveRuleCaseObservation) -> bool| observations.iter().find(|o| pred(o));

        if let Some(o) = first(CurveRuleCaseObservation::downward_transformed_to_default) {
            return diagnosis(
                K::DownwardTransformedToDefault,
                format!("{}: below-default request was read back near the default curve.", o.label),
                "Treat lower-than-default points as rejected or clamped by asusd/EC storage validation.",
            );
        }
        if let Some(o) = first(CurveRuleCaseObservation::curve_transformed) {
            return diagnosis(
                K::CurveTransformed,
                format!("{}: readback curve differs from the requested curve.", o.label),
                "Compare requested/readback points to identify the exact EC/asusd rewrite rule.",
            );
        }
        let applied: Vec<&CurveRuleCaseObservation> = observations.iter().filter(|o| o.downward_applied()).collect();
        if !applied.is_empty() {
            let mut best = applied[0];
            for candidate in &applied[1..] {
                if candidate.priority() > best.priority() {
                    best = candidate;
                }
            }
            return diagnosis(
                K::DownwardApplied,
                format!(
                    "{}: below-default curve was stored and CPU RPM dropped. {}",
                    best.label,
                    curve_rule_numbers(best)
                ),
                "Below-default points can affect runtime in this case; compare which shapes remain accepted. \
                 drop threshold=500 RPM; below-default threshold=20 PWM.",
            );
        }
        if let Some(o) = first(CurveRuleCaseObservation::downward_stored_but_runtime_clamped) {
            return diagnosis(
                K::DownwardStoredButRuntimeClamped,
                format!("{}: below-default curve was stored, but CPU RPM did not drop.", o.label),
                "Treat the stored curve as subject to a runtime floor or EC clamp below the default policy.",
            );
        }
        if let Some(o) = first(CurveRuleCaseObservation::upward_applied) {
            return diagnosis(
                K::UpwardApplied,
                format!("{}: above-default curve raised CPU RPM.", o.label),
                "Upward curve changes are visible; focus further tests on downward clamps and profile caps.",
            );
        }
        if let Some(o) = first(CurveRuleCaseObservation::curve_no_runtime_effect) {
            return diagnosis(
                K::CurveNoRuntimeEffect,
                format!(
                    "{}: requested/readback PWM differed from default, but CPU RPM barely moved.",
                    o.label
                ),
                "Treat this as no runtime effect for that curve shape and compare with profile bounds.",
            );
        }
        diagnosis(
            K::CurveInconclusive,
            "Curve cases did not produce enough requested/default PWM or RPM separation to infer a rule.",
            "Repeat with a stronger thermal load or longer sample window around a known CPU temperature range.",
        )
    }
}

fn curve_rule_numbers(o: &CurveRuleCaseObservation) -> String {
    let int = |v: Option<i64>| v.map_or("unknown".to_string(), |v| v.to_string());
    let float = |v: Option<f64>| match v {
        None => "unknown".to_string(),
        Some(v) if v == v.round() => (v.round() as i64).to_string(),
        Some(v) => format!("{v:.1}"),
    };
    let rpm = match (o.cpu_rpm_before(), o.cpu_rpm_after(), o.cpu_rpm_delta()) {
        (Some(before), Some(after), Some(delta)) => {
            let delta = if delta > 0 { format!("+{delta}") } else { delta.to_string() };
            format!("CPU RPM {before}->{after}, delta={delta}")
        }
        _ => "CPU RPM unavailable".to_string(),
    };
    format!(
        "temp={}C; PWM default={}, requested={}, readback={}; {rpm}.",
        int(o.current_temp_c()),
        float(o.default_pwm_at_temp()),
        float(o.requested_pwm_at_temp()),
        float(o.readback_pwm_at_temp()),
    )
}
