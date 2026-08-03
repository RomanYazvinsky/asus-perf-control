use anyhow::{Context, Result};
use futures_util::{future::BoxFuture, Stream, StreamExt};
use std::time::Duration;
use zbus::zvariant::OwnedObjectPath;

const DAEMON_DESTINATION: &str = "com.asuslinuxcontrol.Daemon";
const DAEMON_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const DAEMON_INTERFACE: &str = "com.asuslinuxcontrol.Daemon";
const ROG_KEY_SIGNAL: &str = "RogKeyPressed";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const LOGIND_DESTINATION: &str = "org.freedesktop.login1";
const SEAT0_PATH: &str = "/org/freedesktop/login1/seat/seat0";
const SEAT_INTERFACE: &str = "org.freedesktop.login1.Seat";
const SESSION_INTERFACE: &str = "org.freedesktop.login1.Session";

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
    let signals = proxy
        .receive_signal(ROG_KEY_SIGNAL)
        .await
        .context("failed to subscribe to RogKeyPressed")?;
    let mut runner = ProcessCommandRunner::default();
    let mut gate = LogindSeatGate::new(connection.clone());

    process_signals(signals, &mut runner, &mut gate).await
}
