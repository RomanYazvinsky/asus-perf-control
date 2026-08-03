#![cfg_attr(test, allow(dead_code))]

mod fan_engine;
mod hardware;
mod package_updates;
mod polkit;
mod rog_hotkey;
mod rog_input;

use fan_engine::config::{
    clear_saved_fan_engine_config, read_saved_fan_engine_config, save_fan_engine_config,
    EngineCurvePoint, FanEngineConfig,
};
use fan_engine::controller::FanEngineController;
use fan_engine::hardware_adapter::SysfsFanHardwareAdapter;
use fan_engine::runtime::FanHardwareAdapter;
use fan_engine::threaded_driver::ThreadedFanEngineDriver;
use hardware::{cpu_boost, temp};
use package_updates::backend::detect_backend;
use package_updates::commands::{command_for_backend, PackageCommandKind};
use package_updates::job::UpdateJobManager;
use package_updates::parser::parse_check_output;
use package_updates::status::{PackageBackend, UpdateErrorCode, UpdateState, UpdateStatus};

use anyhow::Result;
use log::{debug, error, info};
use std::sync::Mutex;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use zbus::zvariant::{OwnedValue, Value};
use zbus::{interface, ConnectionBuilder};

const BUS_NAME: &str = "com.asuslinuxcontrol.Daemon";
const OBJECT_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const ASUSD_BUS_NAME: &str = "xyz.ljones.Asusd";
const ASUSD_PLATFORM_PATH: &str = "/xyz/ljones";
const ASUSD_PLATFORM_INTERFACE: &str = "xyz.ljones.Platform";
const ASUSD_PLATFORM_PROFILE_PROPERTY: &str = "PlatformProfile";
const PROFILE_POLL_INTERVAL: Duration = Duration::from_secs(5);

struct AsusControlDaemon {
    fan_engine: Mutex<FanEngineController>,
    package_update_jobs: UpdateJobManager,
    rog_hotkey_status: rog_hotkey::RogHotkeyStatusStore,
}

async fn authorize_fan_engine_write(
    conn: &zbus::Connection,
    sender: &str,
) -> Result<(), zbus::fdo::Error> {
    polkit::check_authorization(conn, sender, "com.asuslinuxcontrol.manage-fan-engine")
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}

async fn authorize_package_update_write(
    conn: &zbus::Connection,
    sender: &str,
) -> Result<(), zbus::fdo::Error> {
    polkit::check_authorization(conn, sender, "com.asuslinuxcontrol.manage-updates")
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}

fn status_json(status: UpdateStatus) -> Result<String, zbus::fdo::Error> {
    status
        .to_json()
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}

async fn run_update_check_command(
    backend: PackageBackend,
) -> Result<UpdateStatus, zbus::fdo::Error> {
    let command = command_for_backend(backend, PackageCommandKind::Check)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let output = Command::new(command.executable)
        .args(command.args)
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute package update check: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(1);

    Ok(parse_check_output(backend, &stdout, &stderr, exit_code))
}

fn parse_curve(value: serde_json::Value) -> Result<Vec<EngineCurvePoint>, zbus::fdo::Error> {
    serde_json::from_value::<Vec<EngineCurvePoint>>(value)
        .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("Invalid curve json: {e}")))
}

fn requires_system_epp_baseline(mode: cpu_boost::CpuBoostMode) -> bool {
    matches!(
        mode,
        cpu_boost::CpuBoostMode::Disabled | cpu_boost::CpuBoostMode::Enabled
    )
}

