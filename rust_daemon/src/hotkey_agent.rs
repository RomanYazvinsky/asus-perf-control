use anyhow::{Context, Result};
use futures_util::{future::BoxFuture, Stream, StreamExt};
use std::time::Duration;
use zbus::zvariant::OwnedObjectPath;

const DAEMON_DESTINATION: &str = "com.asuslinuxcontrol.Daemon";
const DAEMON_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const DAEMON_INTERFACE: &str = "com.asuslinuxcontrol.Daemon";
const ROG_KEY_SIGNAL: &str = "RogKeyPressed";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const LOGIND_GATE_TIMEOUT: Duration = Duration::from_secs(2);
const LOGIND_DESTINATION: &str = "org.freedesktop.login1";
const SEAT0_PATH: &str = "/org/freedesktop/login1/seat/seat0";
const SEAT_INTERFACE: &str = "org.freedesktop.login1.Seat";
const SESSION_INTERFACE: &str = "org.freedesktop.login1.Session";

// On GNOME the ROG key (KEY_PROG1 = XF86Launch1) is bound as a custom shortcut instead: only a
// launch from a shell-handled shortcut carries the XDG_ACTIVATION_TOKEN that lets the window
// come to the foreground under Wayland focus-stealing prevention.
const MEDIA_KEYS_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const CUSTOM_KEYBINDING_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
pub const GNOME_SHORTCUT_PATH: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/asus-linux-control/";
const ROG_KEYSYM: &str = "XF86Launch1";
// Started directly (not via gtk-launch) so the app receives GNOME's activation token.
const APP_COMMAND: &str = "asus-linux-control";

pub trait CommandRunner {
    fn run<'a>(&'a mut self, program: &'a str, args: &'a [&'a str]) -> BoxFuture<'a, Result<()>>;
}

pub trait ActivationGate {
    fn allows_activation(&mut self) -> BoxFuture<'_, Result<bool>>;
}

pub struct LogindSeatGate {
    connection: zbus::Connection,
    uid: u32,
}

impl LogindSeatGate {
    pub fn new(connection: zbus::Connection) -> Self {
        // SAFETY: geteuid has no preconditions and does not modify memory.
        let uid = unsafe { nix::libc::geteuid() };
        Self { connection, uid }
    }
}

impl ActivationGate for LogindSeatGate {
    fn allows_activation(&mut self) -> BoxFuture<'_, Result<bool>> {
        Box::pin(async move {
            tokio::time::timeout(LOGIND_GATE_TIMEOUT, async {
                let seat = zbus::Proxy::new(
                    &self.connection,
                    LOGIND_DESTINATION,
                    SEAT0_PATH,
                    SEAT_INTERFACE,
                )
                .await
                .context("failed to create logind seat0 proxy")?;
                let (_, session_path): (String, OwnedObjectPath) = seat
                    .get_property("ActiveSession")
                    .await
                    .context("failed to read logind seat0 ActiveSession")?;
                let session = zbus::Proxy::new(
                    &self.connection,
                    LOGIND_DESTINATION,
                    session_path.as_str(),
                    SESSION_INTERFACE,
                )
                .await
                .context("failed to create active logind session proxy")?;
                let (active_uid, _): (u32, OwnedObjectPath) = session
                    .get_property("User")
                    .await
                    .context("failed to read active logind session User")?;

                Ok(active_uid == self.uid)
            })
            .await
            .context("timed out checking logind seat0 owner")?
        })
    }
}

pub struct ProcessCommandRunner {
    timeout: Duration,
}

impl ProcessCommandRunner {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for ProcessCommandRunner {
    fn default() -> Self {
        Self::with_timeout(COMMAND_TIMEOUT)
    }
}

impl CommandRunner for ProcessCommandRunner {
    fn run<'a>(&'a mut self, program: &'a str, args: &'a [&'a str]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut command = tokio::process::Command::new(program);
            command.args(args).kill_on_drop(true);
            let status = tokio::time::timeout(self.timeout, command.status())
                .await
                .with_context(|| format!("{program} timed out after {:?}", self.timeout))?
                .with_context(|| format!("failed to run {program}"))?;

            if !status.success() {
                anyhow::bail!("{program} exited with status {status}");
            }

            Ok(())
        })
    }
}

pub async fn activate_app(runner: &mut impl CommandRunner) -> Result<()> {
    runner.run("gtk-launch", &["asus-linux-control"]).await
}

