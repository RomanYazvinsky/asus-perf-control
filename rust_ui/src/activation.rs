//! Bringing the window to the foreground on Wayland.
//!
//! Compositors with focus-stealing prevention (GNOME) ignore a plain "focus me" request and only
//! show an "is ready" notification. A window is raised only when it presents an
//! `xdg_activation_v1` token the compositor issued for a user action — e.g. the token GNOME passes
//! in `XDG_ACTIVATION_TOKEN` to a command started from a keyboard shortcut. winit has no API to
//! activate an existing window with a foreign token, so this talks to the compositor directly on
//! the window's own Wayland connection.

use anyhow::{anyhow, Context, Result};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::backend::{Backend, ObjectId};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_registry, wl_surface::WlSurface};
use wayland_client::{delegate_noop, Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

pub const TOKEN_ENV: &str = "XDG_ACTIVATION_TOKEN";

/// Takes the activation token this process was started with, so child processes don't reuse it.
pub fn take_startup_token() -> Option<String> {
    let token = std::env::var(TOKEN_ENV).ok().filter(|t| !t.trim().is_empty());
    // Called from `main` before any other thread is started.
    std::env::remove_var(TOKEN_ENV);
    token
}

/// Activates the window of `frame` with `token`. Errors on non-Wayland sessions.
pub fn activate(frame: &eframe::Frame, token: &str) -> Result<()> {
    let display = match frame.display_handle().map_err(|e| anyhow!("{e}"))?.as_raw() {
        RawDisplayHandle::Wayland(handle) => handle.display,
        _ => anyhow::bail!("not a Wayland session"),
    };
    let surface = match frame.window_handle().map_err(|e| anyhow!("{e}"))?.as_raw() {
        RawWindowHandle::Wayland(handle) => handle.surface,
        _ => anyhow::bail!("not a Wayland window"),
    };

    // SAFETY: the display outlives this call (it belongs to the running event loop), and the
    // guest backend is dropped before returning.
    let backend = unsafe { Backend::from_foreign_display(display.as_ptr().cast()) };
    let conn = Connection::from_backend(backend);
    let (globals, mut queue) = registry_queue_init::<State>(&conn).context("Wayland registry")?;
    let activation: XdgActivationV1 = globals
        .bind(&queue.handle(), 1..=1, ())
        .context("compositor does not support xdg_activation_v1")?;
    // SAFETY: the surface pointer comes from the live window of this frame.
    let surface_id = unsafe { ObjectId::from_ptr(WlSurface::interface(), surface.as_ptr().cast()) }
        .context("invalid wl_surface")?;
    let surface = WlSurface::from_id(&conn, surface_id).context("invalid wl_surface")?;

    activation.activate(token.to_string(), &surface);
    activation.destroy();
    queue.roundtrip(&mut State).context("Wayland roundtrip")?;
    Ok(())
}

struct State;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(State: ignore XdgActivationV1);
