//! Structured backend logging, same line format as the former Dart `BackendLogger`:
//! `[Backend] <STATUS> <operation> key=value ... elapsedMs=N error=...`.

use std::fmt::Display;
use std::future::Future;
use std::time::Instant;

pub type Fields<'a> = &'a [(&'a str, String)];

fn format_line(status: &str, operation: &str, fields: Fields<'_>) -> String {
    let mut line = format!("[Backend] {status} {operation}");
    for (key, value) in fields {
        line.push_str(&format!(" {key}={value}"));
    }
    line
}

/// Log a one-off event (e.g. a settings mismatch).
pub fn log_event(operation: &str, fields: Fields<'_>) {
    log::info!("{}", format_line("EVENT", operation, fields));
}

/// Log `Settings.hardwareMismatch` for a desired value that differs from hardware.
pub fn log_hardware_mismatch(key: &str, expected: impl Display, actual: impl Display) {
    log_event(
        "Settings.hardwareMismatch",
        &[
            ("key", key.to_string()),
            ("expected", expected.to_string()),
            ("actual", actual.to_string()),
        ],
    );
}

/// Log `Settings.readMismatch` for a persisted value that could not be used.
pub fn log_read_mismatch(key: &str, actual: impl Display) {
    log_event(
        "Settings.readMismatch",
        &[("key", key.to_string()), ("actual", actual.to_string())],
    );
}

/// Trace an async backend operation: START, then OK or ERROR with elapsed time.
pub async fn trace<T, E, F>(operation: &str, fields: Fields<'_>, fut: F) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    E: Display,
{
    log::debug!("{}", format_line("START", operation, fields));
    let started = Instant::now();
    let result = fut.await;
    let elapsed = started.elapsed().as_millis();
    match &result {
        Ok(_) => log::debug!(
            "{} elapsedMs={elapsed}",
            format_line("OK", operation, fields)
        ),
        Err(error) => log::warn!(
            "{} elapsedMs={elapsed} error={error}",
            format_line("ERROR", operation, fields)
        ),
    }
    result
}
