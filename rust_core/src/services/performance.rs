// Performance mode controller
// Manages ASUS WMI thermal policy via D-Bus daemon

use std::process::Command;

const DBUS_DEST: &str = "com.asuslinuxcontrol.Daemon";
const DBUS_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const DBUS_IFACE: &str = "com.asuslinuxcontrol.Daemon";

pub struct PerformanceController;

impl PerformanceController {
    pub fn new() -> Self {
        PerformanceController
    }

    /// Read current thermal policy via D-Bus daemon
    pub fn read_thermal_policy(&self) -> Result<i32, String> {
        let output = Command::new("dbus-send")
            .args([
                "--system",
                "--print-reply",
                &format!("--dest={DBUS_DEST}"),
                DBUS_PATH,
                &format!("{DBUS_IFACE}.GetPerformanceMode"),
            ])
            .output()
            .map_err(|e| format!("Failed to call D-Bus: {e}. Is asus-control-daemon installed?"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("D-Bus call failed: {stderr}"));
        }

        // Parse dbus-send output: "   int32 1\n"
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("int32") {
                    trimmed.split_whitespace().last()?.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("Unexpected D-Bus response: {stdout}"))
    }

    /// Write thermal policy via D-Bus daemon
    pub fn write_thermal_policy(&self, mode: i32) -> Result<(), String> {
        if mode < 0 || mode > 2 {
            return Err("Invalid mode: must be 0, 1, or 2".to_string());
        }

        let output = Command::new("dbus-send")
            .args([
                "--system",
                "--print-reply",
                &format!("--dest={DBUS_DEST}"),
                DBUS_PATH,
                &format!("{DBUS_IFACE}.SetPerformanceMode"),
                &format!("int32:{mode}"),
            ])
            .output()
            .map_err(|e| format!("Failed to call D-Bus: {e}. Is asus-control-daemon installed?"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("D-Bus call failed: {stderr}"));
        }

        Ok(())
    }
}

impl Default for PerformanceController {
    fn default() -> Self {
        Self::new()
    }
}
