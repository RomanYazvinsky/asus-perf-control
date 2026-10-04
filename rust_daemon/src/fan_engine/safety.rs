pub const HIGH_TEMP_THRESHOLD_C: i32 = 80;
pub const MIN_SAFE_SPEED_PERCENT: i32 = 30;

pub fn speed_is_safe_for_temp(temp_c: i32, speed_percent: i32) -> bool {
    temp_c < HIGH_TEMP_THRESHOLD_C || speed_percent >= MIN_SAFE_SPEED_PERCENT
}

pub fn speed_percent_to_pwm(speed_percent: i32) -> u8 {
    let clamped = speed_percent.clamp(0, 100) as f64;
    ((clamped / 100.0) * 255.0).round() as u8
}
