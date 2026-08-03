#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use hardware::rog_mkey::{
    parse_layout, probe_request, set_opcode_request, FirmwareBindingGuard, MKeyTransport,
    AURA_REPORT_ID, ROG_OPCODE,
};

#[derive(Default)]
struct FakeTransport {
    writes: Vec<Vec<u8>>,
}

impl MKeyTransport for FakeTransport {
    fn report_len(&self) -> usize {
        65
    }

    fn send_feature(&mut self, report: &[u8]) -> anyhow::Result<()> {
        self.writes.push(report.to_vec());
        Ok(())
    }
}

#[test]
fn parses_five_slot_layout_with_rog_key_in_last_slot() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 5, 2, 3, 1, 14, 4])
        .expect("valid M-key layout should parse");

    assert_eq!(layout.rog_slot(), Some(4));
    assert_eq!(layout.rog_opcode(), Some(4));
}

#[test]
fn rejects_slot_count_above_protocol_limit() {
    assert!(parse_layout(&[0x5d, 0x9f, 0x02, 0, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9]).is_err());
}

#[test]
fn rejects_truncated_layout_response() {
    assert!(parse_layout(&[0x5d, 0x9f, 0x02, 0, 5, 2, 3, 1]).is_err());
}

#[test]
fn rejects_zero_slot_layout() {
    assert!(parse_layout(&[0x5d, 0x9f, 0x02, 0, 0]).is_err());
}

#[test]
fn rejects_invalid_layout_header() {
    assert!(parse_layout(&[0x5d, 0x9f, 0x03, 0, 1, 4]).is_err());
}

#[test]
fn rog_slot_requires_more_than_three_slots() {
    let three_slots = parse_layout(&[0x5d, 0x9f, 0x02, 0, 3, 1, 2, 4]).unwrap();
    let four_slots = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 1, 2, 3, 4]).unwrap();

    assert_eq!(three_slots.rog_slot(), None);
    assert_eq!(three_slots.rog_opcode(), None);
    assert_eq!(four_slots.rog_slot(), Some(3));
    assert_eq!(four_slots.rog_opcode(), Some(4));
}

#[test]
fn accepts_eight_slot_layout() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    assert_eq!(layout.rog_slot(), Some(7));
    assert_eq!(layout.rog_opcode(), Some(8));
}

#[test]
fn exposes_protocol_constants() {
    assert_eq!(AURA_REPORT_ID, 0x5d);
    assert_eq!(ROG_OPCODE, 4);
}

#[test]
fn builds_zero_padded_probe_request() {
    let request = probe_request(8);

    assert_eq!(&request[..4], &[0x5d, 0x9f, 0x02, 0]);
    assert_eq!(request, vec![0x5d, 0x9f, 0x02, 0, 0, 0, 0, 0]);
}

#[test]
#[should_panic(expected = "M-key probe report must be at least 4 bytes")]
fn rejects_short_probe_report() {
    probe_request(3);
}

#[test]
fn builds_zero_padded_set_opcode_request() {
    let request = set_opcode_request(8, 4, 4);

    assert_eq!(&request[..6], &[0x5d, 0x9f, 0x03, 0x01, 4, 4]);
    assert_eq!(request, vec![0x5d, 0x9f, 0x03, 0x01, 4, 4, 0, 0]);
}

#[test]
#[should_panic(expected = "M-key set-opcode report must be at least 6 bytes")]
fn rejects_short_set_opcode_report() {
    set_opcode_request(5, 4, 4);
}

#[test]
#[should_panic(expected = "M-key slot must be less than 8")]
fn rejects_out_of_range_set_opcode_slot() {
    set_opcode_request(8, 8, 4);
}

#[test]
fn production_source_attributes_g_helper_protocol() {
    let source = include_str!("../src/hardware/rog_mkey.rs");

    assert!(source.contains("https://github.com/seerge/g-helper/pull/5691"));
    assert!(source.contains("seerge"));
}

#[test]
fn firmware_guard_prepares_and_restores_only_the_last_slot() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let mut transport = FakeTransport::default();
    let guard = FirmwareBindingGuard::prepare(&mut transport, &layout).unwrap();

    assert!(guard.changed());
    assert_eq!(transport.writes, vec![set_opcode_request(65, 3, 4)]);

    guard.restore(&mut transport).unwrap();

    assert_eq!(
        transport.writes,
        vec![set_opcode_request(65, 3, 4), set_opcode_request(65, 3, 14),]
    );
}

#[test]
fn firmware_guard_does_not_write_when_rog_opcode_is_already_bound() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 4]).unwrap();
    let mut transport = FakeTransport::default();
    let guard = FirmwareBindingGuard::prepare(&mut transport, &layout).unwrap();

    assert!(!guard.changed());
    guard.restore(&mut transport).unwrap();

    assert!(transport.writes.is_empty());
}

#[test]
fn firmware_guard_rejects_layout_without_a_rog_slot_without_writing() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 3, 2, 3, 14]).unwrap();
    let mut transport = FakeTransport::default();

    assert!(FirmwareBindingGuard::prepare(&mut transport, &layout).is_err());
    assert!(transport.writes.is_empty());
}
