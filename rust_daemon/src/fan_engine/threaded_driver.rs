use super::config::ValidatedFanEngineConfig;
use super::controller::FanEngineDriver;
use super::runtime::{FanEngineRuntime, FanHardwareAdapter};
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

enum FanEngineCommand {
    Update(ValidatedFanEngineConfig),
    Stop,
}

type HardwareFactory =
    Box<dyn Fn() -> Result<Box<dyn FanHardwareAdapter>, String> + Send + Sync + 'static>;

pub struct ThreadedFanEngineDriver {
    hardware_factory: Arc<HardwareFactory>,
    tx: Option<Sender<FanEngineCommand>>,
    handle: Option<JoinHandle<Result<(), String>>>,
    active: Arc<AtomicBool>,
}

impl ThreadedFanEngineDriver {
    pub fn new(hardware_factory: HardwareFactory) -> Self {
        Self {
            hardware_factory: Arc::new(hardware_factory),
            tx: None,
            handle: None,
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    fn join_existing(&mut self) -> Result<(), String> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(FanEngineCommand::Stop);
        }

        if let Some(handle) = self.handle.take() {
            let result = handle
                .join()
                .map_err(|_| "Fan engine thread panicked".to_string());
            self.active.store(false, Ordering::SeqCst);
            return result?;
        }

        self.active.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl FanEngineDriver for ThreadedFanEngineDriver {
    fn start(&mut self, config: ValidatedFanEngineConfig) -> Result<(), String> {
        self.join_existing()?;

        let hardware = (self.hardware_factory)()?;
        let (tx, rx) = mpsc::channel();
        let active = self.active.clone();
        active.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || run_loop(config, hardware, rx, active));

        self.tx = Some(tx);
        self.handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.join_existing()
    }

    fn update_config(&mut self, config: ValidatedFanEngineConfig) -> Result<(), String> {
        if !self.is_active() {
            return Err("Fan engine thread is not running".to_string());
        }

        let tx = self
            .tx
            .as_ref()
            .ok_or_else(|| "Fan engine thread is not running".to_string())?;

        tx.send(FanEngineCommand::Update(config))
            .map_err(|err| err.to_string())
    }

    fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Drop for ThreadedFanEngineDriver {
    fn drop(&mut self) {
        let _ = self.join_existing();
    }
}

fn run_loop(
    config: ValidatedFanEngineConfig,
    hardware: Box<dyn FanHardwareAdapter>,
    rx: Receiver<FanEngineCommand>,
    active: Arc<AtomicBool>,
) -> Result<(), String> {
    let _active_guard = ActiveGuard { active };
    let mut runtime = FanEngineRuntime::new(config, hardware);
    let mut consecutive_errors = 0usize;

    loop {
        let tick_ms = runtime.config().tick_ms.max(1);

        match rx.recv_timeout(Duration::from_millis(tick_ms)) {
            Ok(FanEngineCommand::Update(config)) => {
                runtime.update_config(config);
                consecutive_errors = 0;
            }
            Ok(FanEngineCommand::Stop) => {
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => match runtime.tick_once() {
                Ok(()) => consecutive_errors = 0,
                Err(err) => {
                    consecutive_errors += 1;
                    error!("Fan engine tick failed: {err}");
                    if consecutive_errors >= 3 {
                        error!("Fan engine stopped after 3 consecutive tick failures");
                        break;
                    }
                }
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    let stop_result = runtime.stop();

    if let Err(err) = &stop_result {
        error!("Failed to restore fan auto mode: {err}");
    } else {
        info!("Fan engine stopped and auto mode restored");
    }

    stop_result
}

struct ActiveGuard {
    active: Arc<AtomicBool>,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.active.store(false, Ordering::SeqCst);
    }
}
