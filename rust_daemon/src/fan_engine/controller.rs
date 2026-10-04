use super::config::{
    config_from_validated, EngineCurvePoint, FanEngineConfig, FanEngineConfigValidationError,
    ValidatedFanEngineConfig,
};

#[derive(Debug)]
pub enum FanEngineControllerError {
    Validation(FanEngineConfigValidationError),
    RuntimeUnavailable(String),
    Runtime(String),
    NotRunning,
    InvalidFanId(String),
}

impl std::fmt::Display for FanEngineControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(err) => write!(f, "{err}"),
            Self::RuntimeUnavailable(err) => write!(f, "{err}"),
            Self::Runtime(err) => write!(f, "{err}"),
            Self::NotRunning => write!(f, "Fan engine is not running"),
            Self::InvalidFanId(fan_id) => write!(f, "Unsupported fan id: {fan_id}"),
        }
    }
}

impl std::error::Error for FanEngineControllerError {}

impl From<FanEngineConfigValidationError> for FanEngineControllerError {
    fn from(value: FanEngineConfigValidationError) -> Self {
        Self::Validation(value)
    }
}

pub trait FanEngineDriver: Send {
    fn start(&mut self, config: ValidatedFanEngineConfig) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn update_config(&mut self, config: ValidatedFanEngineConfig) -> Result<(), String>;
    fn is_active(&self) -> bool;
}

#[cfg(test)]
struct UnavailableFanEngineDriver;

#[cfg(test)]
impl FanEngineDriver for UnavailableFanEngineDriver {
    fn start(&mut self, _config: ValidatedFanEngineConfig) -> Result<(), String> {
        Err("Fan engine runtime hardware adapter is not wired".to_string())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn update_config(&mut self, _config: ValidatedFanEngineConfig) -> Result<(), String> {
        Err("Fan engine runtime hardware adapter is not wired".to_string())
    }

    fn is_active(&self) -> bool {
        false
    }
}

pub struct FanEngineController {
    running: bool,
    config: Option<ValidatedFanEngineConfig>,
    driver: Box<dyn FanEngineDriver>,
}

impl FanEngineController {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::new_with_driver(Box::new(UnavailableFanEngineDriver))
    }

    pub fn new_with_driver(driver: Box<dyn FanEngineDriver>) -> Self {
        Self {
            running: false,
            config: None,
            driver,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running && self.driver.is_active()
    }

    pub fn current_config(&self) -> Option<FanEngineConfig> {
        self.config.as_ref().map(config_from_validated)
    }

    pub fn start(&mut self, config: FanEngineConfig) -> Result<(), FanEngineControllerError> {
        let validated = config.validate()?;
        self.driver
            .start(validated.clone())
            .map_err(FanEngineControllerError::RuntimeUnavailable)?;
        self.config = Some(validated);
        self.running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), FanEngineControllerError> {
        if !self.running {
            return Ok(());
        }
        self.driver
            .stop()
            .map_err(FanEngineControllerError::Runtime)?;
        self.running = false;
        self.config = None;
        Ok(())
    }

    pub fn update_curve(
        &mut self,
        fan_id: String,
        curve: Vec<EngineCurvePoint>,
    ) -> Result<(), FanEngineControllerError> {
        if !self.is_running() {
            self.running = false;
            self.config = None;
            return Err(FanEngineControllerError::NotRunning);
        }

        let cfg = self
            .config
            .as_mut()
            .ok_or(FanEngineControllerError::NotRunning)?;

        let validated_curve = FanEngineConfig {
            profile: cfg.profile.clone(),
            tick_ms: cfg.tick_ms,
            hysteresis_c: cfg.hysteresis_c,
            ramp_down_delay_ms: cfg.ramp_down_delay_ms,
            emergency_temp_c: cfg.emergency_temp_c,
            cpu_curve: curve.clone(),
            gpu_curve: curve.clone(),
            cross_fan: cfg.cross_fan,
            cross_fan_curve: None,
        }
        .validate()?;

        let mut next_config = cfg.clone();
        match fan_id.as_str() {
            "cpu" | "CPU" => {
                next_config.cpu_curve = validated_curve.cpu_curve;
            }
            "gpu" | "GPU" => {
                next_config.gpu_curve = validated_curve.gpu_curve;
            }
            _ => return Err(FanEngineControllerError::InvalidFanId(fan_id)),
        }

        self.driver
            .update_config(next_config.clone())
            .map_err(FanEngineControllerError::Runtime)?;
        *cfg = next_config;
        Ok(())
    }
}
