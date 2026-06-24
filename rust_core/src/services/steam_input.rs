use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInputCommand {
    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,
    Activate,
    Back,
    Apply,
    NextTab,
    PreviousTab,
    CurvePreviousPoint,
    CurveNextPoint,
    CurveSpeedUp,
    CurveSpeedDown,
    CurveTempLeft,
    CurveTempRight,
}

impl UiInputCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FocusUp => "focusUp",
            Self::FocusDown => "focusDown",
            Self::FocusLeft => "focusLeft",
            Self::FocusRight => "focusRight",
            Self::Activate => "activate",
            Self::Back => "back",
            Self::Apply => "apply",
            Self::NextTab => "nextTab",
            Self::PreviousTab => "previousTab",
            Self::CurvePreviousPoint => "curvePreviousPoint",
            Self::CurveNextPoint => "curveNextPoint",
            Self::CurveSpeedUp => "curveSpeedUp",
            Self::CurveSpeedDown => "curveSpeedDown",
            Self::CurveTempLeft => "curveTempLeft",
            Self::CurveTempRight => "curveTempRight",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamInputStatus {
    pub available: bool,
    pub connected_controllers: i32,
    pub active_action_set: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamInputEvent {
    pub command: UiInputCommand,
    pub pressed: bool,
    pub sequence: i64,
}

#[derive(Debug, Default)]
pub struct SteamInputPoller {
    sequence: i64,
}

impl SteamInputPoller {
    pub fn new_stub() -> Self {
        Self { sequence: 0 }
    }

    pub fn status(&self) -> SteamInputStatus {
        SteamInputStatus {
            available: false,
            connected_controllers: 0,
            active_action_set: None,
            reason: Some("Steam Input native adapter is not enabled in this build".to_string()),
        }
    }

    pub fn poll(&mut self) -> Vec<SteamInputEvent> {
        self.sequence += 1;
        Vec::new()
    }
}
