#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use hardware::rog_mkey::{parse_layout, probe_request, set_opcode_request};

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
fn builds_zero_padded_probe_request() {
    let request = probe_request(8);

    assert_eq!(&request[..4], &[0x5d, 0x9f, 0x02, 0]);
    assert_eq!(request, vec![0x5d, 0x9f, 0x02, 0, 0, 0, 0, 0]);
}

#[test]
fn builds_zero_padded_set_opcode_request() {
    let request = set_opcode_request(8, 4, 4);

    assert_eq!(&request[..6], &[0x5d, 0x9f, 0x03, 0x01, 4, 4]);
    assert_eq!(request, vec![0x5d, 0x9f, 0x03, 0x01, 4, 4, 0, 0]);
}

#[test]
fn production_source_attributes_g_helper_protocol() {
    let source = include_str!("../src/hardware/rog_mkey.rs");

    assert!(source.contains("https://github.com/seerge/g-helper/pull/5691"));
    assert!(source.contains("seerge"));
}
