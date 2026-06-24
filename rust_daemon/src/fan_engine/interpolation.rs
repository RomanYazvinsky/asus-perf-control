use super::config::ValidatedCurvePoint;

pub fn interpolate_pwm(curve: &[ValidatedCurvePoint], temp_c: i32) -> u8 {
    if curve.is_empty() {
        return 0;
    }

    if temp_c <= curve[0].temp_c {
        return curve[0].pwm;
    }

    if let Some(last) = curve.last() {
        if temp_c >= last.temp_c {
            return last.pwm;
        }
    }

    for i in 0..curve.len() - 1 {
        let left = &curve[i];
        let right = &curve[i + 1];
        if temp_c >= left.temp_c && temp_c < right.temp_c {
            let temp_span = (right.temp_c - left.temp_c) as f64;
            if temp_span <= 0.0 {
                return right.pwm;
            }

            let ratio = (temp_c - left.temp_c) as f64 / temp_span;
            let pwm = left.pwm as f64 + ratio * (right.pwm as f64 - left.pwm as f64);
            return pwm.round() as u8;
        }
    }

    curve[curve.len() - 1].pwm
}
