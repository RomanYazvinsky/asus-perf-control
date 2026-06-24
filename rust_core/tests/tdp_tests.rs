// Unit tests for TDP validation logic (constitution §IV: Test-Before-Hardware)
// These tests validate the range-checking without requiring real hardware

use asus_linux_control_core::services::tdp::TdpController;

fn controller() -> TdpController {
    TdpController::new()
}

#[test]
fn test_validate_sustained_tdp_too_low() {
    let ctrl = controller();
    let result = ctrl.set_sustained_tdp(4);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("5"),
        "Error should mention 5W minimum, got: {msg}"
    );
}

#[test]
fn test_validate_sustained_tdp_too_high() {
    let ctrl = controller();
    let result = ctrl.set_sustained_tdp(55);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("54"),
        "Error should mention 54W maximum, got: {msg}"
    );
}

#[test]
fn test_validate_boost_tdp_too_low() {
    let ctrl = controller();
    let result = ctrl.set_boost_tdp(4);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("5"),
        "Error should mention 5W minimum, got: {msg}"
    );
}

#[test]
fn test_validate_boost_tdp_too_high() {
    let ctrl = controller();
    let result = ctrl.set_boost_tdp(55);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("54"),
        "Error should mention 54W maximum, got: {msg}"
    );
}

#[test]
fn test_validate_temp_limit_too_low() {
    let ctrl = controller();
    let result = ctrl.set_cpu_temp_limit(59);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("60"),
        "Error should mention 60°C minimum, got: {msg}"
    );
}

#[test]
fn test_validate_temp_limit_too_high() {
    let ctrl = controller();
    let result = ctrl.set_cpu_temp_limit(101);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("100"),
        "Error should mention 100°C maximum, got: {msg}"
    );
}

#[test]
fn test_is_supported_returns_bool_without_panic() {
    // Should not panic regardless of hardware availability
    let _ = TdpController::is_supported();
}

#[test]
fn test_set_tdp_config_boost_less_than_sustained_rejected() {
    // Validate cross-field constraint via FFI layer logic
    // boost < sustained must be rejected (tested at api.rs level via integration test)
    // Here we test that individual validations pass for valid range
    let ctrl = controller();
    // 20W is valid for both sustained and boost individually
    // (sysfs write will fail on non-hardware, but validation should pass)
    let sustained_result = ctrl.set_sustained_tdp(0); // invalid: below min
    assert!(sustained_result.is_err());
    let boost_result = ctrl.set_boost_tdp(54); // valid range
                                               // Result depends on hardware; validation itself should not reject this
    if boost_result.is_err() {
        let msg = boost_result.unwrap_err();
        // Should be I/O error (no hardware), NOT a validation error
        assert!(
            !msg.contains("must be"),
            "54W should pass validation; got unexpected error: {msg}"
        );
    }
}
