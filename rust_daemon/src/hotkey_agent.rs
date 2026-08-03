use anyhow::{Context, Result};
use futures_util::{Stream, StreamExt};
use std::process::Command;

const DAEMON_DESTINATION: &str = "com.asuslinuxcontrol.Daemon";
const DAEMON_PATH: &str = "/com/asuslinuxcontrol/Daemon";
const DAEMON_INTERFACE: &str = "com.asuslinuxcontrol.Daemon";
const ROG_KEY_SIGNAL: &str = "RogKeyPressed";

pub trait CommandRunner {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<()>;
}

pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(program)
            .args(args)
            .status()
            .with_context(|| format!("failed to run {program}"))?;

        if !status.success() {
            anyhow::bail!("{program} exited with status {status}");
        }

        Ok(())
    }
}

pub fn activate_app(runner: &mut impl CommandRunner) -> Result<()> {
    runner.run("gtk-launch", &["asus-linux-control"])
}

pub async fn process_signals<S, T>(mut signals: S, runner: &mut impl CommandRunner)
where
    S: Stream<Item = T> + Unpin,
{
    while signals.next().await.is_some() {
        if let Err(error) = activate_app(runner) {
            log::error!("failed to activate ASUS Linux Control after ROG key press: {error:#}");
        }
    }
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
    let mut runner = ProcessCommandRunner;

    process_signals(signals, &mut runner).await;
    Ok(())
}
