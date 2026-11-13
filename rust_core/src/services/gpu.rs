// GPU discovery, temperature and clock reading
// Enumerates PCI display controllers; names come from pci.ids, temperature and clock from
// the device's hwmon node or, for the proprietary NVIDIA driver (no hwmon), from nvidia-smi.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

const PCI_DEVICES_DIR: &str = "/sys/bus/pci/devices";
const PCI_IDS_FILES: [&str; 3] = ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids", "/usr/share/pci.ids"];
const NVIDIA_VENDOR_ID: u16 = 0x10de;

/// One GPU and its current temperature and clock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuReading {
    /// PCI address, e.g. `0000:01:00.0`.
    pub pci_address: String,
    /// Short marketing name, e.g. `GeForce GTX 1650 Mobile`.
    pub name: String,
    /// Full pci.ids device name, e.g. `TU117M [GeForce GTX 1650 Mobile / Max-Q]`.
    pub full_name: String,
    /// `false` when the GPU is runtime-suspended (powered off); it is not woken to read it.
    pub active: bool,
    /// Integrated GPU (the boot VGA device); it shares the CPU fan.
    pub integrated: bool,
    /// Temperature in °C, `None` when suspended or unreadable.
    pub temp_c: Option<i32>,
    /// Graphics clock in MHz, `None` when suspended or unreadable.
    pub clock_mhz: Option<u32>,
}

/// All PCI display controllers: integrated GPU first, then by PCI address.
pub fn read_gpus() -> Vec<GpuReading> {
    let Ok(entries) = fs::read_dir(PCI_DEVICES_DIR) else {
        return Vec::new();
    };
    let mut devices: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|path| read_trimmed(&path.join("class")).is_some_and(|class| class.starts_with("0x03")))
        .collect();
    devices.sort();
    let mut gpus: Vec<_> = devices.iter().filter_map(|path| read_gpu(path)).collect();
    gpus.sort_by_key(|gpu| !gpu.integrated);
    gpus
}

fn read_gpu(path: &Path) -> Option<GpuReading> {
    let pci_address = path.file_name()?.to_string_lossy().into_owned();
    let vendor = read_hex_id(&path.join("vendor"))?;
    let device = read_hex_id(&path.join("device"))?;
    // Reading a suspended GPU (especially through nvidia-smi) would wake it up.
    let active = read_trimmed(&path.join("power/runtime_status")).is_none_or(|status| status != "suspended");
    let full_name = device_name(vendor, device).unwrap_or_else(|| format!("GPU {vendor:04x}:{device:04x}"));
    let integrated = read_trimmed(&path.join("boot_vga")).is_some_and(|v| v == "1");
    let (temp_c, clock_mhz) = if !active {
        (None, None)
    } else if let Some(hwmon) = hwmon_dir(path) {
        (read_hwmon_temp(&hwmon), read_hwmon_clock_mhz(&hwmon))
    } else if vendor == NVIDIA_VENDOR_ID {
        nvidia_smi_readings(&pci_address).unwrap_or_default()
    } else {
        (None, None)
    };
    Some(GpuReading {
        pci_address,
        name: short_name(&full_name),
        full_name,
        active,
        integrated,
        temp_c,
        clock_mhz,
    })
}

fn hwmon_dir(device: &Path) -> Option<PathBuf> {
    Some(fs::read_dir(device.join("hwmon")).ok()?.flatten().next()?.path())
}

/// First `temp*_input` of the hwmon node, preferring the `edge` sensor (amdgpu).
fn read_hwmon_temp(hwmon: &Path) -> Option<i32> {
    let mut inputs: Vec<_> = fs::read_dir(&hwmon)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("temp") && name.ends_with("_input"))
        .collect();
    inputs.sort();
    let edge = inputs.iter().find(|input| {
        let label = hwmon.join(input.replace("_input", "_label"));
        read_trimmed(&label).is_some_and(|l| l == "edge")
    });
    let input = edge.or(inputs.first())?;
    let millidegrees: i32 = read_trimmed(&hwmon.join(input))?.parse().ok()?;
    Some(millidegrees / 1000)
}

