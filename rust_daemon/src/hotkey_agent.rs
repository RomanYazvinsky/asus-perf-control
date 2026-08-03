use anyhow::{Context, Result};
use futures_util::{future::BoxFuture, Stream, StreamExt};
use std::time::Duration;

const DAEMON_DESTINATION: &str = "com.asuslinuxcontrol.Daemon";
const DAEMON_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const DAEMON_INTERFACE: &str = "com.asuslinuxcontrol.Daemon";
const ROG_KEY_SIGNAL: &str = "RogKeyPressed";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub trait CommandRunner {
    fn run<'a>(&'a mut self, program: &'a str, args: &'a [&'a str]) -> BoxFuture<'a, Result<()>>;
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

pub async fn process_signals<S, T>(mut signals: S, runner: &mut impl CommandRunner) -> Result<()>
where
    S: Stream<Item = T> + Unpin,
{
    while signals.next().await.is_some() {
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

    process_signals(signals, &mut runner).await
}
