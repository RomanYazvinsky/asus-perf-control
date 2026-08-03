#![allow(dead_code)]

#[path = "../src/hotkey_agent.rs"]
mod hotkey_agent;

use futures_util::stream;
use hotkey_agent::{activate_app, process_signals, CommandRunner, ProcessCommandRunner};

#[derive(Default)]
struct FakeRunner {
    calls: Vec<(String, Vec<String>)>,
    failures_remaining: usize,
}

impl CommandRunner for FakeRunner {
    fn run(&mut self, program: &str, args: &[&str]) -> anyhow::Result<()> {
        self.calls.push((
            program.to_string(),
            args.iter().map(|arg| arg.to_string()).collect(),
        ));

        if self.failures_remaining > 0 {
            self.failures_remaining -= 1;
            anyhow::bail!("injected launch failure");
        }

        Ok(())
    }
}

#[test]
fn activate_app_invokes_the_installed_desktop_app_exactly_once() {
    let mut runner = FakeRunner::default();

    activate_app(&mut runner).unwrap();

    assert_eq!(
        runner.calls,
        vec![(
            "gtk-launch".to_string(),
            vec!["asus-linux-control".to_string()]
        )]
    );
}

#[test]
fn production_runner_rejects_a_nonzero_exit_status() {
    let mut runner = ProcessCommandRunner;

    let error = runner.run("sh", &["-c", "exit 7"]).unwrap_err();

    assert!(error.to_string().contains("status"));
}

#[tokio::test]
async fn signal_loop_continues_after_an_activation_failure() {
    let signals = stream::iter([(), ()]);
    let mut runner = FakeRunner {
        failures_remaining: 1,
        ..FakeRunner::default()
    };

    process_signals(signals, &mut runner).await;

    assert_eq!(runner.calls.len(), 2);
    assert!(runner.calls.iter().all(|call| {
        call == &(
            "gtk-launch".to_string(),
            vec!["asus-linux-control".to_string()],
        )
    }));
}
