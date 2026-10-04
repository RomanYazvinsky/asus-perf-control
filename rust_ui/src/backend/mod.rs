//! D-Bus clients for asusd, supergfxd and our privileged daemon.

pub mod asusd;
pub mod bus;
pub mod daemon;
pub mod supergfx;

use std::sync::Arc;

pub use asusd::AsusdClient;
pub use bus::SystemBus;
pub use daemon::DaemonClient;
pub use supergfx::SupergfxClient;

/// All backend clients over one shared system-bus connection.
pub struct Backend {
    pub asusd: AsusdClient,
    pub daemon: DaemonClient,
    pub supergfx: SupergfxClient,
}

impl Backend {
    pub fn new() -> Self {
        let bus = Arc::new(SystemBus::new());
        Self {
            asusd: AsusdClient::new(bus.clone()),
            daemon: DaemonClient::new(bus.clone()),
            supergfx: SupergfxClient::new(bus),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
