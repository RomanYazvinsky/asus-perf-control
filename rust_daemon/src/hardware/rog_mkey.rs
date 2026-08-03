use std::{thread, time::Duration};

use anyhow::{anyhow, ensure, Context, Result};

pub const AURA_REPORT_ID: u8 = 0x5d;
pub const ROG_OPCODE: u8 = 4;
pub const FEATURE_REPORT_LEN: usize = 65;

const ASUS_VENDOR_ID: u16 = 0x0b05;
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

pub struct FirmwareBindingGuard {
    slot: usize,
    original_opcode: u8,
    changed: bool,
}

impl FirmwareBindingGuard {
    pub fn prepare<T: MKeyTransport>(transport: &mut T, layout: &MKeyLayout) -> Result<Self> {
        let slot = layout
            .rog_slot()
            .ok_or_else(|| anyhow!("ROG slot unavailable"))?;
        let original_opcode = layout
            .rog_opcode()
            .ok_or_else(|| anyhow!("ROG opcode unavailable"))?;
        let changed = original_opcode != ROG_OPCODE;

        if changed {
            transport.send_feature(&set_opcode_request(
                transport.report_len(),
                slot,
                ROG_OPCODE,
            ))?;
        }

        Ok(Self {
            slot,
            original_opcode,
            changed,
        })
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn restore<T: MKeyTransport>(&self, transport: &mut T) -> Result<()> {
        if self.changed {
            transport.send_feature(&set_opcode_request(
                transport.report_len(),
                self.slot,
                self.original_opcode,
            ))?;
        }
        Ok(())
    }
}

pub struct HidMKeyTransport {
    device: hidapi::HidDevice,
}

impl HidMKeyTransport {
    pub fn discover_and_probe() -> Result<(Self, MKeyLayout)> {
        let api = hidapi::HidApi::new().context("failed to initialize hidapi")?;
        let mut failures = Vec::new();
        let mut candidate_count = 0;

        for info in api
            .device_list()
            .filter(|info| info.vendor_id() == ASUS_VENDOR_ID)
        {
            candidate_count += 1;
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
            for _ in 0..PROBE_ATTEMPTS {
                thread::sleep(PROBE_DELAY);
                let mut response = [0; FEATURE_REPORT_LEN];
                response[0] = AURA_REPORT_ID;

                match device.get_feature_report(&mut response) {
                    Ok(response_len) => match parse_layout(&response[..response_len]) {
                        Ok(layout) => return Ok((Self { device }, layout)),
                        Err(error) => last_error = Some(format!("invalid response: {error}")),
                    },
                    Err(error) => last_error = Some(format!("read failed: {error}")),
                }
            }

            failures.push(format!(
                "{candidate}: no valid M-key layout after {PROBE_ATTEMPTS} attempts ({})",
                last_error.unwrap_or_else(|| "no response".to_string())
            ));
        }

        if candidate_count == 0 {
            Err(anyhow!("no ASUS HID devices found"))
        } else {
            Err(anyhow!(
                "no ASUS HID device returned a valid M-key layout: {}",
                failures.join("; ")
            ))
        }
    }
}

impl MKeyTransport for HidMKeyTransport {
    fn report_len(&self) -> usize {
        FEATURE_REPORT_LEN
    }

    fn send_feature(&mut self, report: &[u8]) -> Result<()> {
        self.device
            .send_feature_report(report)
            .context("failed to send ASUS M-key feature report")
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
