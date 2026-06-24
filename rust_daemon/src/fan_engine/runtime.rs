use super::config::ValidatedFanEngineConfig;
use super::interpolation::interpolate_pwm;

pub trait FanHardwareAdapter: Send {
    fn read_cpu_temp(&mut self) -> Result<i32, String>;
    fn read_gpu_temp(&mut self) -> Result<i32, String>;
    fn write_cpu_flat_curve(&mut self, pwm: u8) -> Result<(), String>;
    fn write_gpu_flat_curve(&mut self, pwm: u8) -> Result<(), String>;
    fn set_cpu_auto_mode(&mut self) -> Result<(), String>;
    fn set_gpu_auto_mode(&mut self) -> Result<(), String>;
}

impl<T> FanHardwareAdapter for Box<T>
where
    T: FanHardwareAdapter + ?Sized,
{
    fn read_cpu_temp(&mut self) -> Result<i32, String> {
        (**self).read_cpu_temp()
    }

    fn read_gpu_temp(&mut self) -> Result<i32, String> {
        (**self).read_gpu_temp()
    }

    fn write_cpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        (**self).write_cpu_flat_curve(pwm)
    }

    fn write_gpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        (**self).write_gpu_flat_curve(pwm)
    }

    fn set_cpu_auto_mode(&mut self) -> Result<(), String> {
        (**self).set_cpu_auto_mode()
    }

    fn set_gpu_auto_mode(&mut self) -> Result<(), String> {
        (**self).set_gpu_auto_mode()
    }
}

pub struct FanEngineRuntime<H>
where
    H: FanHardwareAdapter,
{
    config: ValidatedFanEngineConfig,
    hardware: H,
}

impl<H> FanEngineRuntime<H>
where
    H: FanHardwareAdapter,
{
    pub fn new(config: ValidatedFanEngineConfig, hardware: H) -> Self {
        Self { config, hardware }
    }

    pub fn update_config(&mut self, config: ValidatedFanEngineConfig) {
        self.config = config;
    }

    pub fn config(&self) -> &ValidatedFanEngineConfig {
        &self.config
    }

    pub fn tick_once(&mut self) -> Result<(), String> {
        let cpu_temp = self.hardware.read_cpu_temp()?;
        let gpu_temp = self.hardware.read_gpu_temp()?;

        let cpu_pwm = interpolate_pwm(&self.config.cpu_curve, cpu_temp);
        let gpu_pwm = interpolate_pwm(&self.config.gpu_curve, gpu_temp);

        let mut target_pwm = if cpu_temp >= gpu_temp {
            cpu_pwm
        } else {
            gpu_pwm
        };

        let max_temp = cpu_temp.max(gpu_temp);
        if max_temp >= self.config.emergency_temp_c {
            target_pwm = 255;
        }

        self.hardware.write_cpu_flat_curve(target_pwm)?;
        self.hardware.write_gpu_flat_curve(target_pwm)?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();

        if let Err(err) = self.hardware.set_cpu_auto_mode() {
            errors.push(format!("CPU auto mode restore failed: {err}"));
        }

        if let Err(err) = self.hardware.set_gpu_auto_mode() {
            errors.push(format!("GPU auto mode restore failed: {err}"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    #[cfg(test)]
    pub fn hardware(&self) -> &H {
        &self.hardware
    }
}
