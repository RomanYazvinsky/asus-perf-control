use anyhow::{ensure, Result};

pub const AURA_REPORT_ID: u8 = 0x5d;
pub const ROG_OPCODE: u8 = 4;

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
