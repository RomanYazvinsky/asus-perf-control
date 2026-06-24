// TDP (Thermal Design Power) and CPU temperature limit controller
// Manages CPU power limits and thermal ceiling via asus-nb-wmi sysfs attributes

use std::fs;
use std::path::Path;

const PPT_PL1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl";
const PPT_PL2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt";
const PPT_CPU_TEMP_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit";
const PPT_APU_SPPT_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_apu_sppt"; // Alternative for some models

const TDP_MIN_W: u32 = 5;
const TDP_MAX_W: u32 = 54;
const TEMP_MIN_C: u32 = 60;
const TEMP_MAX_C: u32 = 100;

pub struct TdpController;

impl TdpController {
    pub fn new() -> Self {
        TdpController
    }

    /// Check whether PPT sysfs attributes exist (kernel 5.17+ with asus-nb-wmi)
    pub fn is_supported() -> bool {
        Path::new(PPT_PL1_PATH).exists()
    }

    /// Detect which CPU temp limit path exists (model-dependent)
    /// Returns the correct path, or None if neither exists
    fn detect_temp_path() -> Option<&'static str> {
        if Path::new(PPT_CPU_TEMP_PATH).exists() {
            Some(PPT_CPU_TEMP_PATH)
        } else if Path::new(PPT_APU_SPPT_PATH).exists() {
            Some(PPT_APU_SPPT_PATH)
        } else {
            None
        }
    }

    /// Read Sustained TDP (PL1/SPL) from hardware — returns watts
    pub fn get_sustained_tdp(&self) -> Result<u32, String> {
        let raw = fs::read_to_string(PPT_PL1_PATH)
            .map_err(|e| format!("Failed to read sustained TDP: {e}. Check udev rules."))?;
        let mw: u32 = raw
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse sustained TDP value: {e}"))?;
        Ok(mw / 1000)
    }

    /// Read Boost TDP (PL2/SPPT) from hardware — returns watts
    pub fn get_boost_tdp(&self) -> Result<u32, String> {
        let raw = fs::read_to_string(PPT_PL2_PATH)
            .map_err(|e| format!("Failed to read boost TDP: {e}. Check udev rules."))?;
        let mw: u32 = raw
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse boost TDP value: {e}"))?;
        Ok(mw / 1000)
    }

    /// Read CPU thermal throttle ceiling — returns degrees Celsius
    /// Auto-detects ppt_cpu_limit or ppt_apu_sppt based on what exists
    pub fn get_cpu_temp_limit(&self) -> Result<u32, String> {
        let path = Self::detect_temp_path().ok_or_else(|| {
            "CPU temp limit not supported (neither ppt_cpu_limit nor ppt_apu_sppt exist)"
                .to_string()
        })?;

        let raw = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read CPU temp limit from {path}: {e}"))?;
        raw.trim()
            .parse()
            .map_err(|e| format!("Failed to parse CPU temp limit value: {e}"))
    }

    /// Set Sustained TDP (PL1/SPL) — validates 5–54W, writes milliwatts
    pub fn set_sustained_tdp(&self, watts: u32) -> Result<(), String> {
        Self::validate_tdp(watts, "Sustained TDP")?;
        let mw = watts * 1000;
        fs::write(PPT_PL1_PATH, mw.to_string())
            .map_err(|e| format!("Failed to set sustained TDP: {e}. Check udev rules."))
    }

    /// Set Boost TDP (PL2/SPPT) — validates 5–54W and >= sustained, writes milliwatts
    pub fn set_boost_tdp(&self, watts: u32) -> Result<(), String> {
        Self::validate_tdp(watts, "Boost TDP")?;
        let mw = watts * 1000;
        fs::write(PPT_PL2_PATH, mw.to_string())
            .map_err(|e| format!("Failed to set boost TDP: {e}. Check udev rules."))
    }

    /// Set CPU thermal throttle ceiling — validates 60–100°C
    /// Auto-detects and writes to the correct path (ppt_cpu_limit or ppt_apu_sppt)
    pub fn set_cpu_temp_limit(&self, celsius: u32) -> Result<(), String> {
        Self::validate_temp(celsius)?;

        let path = Self::detect_temp_path().ok_or_else(|| {
            "CPU temp limit not supported (neither ppt_cpu_limit nor ppt_apu_sppt exist)"
                .to_string()
        })?;

        fs::write(path, celsius.to_string())
            .map_err(|e| format!("Failed to set CPU temp limit at {path}: {e}. Check udev rules."))
    }

    fn validate_tdp(watts: u32, label: &str) -> Result<(), String> {
        if watts < TDP_MIN_W || watts > TDP_MAX_W {
            return Err(format!(
                "{label} must be {TDP_MIN_W}–{TDP_MAX_W}W, got {watts}W"
            ));
        }
        Ok(())
    }

    fn validate_temp(celsius: u32) -> Result<(), String> {
        if celsius < TEMP_MIN_C || celsius > TEMP_MAX_C {
            return Err(format!(
                "CPU temperature limit must be {TEMP_MIN_C}–{TEMP_MAX_C}°C, got {celsius}°C"
            ));
        }
        Ok(())
    }
}

impl Default for TdpController {
    fn default() -> Self {
        Self::new()
    }
}
