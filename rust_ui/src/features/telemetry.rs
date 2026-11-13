//! Live hardware readings for the toolbar: one column per device (CPU, GPU1, GPU2) with
//! name, temperature, clock and fan speed, plus the active performance profile.

use async_trait::async_trait;

pub use asus_linux_control_core::services::gpu::GpuReading;

use super::fan::FanId;
use super::performance::mode_name;

/// Temperatures strictly above this are highlighted.
pub const HOT_TEMP_C: i32 = 85;

/// Toolbar shows at most this many GPUs (iGPU + dGPU).
pub const MAX_GPUS: usize = 2;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Telemetry {
    pub cpu_name: Option<String>,
    pub cpu_temp_c: Option<i32>,
    pub cpu_mhz: Option<u32>,
    /// Integrated GPU first.
    pub gpus: Vec<GpuReading>,
    pub fan_rpms: Vec<(FanId, Option<u32>)>,
    pub performance_mode: Option<i32>,
}

#[async_trait]
pub trait TelemetryService: Send + Sync {
    /// Sensor reads never fail as a whole: a missing sensor is `None`.
    async fn sample_telemetry(&self) -> Telemetry;
}

/// One toolbar column, ready to display.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceColumn {
    /// `CPU`, `GPU1`, `GPU2`.
    pub title: String,
    pub name: String,
    pub temp: String,
    pub hot: bool,
    pub clock: String,
    pub fan: String,
    pub hover: String,
}

pub fn device_columns(t: &Telemetry) -> Vec<DeviceColumn> {
    let fan = |id: FanId| {
        let rpm = t.fan_rpms.iter().find(|(fan, _)| *fan == id).and_then(|(_, rpm)| *rpm);
        format_rpm(rpm)
    };
    let cpu = DeviceColumn {
        title: "CPU".into(),
        name: t.cpu_name.clone().unwrap_or_else(|| "Unknown".into()),
        temp: format_temp(t.cpu_temp_c),
        hot: t.cpu_temp_c.is_some_and(is_hot),
        clock: format_clock(t.cpu_mhz),
        fan: fan(FanId::Cpu),
        hover: "Clock: average of all cores\nFan: CPU fan".into(),
    };
    let gpus = t.gpus.iter().take(MAX_GPUS).enumerate().map(|(index, gpu)| {
        // The integrated GPU sits on the CPU die and is cooled by the CPU fan.
        let fan_id = if gpu.integrated { FanId::Cpu } else { FanId::Gpu };
        DeviceColumn {
            title: format!("GPU{}", index + 1),
            name: gpu.name.clone(),
            temp: if gpu.active { format_temp(gpu.temp_c) } else { "off".into() },
            hot: gpu.active && gpu.temp_c.is_some_and(is_hot),
            clock: if gpu.active { format_clock(gpu.clock_mhz) } else { "--".into() },
            fan: fan(fan_id),
            hover: format!("{}\nPCI {}\nFan: {}", gpu.full_name, gpu.pci_address, fan_id.label()),
        }
    });
    std::iter::once(cpu).chain(gpus).collect()
}

pub fn is_hot(temp_c: i32) -> bool {
    temp_c > HOT_TEMP_C
}

pub fn format_temp(temp_c: Option<i32>) -> String {
    temp_c.map_or_else(|| "--".to_string(), |t| format!("{t}°C"))
}

pub fn format_clock(mhz: Option<u32>) -> String {
    match mhz {
        Some(mhz) if mhz >= 1000 => format!("{:.2} GHz", f64::from(mhz) / 1000.0),
        Some(mhz) => format!("{mhz} MHz"),
        None => "--".to_string(),
    }
}

pub fn format_rpm(rpm: Option<u32>) -> String {
    rpm.map_or_else(|| "--".to_string(), |rpm| format!("{rpm} RPM"))
}

pub fn format_profile(mode: Option<i32>) -> &'static str {
    mode.map_or("--", mode_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(name: &str, integrated: bool, active: bool, temp_c: Option<i32>) -> GpuReading {
        GpuReading {
            pci_address: "0000:01:00.0".into(),
            name: name.into(),
            full_name: format!("[{name}]"),
            active,
            integrated,
            temp_c,
            clock_mhz: active.then_some(1500),
        }
    }

    fn telemetry() -> Telemetry {
        Telemetry {
            cpu_name: Some("Ryzen 9 5980HS".into()),
            cpu_temp_c: Some(86),
            cpu_mhz: Some(3215),
            gpus: vec![
                gpu("Radeon Vega Series", true, true, Some(60)),
                gpu("GeForce GTX 1650 Mobile", false, false, None),
            ],
            fan_rpms: vec![(FanId::Cpu, Some(2300)), (FanId::Gpu, Some(2100))],
            performance_mode: Some(1),
        }
    }

    #[test]
    fn hot_is_strictly_above_85() {
        assert!(!is_hot(85));
        assert!(is_hot(86));
    }

    #[test]
    fn builds_cpu_and_gpu_columns() {
        let columns = device_columns(&telemetry());
        let titles: Vec<_> = columns.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, ["CPU", "GPU1", "GPU2"]);

        let cpu = &columns[0];
        assert_eq!((cpu.name.as_str(), cpu.temp.as_str(), cpu.hot), ("Ryzen 9 5980HS", "86°C", true));
        assert_eq!((cpu.clock.as_str(), cpu.fan.as_str()), ("3.21 GHz", "2300 RPM"));

        let igpu = &columns[1];
        assert_eq!((igpu.temp.as_str(), igpu.hot, igpu.clock.as_str()), ("60°C", false, "1.50 GHz"));
        assert_eq!(igpu.fan, "2300 RPM", "integrated GPU shares the CPU fan");

        let dgpu = &columns[2];
        assert_eq!((dgpu.temp.as_str(), dgpu.clock.as_str()), ("off", "--"));
        assert_eq!(dgpu.fan, "2100 RPM");
    }

    #[test]
    fn shows_at_most_two_gpus() {
        let mut t = telemetry();
        t.gpus.push(gpu("XG Mobile", false, true, Some(50)));
        assert_eq!(device_columns(&t).len(), 1 + MAX_GPUS);
    }

    #[test]
    fn formats_values() {
        assert_eq!(format_temp(None), "--");
        assert_eq!(format_clock(Some(800)), "800 MHz");
        assert_eq!(format_clock(None), "--");
        assert_eq!(format_rpm(None), "--");
        assert_eq!(format_profile(Some(2)), "Turbo");
        assert_eq!(format_profile(None), "--");
    }
}
