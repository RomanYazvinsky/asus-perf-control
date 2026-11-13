//! ASUS Linux Control desktop application entry point.

use asus_linux_control_ui::activation;
use asus_linux_control_ui::app::AsusApp;
use asus_linux_control_ui::single_instance::{activate_existing, socket_path, PrimaryInstance};

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,zbus=warn,tracing=warn")).init();

    let activation_token = activation::take_startup_token();
    let socket = socket_path();
    if activate_existing(&socket, activation_token.as_deref()) {
        log::info!("ASUS Linux Control is already running; raised the existing window");
        return Ok(());
    }
    let instance = match PrimaryInstance::listen(socket) {
        Ok(instance) => Some(instance),
        Err(e) => {
            log::warn!("single-instance socket unavailable: {e}");
            None
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ASUS Linux Control")
            .with_app_id("asus-linux-control")
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ASUS Linux Control",
        options,
        Box::new(|cc| Ok(Box::new(AsusApp::new(cc, instance, activation_token)?))),
    )
}
