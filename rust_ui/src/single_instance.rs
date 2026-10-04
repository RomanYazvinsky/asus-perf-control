//! Single application instance.
//!
//! The ROG hotkey agent launches the app with `gtk-launch asus-linux-control`. Like the former
//! GtkApplication runner, a second launch must raise the running window instead of opening a
//! new one: the first instance listens on a Unix socket, later launches send `activate` and exit.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub fn socket_path() -> PathBuf {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir).join("asus-linux-control.sock"),
        _ => {
            let uid = std::fs::metadata("/proc/self").map(|m| std::os::unix::fs::MetadataExt::uid(&m)).unwrap_or(0);
            std::env::temp_dir().join(format!("asus-linux-control-{uid}.sock"))
        }
    }
}

/// Asks a running instance to raise its window. Returns true if one answered.
pub fn activate_existing(path: &std::path::Path) -> bool {
    let Ok(mut stream) = UnixStream::connect(path) else {
        return false;
    };
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    stream.write_all(b"activate\n").is_ok()
}

/// Listener owned by the primary instance; removes the socket on drop.
pub struct PrimaryInstance {
    path: PathBuf,
    activate: Arc<AtomicBool>,
    ctx: Arc<OnceLock<egui::Context>>,
}

impl PrimaryInstance {
    pub fn listen(path: PathBuf) -> std::io::Result<Self> {
        // A socket nobody answers on is stale (crashed instance).
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        let activate = Arc::new(AtomicBool::new(false));
        let ctx: Arc<OnceLock<egui::Context>> = Arc::default();
        let (flag, ctx_slot) = (activate.clone(), ctx.clone());
        std::thread::Builder::new()
            .name("single-instance".into())
            .spawn(move || {
                for stream in listener.incoming().flatten() {
                    let mut line = String::new();
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    if BufReader::new(stream).read_line(&mut line).is_ok() && line.trim() == "activate" {
                        flag.store(true, Ordering::SeqCst);
                        if let Some(ctx) = ctx_slot.get() {
                            ctx.request_repaint();
                        }
                    }
                }
            })?;
        Ok(Self { path, activate, ctx })
    }

    pub fn set_context(&self, ctx: egui::Context) {
        let _ = self.ctx.set(ctx);
    }

    /// True once per activation request.
    pub fn take_activation(&self) -> bool {
        self.activate.swap(false, Ordering::SeqCst)
    }
}

impl Drop for PrimaryInstance {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_launch_activates_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.sock");
        assert!(!activate_existing(&path));

        let primary = PrimaryInstance::listen(path.clone()).unwrap();
        assert!(activate_existing(&path));
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !primary.take_activation() {
            assert!(std::time::Instant::now() < deadline, "activation not received");
            std::thread::sleep(Duration::from_millis(10));
        }

        drop(primary);
        assert!(!path.exists());
    }
}
