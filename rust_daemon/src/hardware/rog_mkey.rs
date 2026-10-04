use std::{collections::HashSet, hash::Hash, thread, time::Duration};

use anyhow::{anyhow, ensure, Context, Result};

pub const AURA_REPORT_ID: u8 = 0x5d;
pub const ROG_OPCODE: u8 = 4;
pub const FEATURE_REPORT_LEN: usize = 65;

const ASUS_VENDOR_ID: u16 = 0x0b05;
const ASUS_MKEY_USAGE_PAGE: u16 = 0xff31;
const ASUS_MKEY_USAGE: u16 = 0x79;
const PROBE_ATTEMPTS: usize = 10;
const PROBE_DELAY: Duration = Duration::from_millis(20);

const LAYOUT_COMMAND: u8 = 0x02;
const SET_OPCODE_COMMAND: u8 = 0x03;

// The ASUS M-key feature-report protocol is adapted from G-Helper's
// MKeyControl.cs by seerge: https://github.com/seerge/g-helper/pull/5691
pub struct MKeyLayout {
    opcodes: Vec<u8>,
}

impl MKeyLayout {
    pub fn rog_slot(&self) -> Option<usize> {
        (self.opcodes.len() > 3).then(|| self.opcodes.len() - 1)
    }

    pub fn rog_opcode(&self) -> Option<u8> {
        self.rog_slot().map(|slot| self.opcodes[slot])
    }
}

pub trait MKeyTransport {
    fn report_len(&self) -> usize;
    fn send_feature(&mut self, report: &[u8]) -> Result<()>;
}

pub struct FirmwareBindingGuard<T: MKeyTransport> {
    transport: T,
    slot: usize,
    original_opcode: u8,
    active: bool,
}

impl<T: MKeyTransport> FirmwareBindingGuard<T> {
    pub fn prepare(transport: T, layout: &MKeyLayout) -> Result<Self> {
        ensure!(
            transport.report_len() == FEATURE_REPORT_LEN,
            "invalid M-key feature report length: expected {FEATURE_REPORT_LEN}, got {}",
            transport.report_len()
        );
        let slot = layout
            .rog_slot()
            .ok_or_else(|| anyhow!("ROG slot unavailable"))?;
        let original_opcode = layout
            .rog_opcode()
            .ok_or_else(|| anyhow!("ROG opcode unavailable"))?;
        let active = original_opcode != ROG_OPCODE;
        let mut guard = Self {
            transport,
            slot,
            original_opcode,
            active,
        };

        if guard.active {
            let request = set_opcode_request(FEATURE_REPORT_LEN, slot, ROG_OPCODE);
            if let Err(prepare_error) = guard.transport.send_feature(&request) {
                return match guard.restore() {
                    Ok(()) => Err(anyhow!(
                        "failed to prepare ROG firmware binding: {prepare_error:#}; original opcode restored"
                    )),
                    Err(restore_error) => Err(anyhow!(
                        "failed to prepare ROG firmware binding: {prepare_error:#}; failed to restore original opcode: {restore_error:#}"
                    )),
                };
            }
        }

        Ok(guard)
    }

    pub fn changed(&self) -> bool {
        self.active
    }

    pub fn restore(&mut self) -> Result<()> {
        if self.active {
            self.transport.send_feature(&set_opcode_request(
                FEATURE_REPORT_LEN,
                self.slot,
                self.original_opcode,
            ))?;
            self.active = false;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: MKeyTransport> Drop for FirmwareBindingGuard<T> {
    fn drop(&mut self) {
        if self.active {
            if let Err(error) = self.restore() {
                log::error!("failed to restore original ROG firmware binding on drop: {error:#}");
            }
        }
    }
}

pub(crate) struct ProbedCandidate<T> {
    pub(crate) identity: String,
    pub(crate) transport: T,
    pub(crate) layout: MKeyLayout,
}

pub(crate) fn select_capable_candidate<T>(
    candidates: Vec<ProbedCandidate<T>>,
) -> Result<ProbedCandidate<T>> {
    let mut capable = Vec::new();
    let mut mismatches = Vec::new();

    for candidate in candidates {
        if candidate.layout.rog_slot().is_some() {
            capable.push(candidate);
        } else {
            mismatches.push(format!("{}: layout has no ROG slot", candidate.identity));
        }
    }

    match capable.len() {
        1 => Ok(capable.remove(0)),
        0 => Err(anyhow!(
            "no capable ASUS M-key HID candidate found{}",
            if mismatches.is_empty() {
                String::new()
            } else {
                format!(": {}", mismatches.join("; "))
            }
        )),
        _ => Err(anyhow!(
            "ambiguous ASUS M-key HID candidates: {}",
            capable
                .iter()
                .map(|candidate| candidate.identity.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )),
    }
}

pub(crate) fn finalize_discovery<T>(
    candidates: Vec<ProbedCandidate<T>>,
    failures: Vec<String>,
) -> Result<ProbedCandidate<T>> {
    if !failures.is_empty() {
        let capable_identities = candidates
            .iter()
            .filter(|candidate| candidate.layout.rog_slot().is_some())
            .map(|candidate| candidate.identity.as_str())
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "incomplete ASUS M-key discovery; capable candidates: {}; failures: {}",
            if capable_identities.is_empty() {
                "none".to_string()
            } else {
                capable_identities.join("; ")
            },
            failures.join("; ")
        ));
    }

    select_capable_candidate(candidates)
}

pub(crate) fn deduplicate_by_path<T, K, F>(candidates: Vec<T>, path: F) -> Vec<T>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(path(candidate)))
        .collect()
}

