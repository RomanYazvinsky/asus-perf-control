#![allow(dead_code)]

#[path = "../src/hotkey_agent.rs"]
mod hotkey_agent;

use futures_util::{future::BoxFuture, stream};
use hotkey_agent::{activate_app, process_signals, CommandRunner, ProcessCommandRunner};
use std::time::{Duration, Instant};

#[derive(Default)]
struct FakeRunner {
    calls: Vec<(String, Vec<String>)>,
    failures_remaining: usize,
}

impl CommandRunner for FakeRunner {
    fn run<'a>(
        &'a mut self,
        program: &'a str,
        args: &'a [&'a str],
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        self.calls.push((
            program.to_string(),
            args.iter().map(|arg| arg.to_string()).collect(),
        ));
        let should_fail = self.failures_remaining > 0;

        if should_fail {
            self.failures_remaining -= 1;
        }

        Box::pin(async move {
            if should_fail {
                anyhow::bail!("injected launch failure");
            }

            Ok(())
        })
    }
}

#[tokio::test]
async fn activate_app_invokes_the_installed_desktop_app_exactly_once() {
    let mut runner = FakeRunner::default();

    activate_app(&mut runner).await.unwrap();

    assert_eq!(
        runner.calls,
        vec![(
            "gtk-launch".to_string(),
            vec!["asus-linux-control".to_string()]
        )]
    );
}

#[tokio::test]
async fn production_runner_rejects_a_nonzero_exit_status() {
    let mut runner = ProcessCommandRunner::default();

    let error = runner.run("sh", &["-c", "exit 7"]).await.unwrap_err();

    assert!(error.to_string().contains("status"));
}

#[tokio::test]
async fn production_runner_reports_a_spawn_failure() {
    let mut runner = ProcessCommandRunner::default();

    let error = runner
        .run("/definitely/missing/asus-hotkey-test-command", &[])
        .await
        .unwrap_err();

    assert!(error.to_string().contains("failed to run"));
}

#[tokio::test]
async fn production_runner_times_out_a_hung_command() {
    let mut runner = ProcessCommandRunner::with_timeout(Duration::from_millis(25));
    let started = Instant::now();

    let error = runner.run("sh", &["-c", "exec sleep 5"]).await.unwrap_err();

    assert!(error.to_string().contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn signal_loop_continues_after_an_activation_failure() {
    let signals = stream::iter([(), ()]);
    let mut runner = FakeRunner {
        failures_remaining: 1,
        ..FakeRunner::default()
    };

    let error = process_signals(signals, &mut runner).await.unwrap_err();

    assert_eq!(runner.calls.len(), 2);
    assert!(error.to_string().contains("ended unexpectedly"));
    assert!(runner.calls.iter().all(|call| {
        call == &(
            "gtk-launch".to_string(),
            vec!["asus-linux-control".to_string()],
        )
    }));
}

#[tokio::test]
async fn signal_stream_eof_requests_a_supervisor_restart() {
    let signals = stream::empty::<()>();
    let mut runner = FakeRunner::default();

    let error = process_signals(signals, &mut runner).await.unwrap_err();

    assert!(error.to_string().contains("ended unexpectedly"));
}

#[test]
fn production_runner_uses_async_kill_on_drop_processes() {
    let source = include_str!("../src/hotkey_agent.rs");

    assert!(source.contains("tokio::process::Command"));
    assert!(source.contains("kill_on_drop(true)"));
    assert!(!source.contains("std::process::Command"));
}
