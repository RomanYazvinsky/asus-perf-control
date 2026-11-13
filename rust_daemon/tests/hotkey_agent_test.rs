#![allow(dead_code)]

#[path = "../src/hotkey_agent.rs"]
mod hotkey_agent;

use futures_util::{future::BoxFuture, stream};
use hotkey_agent::{
    activate_app, is_gnome_session, process_signals, with_custom_keybinding, ActivationGate,
    CommandRunner, ProcessCommandRunner, GNOME_SHORTCUT_PATH,
};
use std::collections::VecDeque;
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

struct FakeGate {
    results: VecDeque<anyhow::Result<bool>>,
    checks: usize,
}

impl ActivationGate for FakeGate {
    fn allows_activation(&mut self) -> BoxFuture<'_, anyhow::Result<bool>> {
        self.checks += 1;
        let result = self.results.pop_front().unwrap_or(Ok(false));
        Box::pin(async move { result })
    }
}

fn allowing_gate(count: usize) -> FakeGate {
    FakeGate {
        results: (0..count).map(|_| Ok(true)).collect(),
        checks: 0,
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
    let mut gate = allowing_gate(2);

    let error = process_signals(signals, &mut runner, &mut gate)
        .await
        .unwrap_err();

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
    let mut gate = allowing_gate(0);

    let error = process_signals(signals, &mut runner, &mut gate)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("ended unexpectedly"));
}

#[tokio::test]
async fn matching_active_seat_uid_activates_gui() {
    let mut runner = FakeRunner::default();
    let mut gate = allowing_gate(1);

    let _ = process_signals(stream::iter([()]), &mut runner, &mut gate).await;

    assert_eq!(gate.checks, 1);
    assert_eq!(runner.calls.len(), 1);
}

#[tokio::test]
async fn different_active_seat_uid_skips_gui_activation() {
    let mut runner = FakeRunner::default();
    let mut gate = FakeGate {
        results: VecDeque::from([Ok(false)]),
        checks: 0,
    };

    let _ = process_signals(stream::iter([()]), &mut runner, &mut gate).await;

    assert_eq!(gate.checks, 1);
    assert!(runner.calls.is_empty());
}

#[tokio::test]
async fn gate_error_skips_then_next_signal_retries_gate() {
    let mut runner = FakeRunner::default();
    let mut gate = FakeGate {
        results: VecDeque::from([Err(anyhow::anyhow!("logind unavailable")), Ok(true)]),
        checks: 0,
    };

    let _ = process_signals(stream::iter([(), ()]), &mut runner, &mut gate).await;

    assert_eq!(gate.checks, 2);
    assert_eq!(runner.calls.len(), 1);
}

#[test]
fn production_runner_uses_async_kill_on_drop_processes() {
    let source = include_str!("../src/hotkey_agent.rs");

    assert!(source.contains("tokio::process::Command"));
    assert!(source.contains("kill_on_drop(true)"));
    assert!(!source.contains("std::process::Command"));
}

#[test]
fn production_gate_is_async_timeout_bounded_and_daemon_is_activated_before_subscribe() {
    let source = include_str!("../src/hotkey_agent.rs");
    let startup_call = source.find("GetRogHotkeyStatus").unwrap();
    let subscribe = source.find("receive_signal(ROG_KEY_SIGNAL)").unwrap();

    assert!(source.contains("tokio::time::timeout"));
    assert!(startup_call < subscribe);
}

#[test]
fn detects_gnome_sessions() {
    assert!(is_gnome_session(Some("GNOME")));
    assert!(is_gnome_session(Some("ubuntu:GNOME")));
    assert!(!is_gnome_session(Some("KDE")));
    assert!(!is_gnome_session(None));
}

#[test]
fn adds_gnome_shortcut_path_once() {
    let ours = GNOME_SHORTCUT_PATH;
    assert_eq!(with_custom_keybinding("@as []", ours), Some(format!("['{ours}']")));
    assert_eq!(
        with_custom_keybinding("['/custom0/', '/custom1/']", ours),
        Some(format!("['/custom0/', '/custom1/', '{ours}']"))
    );
    assert_eq!(with_custom_keybinding(&format!("['/custom0/', '{ours}']"), ours), None);
}
