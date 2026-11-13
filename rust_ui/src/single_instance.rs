//! Single application instance.
//!
//! The ROG hotkey agent launches the app with `gtk-launch asus-linux-control`. Like the former
//! GtkApplication runner, a second launch must raise the running window instead of opening a
//! new one: the first instance listens on a Unix socket, later launches send `activate` and exit.
//! A launch that received an xdg-activation token forwards it (`activate <token>`): only the
//! token lets the running window take focus on Wayland (see [`crate::activation`]).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
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

/// A request to raise the window, with the launcher's activation token if it had one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activation {
    pub token: Option<String>,
}

/// Asks a running instance to raise its window. Returns true if one answered.
pub fn activate_existing(path: &std::path::Path, token: Option<&str>) -> bool {
    let Ok(mut stream) = UnixStream::connect(path) else {
        return false;
    };
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    stream.write_all(activation_request(token).as_bytes()).is_ok()
}

fn activation_request(token: Option<&str>) -> String {
    match token {
        Some(token) => format!("activate {token}\n"),
        None => "activate\n".to_string(),
    }
}

fn parse_activation(line: &str) -> Option<Activation> {
    let rest = line.trim().strip_prefix("activate")?;
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let token = rest.trim();
    Some(Activation {
        token: (!token.is_empty()).then(|| token.to_string()),
    })
}

/// Listener owned by the primary instance; removes the socket on drop.
pub struct PrimaryInstance {
    path: PathBuf,
    activation: Arc<Mutex<Option<Activation>>>,
    ctx: Arc<OnceLock<egui::Context>>,
}

impl PrimaryInstance {
    pub fn listen(path: PathBuf) -> std::io::Result<Self> {
        // A socket nobody answers on is stale (crashed instance).
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        let activation: Arc<Mutex<Option<Activation>>> = Arc::default();
        let ctx: Arc<OnceLock<egui::Context>> = Arc::default();
        let (pending, ctx_slot) = (activation.clone(), ctx.clone());
        std::thread::Builder::new()
            .name("single-instance".into())
            .spawn(move || {
                for stream in listener.incoming().flatten() {
                    let mut line = String::new();
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    if BufReader::new(stream).read_line(&mut line).is_err() {
                        continue;
                    }
                    if let Some(request) = parse_activation(&line) {
                        let mut pending = pending.lock().unwrap_or_else(|e| e.into_inner());
                        // Keep a token from an earlier, not yet handled request.
                        if request.token.is_some() || pending.is_none() {
                            *pending = Some(request);
                        }
                        drop(pending);
                        if let Some(ctx) = ctx_slot.get() {
                            ctx.request_repaint();
                        }
                    }
                }
            })?;
        Ok(Self { path, activation, ctx })
    }

    pub fn set_context(&self, ctx: egui::Context) {
        let _ = self.ctx.set(ctx);
    }

    /// The pending activation request, once.
    pub fn take_activation(&self) -> Option<Activation> {
        self.activation.lock().unwrap_or_else(|e| e.into_inner()).take()
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
        assert!(!activate_existing(&path, None));

        let primary = PrimaryInstance::listen(path.clone()).unwrap();
        assert!(activate_existing(&path, Some("gnome-shell/token_1")));
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let activation = loop {
            if let Some(activation) = primary.take_activation() {
                break activation;
            }
            assert!(std::time::Instant::now() < deadline, "activation not received");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(activation.token.as_deref(), Some("gnome-shell/token_1"));

        drop(primary);
        assert!(!path.exists());
    }

    #[test]
    fn parses_activation_requests() {
        let parse = |token| parse_activation(&activation_request(token));
        assert_eq!(parse(None), Some(Activation { token: None }));
        assert_eq!(parse(Some("abc")), Some(Activation { token: Some("abc".into()) }));
        assert_eq!(parse_activation("activated\n"), None);
        assert_eq!(parse_activation("quit\n"), None);
    }
}