/// `XDG_CURRENT_DESKTOP` is a colon-separated list, e.g. `ubuntu:GNOME`.
pub fn is_gnome_session(current_desktop: Option<&str>) -> bool {
    current_desktop.is_some_and(|desktops| desktops.split(':').any(|d| d.eq_ignore_ascii_case("GNOME")))
}

/// Adds `path` to a `gsettings get ... custom-keybindings` value (`@as []` or `['/a/', '/b/']`).
/// Returns `None` when it is already listed.
pub fn with_custom_keybinding(current: &str, path: &str) -> Option<String> {
    let list = current.trim().trim_start_matches("@as").trim();
    let list = list.strip_prefix('[').and_then(|l| l.strip_suffix(']')).unwrap_or("");
    let mut paths: Vec<String> = list
        .split(',')
        .map(|p| p.trim().trim_matches('\'').to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if paths.iter().any(|p| p == path) {
        return None;
    }
    paths.push(path.to_string());
    let quoted: Vec<String> = paths.iter().map(|p| format!("'{p}'")).collect();
    Some(format!("[{}]", quoted.join(", ")))
}

async fn gsettings(args: &[&str]) -> Result<String> {
    let output = tokio::time::timeout(COMMAND_TIMEOUT, tokio::process::Command::new("gsettings").args(args).output())
        .await
        .context("gsettings timed out")?
        .context("failed to run gsettings")?;
    if !output.status.success() {
        anyhow::bail!("gsettings {args:?} failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Binds the ROG key to the app as a GNOME custom shortcut (idempotent).
pub async fn ensure_gnome_shortcut() -> Result<()> {
    let schema = format!("{CUSTOM_KEYBINDING_SCHEMA}:{GNOME_SHORTCUT_PATH}");
    gsettings(&["set", &schema, "name", "ASUS Linux Control"]).await?;
    gsettings(&["set", &schema, "command", APP_COMMAND]).await?;
    gsettings(&["set", &schema, "binding", ROG_KEYSYM]).await?;
    let current = gsettings(&["get", MEDIA_KEYS_SCHEMA, "custom-keybindings"]).await?;
    if let Some(updated) = with_custom_keybinding(&current, GNOME_SHORTCUT_PATH) {
        gsettings(&["set", MEDIA_KEYS_SCHEMA, "custom-keybindings", &updated]).await?;
    }
    Ok(())
}

pub async fn process_signals<S, T>(
    mut signals: S,
    runner: &mut impl CommandRunner,
    gate: &mut impl ActivationGate,
) -> Result<()>
where
    S: Stream<Item = T> + Unpin,
{
    while signals.next().await.is_some() {
        match gate.allows_activation().await {
            Ok(true) => {}
            Ok(false) => {
                log::debug!("skipping ROG activation: this UID does not own seat0 ActiveSession");
                continue;
            }
            Err(error) => {
                log::error!("failed to verify seat0 owner for ROG activation: {error:#}");
                continue;
            }
        }
        if let Err(error) = activate_app(runner).await {
            log::error!("failed to activate ASUS Linux Control after ROG key press: {error:#}");
        }
    }

    anyhow::bail!("RogKeyPressed signal stream ended unexpectedly")
}

pub async fn run() -> Result<()> {
    if is_gnome_session(std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref()) {
        match ensure_gnome_shortcut().await {
            Ok(()) => {
                log::info!("ROG key is bound to ASUS Linux Control as a GNOME shortcut ({ROG_KEYSYM})");
                // GNOME launches the app; launching it here as well would race the shortcut's
                // focused activation with an unfocused one.
                return std::future::pending().await;
            }
            Err(error) => {
                log::warn!("failed to register GNOME shortcut, launching the app on ROG key signals instead: {error:#}");
            }
        }
    }

    let connection = zbus::Connection::system()
        .await
        .context("failed to connect ROG hotkey agent to the system bus")?;
    let proxy = zbus::Proxy::new(
        &connection,
        DAEMON_DESTINATION,
        DAEMON_PATH,
        DAEMON_INTERFACE,
    )
    .await
    .context("failed to create ROG hotkey daemon proxy")?;
    proxy
        .call_method("GetRogHotkeyStatus", &())
        .await
        .context("failed to activate ROG hotkey daemon through status call")?;
    let signals = proxy
        .receive_signal(ROG_KEY_SIGNAL)
        .await
        .context("failed to subscribe to RogKeyPressed")?;
    let mut runner = ProcessCommandRunner::default();
    let mut gate = LogindSeatGate::new(connection.clone());

    process_signals(signals, &mut runner, &mut gate).await
}