/// amdgpu shader clock (`freq1_input`, in Hz).
fn read_hwmon_clock_mhz(hwmon: &Path) -> Option<u32> {
    let hz: u64 = read_trimmed(&hwmon.join("freq1_input"))?.parse().ok()?;
    Some((hz / 1_000_000) as u32)
}

/// (temperature °C, graphics clock MHz) from one nvidia-smi call.
fn nvidia_smi_readings(pci_address: &str) -> Option<(Option<i32>, Option<u32>)> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu,clocks.gr", "--format=csv,noheader,nounits", "-i", pci_address])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_nvidia_smi(&String::from_utf8_lossy(&output.stdout)))
}

/// Parses `53, 300` (temperature, clock); `[N/A]` fields become `None`.
pub fn parse_nvidia_smi(line: &str) -> (Option<i32>, Option<u32>) {
    let mut fields = line.trim().split(',').map(str::trim);
    let temp = fields.next().and_then(|f| f.parse().ok());
    let clock = fields.next().and_then(|f| f.parse().ok());
    (temp, clock)
}

/// pci.ids device name, cached per (vendor, device) since the database is large.
fn device_name(vendor: u16, device: u16) -> Option<String> {
    static CACHE: OnceLock<Mutex<HashMap<(u16, u16), Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    cache
        .entry((vendor, device))
        .or_insert_with(|| {
            PCI_IDS_FILES
                .iter()
                .find_map(|file| fs::read_to_string(file).ok())
                .and_then(|db| lookup_device_name(&db, vendor, device))
        })
        .clone()
}

/// Finds `device` under `vendor` in pci.ids content.
pub fn lookup_device_name(db: &str, vendor: u16, device: u16) -> Option<String> {
    let vendor_prefix = format!("{vendor:04x}  ");
    let device_prefix = format!("\t{device:04x}  ");
    let mut lines = db.lines().skip_while(|line| !line.starts_with(&vendor_prefix)).skip(1);
    lines
        .by_ref()
        .take_while(|line| line.starts_with('\t') || line.starts_with('#') || line.is_empty())
        .find_map(|line| line.strip_prefix(&device_prefix))
        .map(|name| name.trim().to_string())
}

/// `TU117M [GeForce GTX 1650 Mobile / Max-Q]` → `GeForce GTX 1650 Mobile`.
pub fn short_name(full_name: &str) -> String {
    let marketing = match (full_name.find('['), full_name.rfind(']')) {
        (Some(start), Some(end)) if start < end => &full_name[start + 1..end],
        _ => full_name,
    };
    marketing.split(" / ").next().unwrap_or(marketing).trim().to_string()
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_hex_id(path: &Path) -> Option<u16> {
    let value = read_trimmed(path)?;
    u16::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DB: &str = "\
# comment
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t1638  Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]
\t\t1043 16c2  Radeon Vega 8
10de  NVIDIA Corporation
\t1f9d  TU117M [GeForce GTX 1650 Mobile / Max-Q]
1f9d  Axelera AI
\t1638  Not a GPU
";

    #[test]
    fn looks_up_device_under_its_vendor() {
        assert_eq!(
            lookup_device_name(DB, 0x10de, 0x1f9d).as_deref(),
            Some("TU117M [GeForce GTX 1650 Mobile / Max-Q]")
        );
        assert_eq!(
            lookup_device_name(DB, 0x1002, 0x1638).as_deref(),
            Some("Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]")
        );
    }

    #[test]
    fn does_not_leak_into_next_vendor() {
        assert_eq!(lookup_device_name(DB, 0x10de, 0x1638), None);
        assert_eq!(lookup_device_name(DB, 0xffff, 0x1638), None);
    }

    #[test]
    fn parses_nvidia_smi_output() {
        assert_eq!(parse_nvidia_smi("53, 300\n"), (Some(53), Some(300)));
        assert_eq!(parse_nvidia_smi("[N/A], [N/A]"), (None, None));
        assert_eq!(parse_nvidia_smi(""), (None, None));
    }

    #[test]
    fn shortens_to_first_marketing_name() {
        assert_eq!(short_name("TU117M [GeForce GTX 1650 Mobile / Max-Q]"), "GeForce GTX 1650 Mobile");
        assert_eq!(
            short_name("Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]"),
            "Radeon Vega Series"
        );
        assert_eq!(short_name("GPU 10de:1f9d"), "GPU 10de:1f9d");
    }
}
