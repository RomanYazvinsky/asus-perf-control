use super::runtime::FanHardwareAdapter;

#[cfg(test)]
#[allow(dead_code)]
#[path = "../hardware/mod.rs"]
mod hardware;

#[cfg(test)]
use hardware::fan::{
    discover_asus_curve_base_path, read_cpu_temp, read_gpu_temp, set_fan_auto_mode,
    write_flat_fan_curve, FanChannel,
};

#[cfg(not(test))]
use crate::hardware::fan::{
    discover_asus_curve_base_path, read_cpu_temp, read_gpu_temp, set_fan_auto_mode,
    write_flat_fan_curve, FanChannel,
};
use std::path::PathBuf;

pub struct SysfsFanHardwareAdapter {
    curve_base_path: PathBuf,
}

impl SysfsFanHardwareAdapter {
    pub fn discover() -> Result<Self, String> {
        let curve_base_path = discover_asus_curve_base_path().map_err(|e| e.to_string())?;
        Ok(Self { curve_base_path })
    }

    #[cfg(test)]
    pub fn new(curve_base_path: PathBuf) -> Self {
        Self { curve_base_path }
    }
}

impl FanHardwareAdapter for SysfsFanHardwareAdapter {
    fn read_cpu_temp(&mut self) -> Result<i32, String> {
        read_cpu_temp().map_err(|e| e.to_string())
    }

    fn read_gpu_temp(&mut self) -> Result<i32, String> {
        read_gpu_temp().map_err(|e| e.to_string())
    }

    fn write_cpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        write_flat_fan_curve(&self.curve_base_path, FanChannel::Cpu, pwm).map_err(|e| e.to_string())
    }

    fn write_gpu_flat_curve(&mut self, pwm: u8) -> Result<(), String> {
        write_flat_fan_curve(&self.curve_base_path, FanChannel::Gpu, pwm).map_err(|e| e.to_string())
    }

    fn set_cpu_auto_mode(&mut self) -> Result<(), String> {
        set_fan_auto_mode(&self.curve_base_path, FanChannel::Cpu).map_err(|e| e.to_string())
    }

    fn set_gpu_auto_mode(&mut self) -> Result<(), String> {
        set_fan_auto_mode(&self.curve_base_path, FanChannel::Gpu).map_err(|e| e.to_string())
    }
}
