use super::commands::{command_for_backend, PackageCommandError, PackageCommandKind};
use super::parser::classify_error;
use super::status::{PackageBackend, UpdateErrorCode, UpdateState, UpdateStatus};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

const LOG_TAIL_LIMIT: usize = 20;

#[derive(Debug, Clone)]
struct UpdateJob {
    id: String,
    backend: PackageBackend,
    status: UpdateStatus,
}

#[derive(Debug, Clone)]
pub struct UpdateJobManager {
    active: Arc<Mutex<Option<UpdateJob>>>,
}

impl UpdateJobManager {
    pub fn new() -> Self {
        Self {
            active: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_update(&self, backend: PackageBackend) -> Result<String, PackageCommandError> {
        let command = command_for_backend(backend, PackageCommandKind::Update)?;
        let update_id = self.create_running_job(backend);
        let active = Arc::clone(&self.active);
        let id = update_id.clone();

        tokio::spawn(async move {
            let output = Command::new(command.executable)
                .args(command.args)
                .output()
                .await;
            let status = match output {
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let combined = format!("{stdout}\n{stderr}");
                    completion_status(backend, output.status.code().unwrap_or(1), &combined)
                }
                Err(e) => failed_status(
                    backend,
                    UpdateErrorCode::PackageManagerUnavailable,
                    "Package manager is unavailable",
                    &[e.to_string()],
                ),
            };
            set_completed_status(&active, &id, status);
        });

        Ok(update_id)
    }

    #[cfg(test)]
    pub fn start_update_for_test(&self, backend: PackageBackend) -> String {
        self.create_running_job(backend)
    }

    pub fn get_status(&self, update_id: &str) -> Option<UpdateStatus> {
        self.active
            .lock()
            .ok()
            .and_then(|active| active.as_ref().cloned())
            .and_then(|job| {
                if job.id == update_id {
                    Some(job.status)
                } else {
                    None
                }
            })
    }

    pub fn cancel_update(&self, update_id: &str) -> bool {
        let Ok(mut active) = self.active.lock() else {
            return false;
        };
        let Some(job) = active.as_mut() else {
            return false;
        };
        if job.id != update_id || job.status.state != UpdateState::Running {
            return false;
        }

        job.status = failed_status(
            job.backend,
            UpdateErrorCode::UpdateFailed,
            "Package update canceled",
            &[],
        );
        true
    }

    #[cfg(test)]
    pub fn complete_for_test(&self, update_id: &str, exit_code: i32, output: &str) {
        let backend = {
            let Ok(active) = self.active.lock() else {
                return;
            };
            let Some(job) = active.as_ref() else {
                return;
            };
            if job.id != update_id {
                return;
            }
            job.backend
        };
        set_completed_status(
            &self.active,
            update_id,
            completion_status(backend, exit_code, output),
        );
    }

    fn create_running_job(&self, backend: PackageBackend) -> String {
        let mut active = self
            .active
            .lock()
            .expect("package update job mutex should not be poisoned");
        if let Some(job) = active.as_ref() {
            if job.status.state == UpdateState::Running {
                return job.id.clone();
            }
        }

        let id = new_update_id();
        let status = UpdateStatus {
            backend,
            state: UpdateState::Running,
            installed_version: None,
            available_version: None,
            reboot_required: Some(false),
            summary: "Package update is running".to_string(),
            error_code: None,
            log_tail: Vec::new(),
        };
        *active = Some(UpdateJob {
            id: id.clone(),
            backend,
            status,
        });
        id
    }
}

impl Default for UpdateJobManager {
    fn default() -> Self {
        Self::new()
    }
}

fn set_completed_status(
    active: &Arc<Mutex<Option<UpdateJob>>>,
    update_id: &str,
    status: UpdateStatus,
) {
    let Ok(mut active) = active.lock() else {
        return;
    };
    let Some(job) = active.as_mut() else {
        return;
    };
    if job.id == update_id && job.status.state == UpdateState::Running {
        job.status = status;
    }
}

fn completion_status(backend: PackageBackend, exit_code: i32, output: &str) -> UpdateStatus {
    if exit_code == 0 {
        return UpdateStatus {
            backend,
            state: UpdateState::Succeeded,
            installed_version: None,
            available_version: None,
            reboot_required: Some(backend == PackageBackend::BazziteRpmOstree),
            summary: if backend == PackageBackend::BazziteRpmOstree {
                "Package update staged; reboot required".to_string()
            } else {
                "Package update completed".to_string()
            },
            error_code: None,
            log_tail: tail_lines(output),
        };
    }

    failed_status(
        backend,
        classify_error(output).unwrap_or(UpdateErrorCode::UpdateFailed),
        "Package update failed",
        &tail_lines(output),
    )
}

fn failed_status(
    backend: PackageBackend,
    error_code: UpdateErrorCode,
    summary: &str,
    log_tail: &[String],
) -> UpdateStatus {
    UpdateStatus {
        backend,
        state: UpdateState::Failed,
        installed_version: None,
        available_version: None,
        reboot_required: None,
        summary: summary.to_string(),
        error_code: Some(error_code),
        log_tail: log_tail.to_vec(),
    }
}

fn tail_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .rev()
        .take(LOG_TAIL_LIMIT)
        .map(str::to_string)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn new_update_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("pkg-update-{nanos}")
}