pub(crate) fn feature_report_context(identity: &str) -> String {
    format!("failed to send ASUS M-key feature report to {identity}")
}

pub struct HidMKeyTransport {
    device: hidapi::HidDevice,
    identity: String,
}

impl HidMKeyTransport {
    pub fn discover_and_probe() -> Result<(Self, MKeyLayout)> {
        let api = hidapi::HidApi::new().context("failed to initialize hidapi")?;
        let mut failures = Vec::new();
        let mut probed = Vec::new();

        let matching_candidates = api
            .device_list()
            .filter(|info| {
                info.vendor_id() == ASUS_VENDOR_ID
                    && info.usage_page() == ASUS_MKEY_USAGE_PAGE
                    && info.usage() == ASUS_MKEY_USAGE
            })
            .cloned()
            .collect();
        let unique_candidates =
            deduplicate_by_path(matching_candidates, |info| info.path().to_bytes().to_vec());

        for info in unique_candidates {
            let candidate = format!(
                "ASUS HID {:04x}:{:04x} at {}",
                info.vendor_id(),
                info.product_id(),
                info.path().to_string_lossy()
            );
            let device = match info.open_device(&api) {
                Ok(device) => device,
                Err(error) => {
                    failures.push(format!("{candidate}: open failed: {error}"));
                    continue;
                }
            };

            if let Err(error) = device.send_feature_report(&probe_request(FEATURE_REPORT_LEN)) {
                failures.push(format!("{candidate}: probe request failed: {error}"));
                continue;
            }

            let mut last_error = None;
            let mut layout = None;
            for _ in 0..PROBE_ATTEMPTS {
                thread::sleep(PROBE_DELAY);
                let mut response = [0; FEATURE_REPORT_LEN];
                response[0] = AURA_REPORT_ID;

                match device.get_feature_report(&mut response) {
                    Ok(response_len) => match parse_layout(&response[..response_len]) {
                        Ok(parsed) => {
                            layout = Some(parsed);
                            break;
                        }
                        Err(error) => last_error = Some(format!("invalid response: {error}")),
                    },
                    Err(error) => last_error = Some(format!("read failed: {error}")),
                }
            }

            match layout {
                Some(layout) => probed.push(ProbedCandidate {
                    identity: candidate.clone(),
                    transport: Self {
                        device,
                        identity: candidate,
                    },
                    layout,
                }),
                None => failures.push(format!(
                    "{candidate}: no valid M-key layout after {PROBE_ATTEMPTS} attempts ({})",
                    last_error.unwrap_or_else(|| "no response".to_string())
                )),
            }
        }

        let selected = finalize_discovery(probed, failures)?;

        Ok((selected.transport, selected.layout))
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

impl MKeyTransport for HidMKeyTransport {
    fn report_len(&self) -> usize {
        FEATURE_REPORT_LEN
    }

    fn send_feature(&mut self, report: &[u8]) -> Result<()> {
        self.device
            .send_feature_report(report)
            .with_context(|| feature_report_context(&self.identity))
    }
}

pub fn probe_request(report_len: usize) -> Vec<u8> {
    assert!(
        report_len >= 4,
        "M-key probe report must be at least 4 bytes"
    );
    let mut request = vec![0; report_len];
    request[..4].copy_from_slice(&[AURA_REPORT_ID, 0x9f, LAYOUT_COMMAND, 0]);
    request
}

pub fn set_opcode_request(report_len: usize, slot: usize, opcode: u8) -> Vec<u8> {
    assert!(
        report_len >= 6,
        "M-key set-opcode report must be at least 6 bytes"
    );
    assert!(slot < 8, "M-key slot must be less than 8");
    let mut request = vec![0; report_len];
    request[..6].copy_from_slice(&[
        AURA_REPORT_ID,
        0x9f,
        SET_OPCODE_COMMAND,
        0x01,
        slot as u8,
        opcode,
    ]);
    request
}

pub fn parse_layout(response: &[u8]) -> Result<MKeyLayout> {
    ensure!(
        response.get(..3) == Some(&[AURA_REPORT_ID, 0x9f, LAYOUT_COMMAND]),
        "invalid M-key layout response header"
    );
    ensure!(response.len() >= 5, "truncated M-key layout response");

    let slot_count = response[4] as usize;
    ensure!(
        (1..=8).contains(&slot_count),
        "invalid M-key slot count: {slot_count}"
    );
    ensure!(
        response.len() >= 5 + slot_count,
        "truncated M-key layout response"
    );

    Ok(MKeyLayout {
        opcodes: response[5..5 + slot_count].to_vec(),
    })
}
