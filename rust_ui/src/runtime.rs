//! Background async runtime.
//!
//! The UI thread never awaits: it spawns a job, keeps the returned [`Task`] and polls it
//! every frame. When a job finishes it wakes the UI with `request_repaint`.

use std::future::Future;
use std::sync::mpsc;

pub struct Runtime {
    rt: tokio::runtime::Runtime,
    ctx: egui::Context,
}

impl Runtime {
    pub fn new(ctx: egui::Context) -> std::io::Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("asus-ui-worker")
            .build()?;
        Ok(Self { rt, ctx })
    }

    pub fn spawn<T, F>(&self, fut: F) -> Task<T>
    where
        T: Send + 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let ctx = self.ctx.clone();
        self.rt.spawn(async move {
            let value = fut.await;
            let _ = tx.send(value);
            ctx.request_repaint();
        });
        Task { rx }
    }

    /// Fire-and-forget job whose result is only logged.
    pub fn spawn_detached<F>(&self, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.rt.spawn(fut);
    }

    pub fn repaint_after(&self, delay: std::time::Duration) {
        self.ctx.request_repaint_after(delay);
    }

    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.rt.block_on(fut)
    }
}

/// Handle to a job running on the [`Runtime`].
pub struct Task<T> {
    rx: mpsc::Receiver<T>,
}

impl<T> Task<T> {
    /// Returns the result once, as soon as it is available.
    pub fn poll(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

/// Poll an optional task slot; clears the slot when the result arrives.
pub fn take_ready<T>(slot: &mut Option<Task<T>>) -> Option<T> {
    let result = slot.as_mut()?.poll();
    if result.is_some() {
        *slot = None;
    }
    result
}