async fn reapply_active_asusd_platform_profile() -> Result<(), zbus::fdo::Error> {
    let conn = zbus::Connection::system()
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let props = zbus::fdo::PropertiesProxy::builder(&conn)
        .destination(ASUSD_BUS_NAME)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
        .path(ASUSD_PLATFORM_PATH)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
        .build()
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let interface = zbus::names::InterfaceName::try_from(ASUSD_PLATFORM_INTERFACE)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

    let platform_profile = props
        .get(interface.clone(), ASUSD_PLATFORM_PROFILE_PROPERTY)
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let platform_profile_value =
        Value::try_from(&platform_profile).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

    props
        .set(
            interface,
            ASUSD_PLATFORM_PROFILE_PROPERTY,
            &platform_profile_value,
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

    cpu_boost::refresh_system_epp_baseline().map_err(|e| {
        error!("Failed to refresh CPU boost EPP baseline: {}", e);
        zbus::fdo::Error::Failed(e.to_string())
    })?;

    Ok(())
}

fn platform_profile_name(value: OwnedValue) -> Result<String, zbus::fdo::Error> {
    if let Ok(profile) = u32::try_from(&value) {
        return match profile {
            0 => Ok("Balanced".to_string()),
            1 => Ok("Performance".to_string()),
            2 => Ok("Quiet".to_string()),
            3 => Ok("LowPower".to_string()),
            other => Err(zbus::fdo::Error::Failed(format!(
                "Unknown PlatformProfile value: {other}"
            ))),
        };
    }

    if let Ok(profile) = String::try_from(value) {
        return Ok(profile);
    }

    Err(zbus::fdo::Error::Failed(
        "Unsupported PlatformProfile value type".to_string(),
    ))
}

async fn active_asusd_platform_profile(
    conn: &zbus::Connection,
) -> Result<String, zbus::fdo::Error> {
    let props = zbus::fdo::PropertiesProxy::builder(conn)
        .destination(ASUSD_BUS_NAME)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
        .path(ASUSD_PLATFORM_PATH)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
        .build()
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let interface = zbus::names::InterfaceName::try_from(ASUSD_PLATFORM_INTERFACE)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

    let platform_profile = props
        .get(interface, ASUSD_PLATFORM_PROFILE_PROPERTY)
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

    platform_profile_name(platform_profile)
}

async fn apply_active_profile_temp_limit(conn: &zbus::Connection) -> Result<(), zbus::fdo::Error> {
    let profile = active_asusd_platform_profile(conn).await?;
    info!("Applying saved CPU temperature setting for profile={profile}");
    temp::apply_saved_temp_limit_for_profile(&profile).map_err(|e| {
        error!(
            "Failed to apply saved CPU temperature setting for profile={}: {}",
            profile, e
        );
        zbus::fdo::Error::Failed(e.to_string())
    })
}

fn restore_saved_fan_engine_on_startup(daemon: &AsusControlDaemon) -> Result<(), String> {
    let Some(config) = read_saved_fan_engine_config().map_err(|e| e.to_string())? else {
        return Ok(());
    };

    info!("Restoring saved dynamic fan engine config on startup");
    let mut controller = daemon
        .fan_engine
        .lock()
        .map_err(|_| "Fan engine mutex poisoned".to_string())?;
    controller.start(config).map_err(|e| e.to_string())
}

async fn monitor_platform_profile_changes(conn: zbus::Connection) {
    let mut last_profile: Option<String> = None;

    loop {
        match active_asusd_platform_profile(&conn).await {
            Ok(profile) => {
                if last_profile.as_deref() != Some(profile.as_str()) {
                    info!("Platform profile changed to {profile}");
                    if let Err(e) = temp::apply_saved_temp_limit_for_profile(&profile) {
                        error!(
                            "Failed to apply CPU temperature setting after profile change: {}",
                            e
                        );
                    }
                    last_profile = Some(profile);
                }
            }
            Err(e) => {
                error!("Failed to read active platform profile: {}", e);
            }
        }

        sleep(PROFILE_POLL_INTERVAL).await;
    }
}

async fn shutdown_already_pending(
    sigint: &mut tokio::signal::unix::Signal,
    sigterm: &mut tokio::signal::unix::Signal,
) -> bool {
    tokio::time::timeout(Duration::from_millis(1), async {
        tokio::select! {
            _ = sigint.recv() => (),
            _ = sigterm.recv() => (),
        }
    })
    .await
    .is_ok()
}

#[interface(name = "com.asuslinuxcontrol.Daemon")]
impl AsusControlDaemon {
    /// Read ROG hotkey runtime diagnostics.
    async fn get_rog_hotkey_status(&self) -> Result<String, zbus::fdo::Error> {
        self.rog_hotkey_status
            .to_json()
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }

    #[zbus(signal)]
    async fn rog_key_pressed(signal_context: &zbus::SignalContext<'_>) -> zbus::Result<()>;

    /// Start dynamic fan engine from JSON config.
    async fn start_fan_engine(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        config_json: String,
    ) -> Result<bool, zbus::fdo::Error> {
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?
            .to_string();
        authorize_fan_engine_write(conn, &sender).await?;

        let config = serde_json::from_str::<FanEngineConfig>(&config_json).map_err(|e| {
            zbus::fdo::Error::InvalidArgs(format!("Invalid fan engine config json: {e}"))
        })?;

        let mut controller = self
            .fan_engine
            .lock()
            .map_err(|_| zbus::fdo::Error::Failed("Fan engine mutex poisoned".to_string()))?;

        controller
            .start(config.clone())
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.to_string()))?;
        save_fan_engine_config(&config).map_err(|e| {
            error!("Failed to save fan engine config: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })?;

        Ok(true)
    }

    /// Stop dynamic fan engine and restore auto mode.
    async fn stop_fan_engine(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
    ) -> Result<bool, zbus::fdo::Error> {
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?
            .to_string();
        authorize_fan_engine_write(conn, &sender).await?;

        let mut controller = self
            .fan_engine
            .lock()
            .map_err(|_| zbus::fdo::Error::Failed("Fan engine mutex poisoned".to_string()))?;

        controller
            .stop()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        clear_saved_fan_engine_config().map_err(|e| {
            error!("Failed to clear saved fan engine config: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })?;

        Ok(true)
    }

    /// Update fan curve while engine is running.
    async fn update_fan_engine_curve(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        fan_id: String,
        curve_json: String,
    ) -> Result<bool, zbus::fdo::Error> {
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?
            .to_string();
        authorize_fan_engine_write(conn, &sender).await?;

        let parsed: serde_json::Value = serde_json::from_str(&curve_json)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("Invalid curve json: {e}")))?;
        let curve = parse_curve(parsed)?;

        let mut controller = self
            .fan_engine
            .lock()
            .map_err(|_| zbus::fdo::Error::Failed("Fan engine mutex poisoned".to_string()))?;

        controller
            .update_curve(fan_id, curve)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        if let Some(config) = controller.current_config() {
            save_fan_engine_config(&config).map_err(|e| {
                error!("Failed to save updated fan engine config: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })?;
        }

        Ok(true)
    }

    /// Read running state of dynamic fan engine.
    async fn get_fan_engine_running(&self) -> Result<bool, zbus::fdo::Error> {
        let controller = self
            .fan_engine
            .lock()
            .map_err(|_| zbus::fdo::Error::Failed("Fan engine mutex poisoned".to_string()))?;

        Ok(controller.is_running())
    }

    /// Get detected native package update backend.
    async fn get_package_backend(&self) -> Result<String, zbus::fdo::Error> {
        Ok(detect_backend().as_str().to_string())
    }

    /// Check for ASUS Linux Control package updates through the native backend.
    async fn check_for_updates(&self) -> Result<String, zbus::fdo::Error> {
        let backend = detect_backend();
        if backend == PackageBackend::Unsupported {
            return status_json(UpdateStatus::unsupported(
                "Package updates are unsupported on this system",
            ));
        }

        match run_update_check_command(backend).await {
            Ok(status) => status_json(status),
            Err(e) => status_json(UpdateStatus {
                backend,
                state: UpdateState::Failed,
                installed_version: None,
                available_version: None,
                reboot_required: None,
                summary: "Package manager is unavailable".to_string(),
                error_code: Some(UpdateErrorCode::PackageManagerUnavailable),
                log_tail: vec![e.to_string()],
            }),
        }
    }

    /// Start one-click ASUS Linux Control package update.
    async fn start_update(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
    ) -> Result<String, zbus::fdo::Error> {
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?
            .to_string();
        authorize_package_update_write(conn, &sender).await?;

        let backend = detect_backend();
        if backend == PackageBackend::Unsupported {
            return Err(zbus::fdo::Error::Failed(
                "Package updates are unsupported on this system".to_string(),
            ));
        }

        self.package_update_jobs
            .start_update(backend)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Read update job status.
    async fn get_update_status(&self, update_id: String) -> Result<String, zbus::fdo::Error> {
        self.package_update_jobs
            .get_status(&update_id)
            .ok_or_else(|| zbus::fdo::Error::InvalidArgs("Unknown update id".to_string()))
            .and_then(status_json)
    }

    /// Cancel a running update job.
    async fn cancel_update(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        update_id: String,
    ) -> Result<bool, zbus::fdo::Error> {
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?
            .to_string();
        authorize_package_update_write(conn, &sender).await?;

        Ok(self.package_update_jobs.cancel_update(&update_id))
    }

    /// Get saved CPU temperature limit (0 = no saved limit)
    /// No authorization required for reading
    async fn get_temp_limit(&self) -> Result<i32, zbus::fdo::Error> {
        info!("GetTempLimit called");

        let conn = zbus::Connection::system()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        let profile = active_asusd_platform_profile(&conn).await?;

        temp::read_profile_temp_limit(&profile).map_err(|e| {
            error!(
                "Failed to read saved temp limit for profile={}: {}",
                profile, e
            );
            zbus::fdo::Error::Failed(e.to_string())
        })
    }

    /// Get CPU boost status as JSON.
    /// No authorization required for reading.
    async fn get_cpu_boost_status(&self) -> Result<String, zbus::fdo::Error> {
        info!("GetCpuBoostStatus called");

        let status = cpu_boost::get_cpu_boost_status().map_err(|e| {
            error!("Failed to get CPU boost status: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })?;
        cpu_boost::status_to_json(&status).map_err(|e| {
            error!("Failed to serialize CPU boost status: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })
    }

    /// Set CPU boost mode.
    /// Requires polkit authorization because this writes CPUFreq sysfs.
    async fn set_cpu_boost_mode(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        mode: String,
    ) -> Result<bool, zbus::fdo::Error> {
        info!("SetCpuBoostMode called with mode={}", mode);

        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?;

        polkit::check_authorization(
            conn,
            sender.as_str(),
            "com.asuslinuxcontrol.set-cpu-boost-mode",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        let mode = cpu_boost::mode_from_daemon_id(&mode).map_err(|e| {
            error!("Invalid CPU boost mode: {}", e);
            zbus::fdo::Error::InvalidArgs(e.to_string())
        })?;

        if requires_system_epp_baseline(mode) {
            reapply_active_asusd_platform_profile().await?;
        }

        cpu_boost::apply_cpu_boost_mode(mode).map_err(|e| {
            error!("Failed to set CPU boost mode: {}", e);
            zbus::fdo::Error::Failed(e.to_string())
        })?;

        Ok(true)
    }

    /// Set CPU temperature limit (60-100°C, 0 to clear)
    /// Saves to file and applies via ryzenadj
    /// Requires polkit authorization
    async fn set_temp_limit(
        &self,
        #[zbus(connection)] conn: &zbus::Connection,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        temp_c: i32,
    ) -> Result<bool, zbus::fdo::Error> {
        info!("SetTempLimit called with temp={}°C", temp_c);

        // Validate input (0 means "clear saved limit")
        if temp_c != 0 && !(60..=100).contains(&temp_c) {
            error!(
                "Invalid temperature limit: {}. Must be 0 (clear) or 60-100",
                temp_c
            );
            return Err(zbus::fdo::Error::InvalidArgs(format!(
                "Temperature limit must be 0 (clear) or 60-100°C, got {}°C",
                temp_c
            )));
        }

        // Check polkit authorization
        let sender = hdr
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message".to_string()))?;

        polkit::check_authorization(conn, sender.as_str(), "com.asuslinuxcontrol.set-temp-limit")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        let profile = active_asusd_platform_profile(conn).await?;

        // Apply and save for the active platform profile bucket.
        if temp_c == 0 {
            temp::save_profile_temp_limit(&profile, temp_c).map_err(|e| {
                error!("Failed to clear temp limit: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })?;
            info!("Temperature limit cleared for profile={profile}");
        } else {
            temp::save_profile_temp_limit(&profile, temp_c).map_err(|e| {
                error!("Failed to save temp limit: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })?;
            temp::apply_temp_limit_ryzenadj(temp_c).map_err(|e| {
                error!("Failed to apply temp limit via ryzenadj: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })?;
            info!(
                "Temperature limit set to {}°C and saved for profile={}",
                temp_c, profile
            );
        }

        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting ASUS Control Daemon");
    info!("Bus name: {}", BUS_NAME);
    info!("Object path: {}", OBJECT_PATH);

    // Register before any ROG firmware mutation so startup signals remain queued for cleanup.
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    let fan_engine_driver = ThreadedFanEngineDriver::new(Box::new(|| {
        Ok(Box::new(SysfsFanHardwareAdapter::discover()?) as Box<dyn FanHardwareAdapter>)
    }));

    let rog_hotkey_status = rog_hotkey::RogHotkeyStatusStore::new(
        rog_hotkey::RogHotkeyStatus::unsupported("ROG hotkey runtime is starting"),
    );

    let daemon = AsusControlDaemon {
        fan_engine: Mutex::new(FanEngineController::new_with_driver(Box::new(
            fan_engine_driver,
        ))),
        package_update_jobs: UpdateJobManager::new(),
        rog_hotkey_status: rog_hotkey_status.clone(),
    };

    if let Err(e) = restore_saved_fan_engine_on_startup(&daemon) {
        error!(
            "Failed to restore saved fan engine config on startup: {}",
            e
        );
        // Continue anyway - fan engine can be started manually from the UI.
    }

    let _conn = ConnectionBuilder::system()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, daemon)?
        .build()
        .await?;

    info!("ASUS Control D-Bus service is ready");

    // Apply saved temperature limit for the active platform profile on startup.
    if let Err(e) = apply_active_profile_temp_limit(&_conn).await {
        error!("Failed to apply saved temperature limit on startup: {}", e);
        // Continue anyway - not critical
    }

    tokio::spawn(monitor_platform_profile_changes(_conn.clone()));

    // Refresh system power-plan EPP baseline before applying saved CPU boost overlay.
    if let Err(e) = reapply_active_asusd_platform_profile().await {
        error!("Failed to refresh CPU boost EPP baseline on startup: {}", e);
        // Continue anyway - not critical
    }

    // Apply saved CPU boost mode on startup (if exists).
    if let Err(e) = cpu_boost::apply_saved_cpu_boost_mode_on_startup() {
        error!("Failed to apply saved CPU boost mode on startup: {}", e);
        // Continue anyway - not critical
    }

    if shutdown_already_pending(&mut sigint, &mut sigterm).await {
        info!("Shutdown signal received during startup; skipping ROG firmware mutation");
        return Ok(());
    }

    // Mutate ROG firmware only after all potentially blocking startup calls are complete.
    let (mut rog_hotkey_handle, mut rog_press_receiver) =
        rog_hotkey::start(rog_hotkey_status.clone())?;
    info!("ASUS Control Daemon started successfully");

    let rog_signal_connection = _conn.clone();
    let rog_signal_status = rog_hotkey_status.clone();
    tokio::spawn(async move {
        let signal_context = match zbus::SignalContext::new(&rog_signal_connection, OBJECT_PATH) {
            Ok(context) => context,
            Err(error) => {
                error!("Failed to create ROG hotkey signal context: {error}");
                rog_signal_status.set_signal_error(Some(error.to_string()));
                return;
            }
        };

        while rog_press_receiver.recv().await.is_some() {
            debug!("Accepted ROG key press; emitting D-Bus signal");
            if let Err(error) = AsusControlDaemon::rog_key_pressed(&signal_context).await {
                error!("Failed to emit RogKeyPressed signal: {error}");
                rog_signal_status.set_signal_error(Some(error.to_string()));
            } else {
                rog_signal_status.set_signal_error(None);
            }
        }
    });

    tokio::select! {
        _ = sigint.recv() => info!("SIGINT received; shutting down"),
        _ = sigterm.recv() => info!("SIGTERM received; shutting down"),
    }

    if let Err(error) = rog_hotkey_handle.shutdown() {
        error!("Failed to restore ROG firmware binding during shutdown: {error:#}");
        return Err(error);
    }

    Ok(())
}
