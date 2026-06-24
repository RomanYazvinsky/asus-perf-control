#[path = "../src/services/steam_input.rs"]
mod steam_input;

use steam_input::{SteamInputPoller, UiInputCommand};

#[test]
fn stub_reports_unavailable_without_steamworks() {
    let poller = SteamInputPoller::new_stub();

    let status = poller.status();

    assert!(!status.available);
    assert_eq!(status.connected_controllers, 0);
    assert_eq!(
        status.reason.as_deref(),
        Some("Steam Input native adapter is not enabled in this build")
    );
}

#[test]
fn command_names_are_stable() {
    assert_eq!(UiInputCommand::FocusUp.as_str(), "focusUp");
    assert_eq!(UiInputCommand::FocusDown.as_str(), "focusDown");
    assert_eq!(UiInputCommand::FocusLeft.as_str(), "focusLeft");
    assert_eq!(UiInputCommand::FocusRight.as_str(), "focusRight");
    assert_eq!(UiInputCommand::Activate.as_str(), "activate");
    assert_eq!(UiInputCommand::Back.as_str(), "back");
    assert_eq!(UiInputCommand::Apply.as_str(), "apply");
    assert_eq!(UiInputCommand::NextTab.as_str(), "nextTab");
    assert_eq!(UiInputCommand::PreviousTab.as_str(), "previousTab");
    assert_eq!(
        UiInputCommand::CurvePreviousPoint.as_str(),
        "curvePreviousPoint"
    );
    assert_eq!(UiInputCommand::CurveNextPoint.as_str(), "curveNextPoint");
    assert_eq!(UiInputCommand::CurveSpeedUp.as_str(), "curveSpeedUp");
    assert_eq!(UiInputCommand::CurveSpeedDown.as_str(), "curveSpeedDown");
    assert_eq!(UiInputCommand::CurveTempLeft.as_str(), "curveTempLeft");
    assert_eq!(UiInputCommand::CurveTempRight.as_str(), "curveTempRight");
}
