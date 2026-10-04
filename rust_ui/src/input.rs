//! Gamepad navigation (former `GamepadInputScope`).
//!
//! Steam Input commands from `rust_core` are turned into egui key events (focus movement,
//! activate, back); tab switching and curve commands are handed to the app.

use asus_linux_control_core::services::steam_input::SteamInputPoller;
pub use asus_linux_control_core::UiInputCommand;

pub struct InputBridge {
    poller: SteamInputPoller,
}

/// Commands that the app handles itself instead of egui.
#[derive(Default)]
pub struct AppCommands {
    pub next_tab: bool,
    pub previous_tab: bool,
    pub curve: Vec<UiInputCommand>,
}

impl Default for InputBridge {
    fn default() -> Self {
        Self {
            poller: SteamInputPoller::new_stub(),
        }
    }
}

fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> [egui::Event; 2] {
    let event = |pressed| egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    };
    [event(true), event(false)]
}

impl InputBridge {
    /// Injects focus/activation key events into `raw_input`; returns app-level commands.
    pub fn pump(&mut self, raw_input: &mut egui::RawInput) -> AppCommands {
        let mut commands = AppCommands::default();
        for event in self.poller.poll() {
            if !event.pressed {
                continue;
            }
            let shift = egui::Modifiers::SHIFT;
            let none = egui::Modifiers::NONE;
            let keys = match event.command {
                UiInputCommand::FocusDown | UiInputCommand::FocusRight => Some((egui::Key::Tab, none)),
                UiInputCommand::FocusUp | UiInputCommand::FocusLeft => Some((egui::Key::Tab, shift)),
                UiInputCommand::Activate => Some((egui::Key::Enter, none)),
                UiInputCommand::Back => Some((egui::Key::Escape, none)),
                UiInputCommand::NextTab => {
                    commands.next_tab = true;
                    None
                }
                UiInputCommand::PreviousTab => {
                    commands.previous_tab = true;
                    None
                }
                UiInputCommand::Apply => None,
                curve => {
                    commands.curve.push(curve);
                    None
                }
            };
            if let Some((key, modifiers)) = keys {
                raw_input.events.extend(key_event(key, modifiers));
            }
        }
        commands
    }
}
