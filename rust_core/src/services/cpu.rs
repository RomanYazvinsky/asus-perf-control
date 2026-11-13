// CPU model name and clock speed

use std::fs;

const CPU_DIR: &str = "/sys/devices/system/cpu";
const CPUINFO: &str = "/proc/cpuinfo";

/// Short CPU model name from /proc/cpuinfo, e.g. `Ryzen 9 5980HS`.
pub fn read_cpu_name() -> Option<String> {
    let cpuinfo = fs::read_to_string(CPUINFO).ok()?;
    let model = cpuinfo
        .lines()
        .find(|line| line.starts_with("model name"))?
        .split_once(':')?
        .1;
    Some(short_cpu_name(model))
}

/// `AMD Ryzen 9 5980HS with Radeon Graphics` → `Ryzen 9 5980HS`,
/// `11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz` → `Core i7-11800H`.
pub fn short_cpu_name(model: &str) -> String {
    let model = model.replace("(R)", "").replace("(TM)", "");
    let model = [" with ", " @ ", " CPU"]
        .iter()
        .fold(model.as_str(), |m, cut| m.split(cut).next().unwrap_or(m));
    let words: Vec<&str> = model.split_whitespace().collect();
    // Drop the vendor and everything before it ("11th Gen Intel").
    let start = words
        .iter()
        .position(|w| *w == "AMD" || *w == "Intel")
        .map_or(0, |i| i + 1);
    let short = words[start..].join(" ");
    if short.is_empty() { words.join(" ") } else { short }
}

/// Average current clock across all online cores, in MHz.
pub fn read_cpu_freq_mhz() -> Option<u32> {
    let khz: Vec<u64> = fs::read_dir(CPU_DIR)
        .ok()?
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("cpu").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .filter_map(|e| fs::read_to_string(e.path().join("cpufreq/scaling_cur_freq")).ok())
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    average_mhz(&khz)
}

/// Average of kHz readings, in MHz.
pub fn average_mhz(khz: &[u64]) -> Option<u32> {
    if khz.is_empty() {
        return None;
    }
    Some((khz.iter().sum::<u64>() / khz.len() as u64 / 1000) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortens_cpu_names() {
        assert_eq!(short_cpu_name(" AMD Ryzen 9 5980HS with Radeon Graphics"), "Ryzen 9 5980HS");
        assert_eq!(short_cpu_name("11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz"), "Core i7-11800H");
        assert_eq!(short_cpu_name("Intel(R) Core(TM) i7-8750H CPU @ 2.20GHz"), "Core i7-8750H");
        assert_eq!(short_cpu_name("AMD Ryzen 7 4800HS"), "Ryzen 7 4800HS");
    }

    #[test]
    fn averages_khz_to_mhz() {
        assert_eq!(average_mhz(&[1_400_000, 3_000_000]), Some(2200));
        assert_eq!(average_mhz(&[]), None);
    }
}
