#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use hardware::rog_mkey::{
    deduplicate_by_path, feature_report_context, finalize_discovery, parse_layout, probe_request,
    select_capable_candidate, set_opcode_request, FirmwareBindingGuard, MKeyTransport,
    ProbedCandidate, AURA_REPORT_ID, FEATURE_REPORT_LEN, ROG_OPCODE,
};

#[derive(Default)]
struct FakeState {
    writes: Vec<Vec<u8>>,
    failures: VecDeque<&'static str>,
}

#[derive(Clone)]
struct FakeTransport {
    report_len: usize,
    state: Rc<RefCell<FakeState>>,
}

impl Default for FakeTransport {
    fn default() -> Self {
        Self {
            report_len: FEATURE_REPORT_LEN,
            state: Rc::new(RefCell::new(FakeState::default())),
        }
    }
}

impl FakeTransport {
    fn with_failures(failures: impl IntoIterator<Item = &'static str>) -> Self {
        let transport = Self::default();
        transport.state.borrow_mut().failures.extend(failures);
        transport
    }

    fn with_report_len(report_len: usize) -> Self {
        Self {
            report_len,
            ..Self::default()
        }
    }

    fn writes(&self) -> Vec<Vec<u8>> {
        self.state.borrow().writes.clone()
    }
}

impl MKeyTransport for FakeTransport {
    fn report_len(&self) -> usize {
        self.report_len
    }

    fn send_feature(&mut self, report: &[u8]) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        state.writes.push(report.to_vec());
        match state.failures.pop_front() {
            Some(message) => Err(anyhow::anyhow!(message)),
            None => Ok(()),
        }
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
    let transport = FakeTransport::default();
    let mut guard = FirmwareBindingGuard::prepare(transport, &layout).unwrap();

    assert!(guard.changed());
    assert_eq!(
        guard.transport().writes(),
        vec![set_opcode_request(FEATURE_REPORT_LEN, 3, 4)]
    );

    guard.restore().unwrap();

    assert_eq!(
        guard.transport().writes(),
        vec![
            set_opcode_request(FEATURE_REPORT_LEN, 3, 4),
            set_opcode_request(FEATURE_REPORT_LEN, 3, 14),
        ]
    );
    assert!(!guard.changed());
}

#[test]
fn firmware_guard_does_not_write_when_rog_opcode_is_already_bound() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 4]).unwrap();
    let transport = FakeTransport::default();
    let mut guard = FirmwareBindingGuard::prepare(transport, &layout).unwrap();

    assert!(!guard.changed());
    guard.restore().unwrap();

    assert!(guard.transport().writes().is_empty());
}

#[test]
fn firmware_guard_rejects_layout_without_a_rog_slot_without_writing() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 3, 2, 3, 14]).unwrap();
    let transport = FakeTransport::default();
    let observer = transport.clone();

    assert!(FirmwareBindingGuard::prepare(transport, &layout).is_err());
    assert!(observer.writes().is_empty());
}

#[test]
fn repeated_restore_is_a_no_op() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let mut guard = FirmwareBindingGuard::prepare(FakeTransport::default(), &layout).unwrap();

    guard.restore().unwrap();
    guard.restore().unwrap();

    assert_eq!(guard.transport().writes().len(), 2);
}

#[test]
fn drop_restores_an_active_binding_on_the_owned_transport() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let transport = FakeTransport::default();
    let observer = transport.clone();

    {
        let guard = FirmwareBindingGuard::prepare(transport, &layout).unwrap();
        assert!(guard.changed());
    }

    assert_eq!(
        observer.writes(),
        vec![
            set_opcode_request(FEATURE_REPORT_LEN, 3, ROG_OPCODE),
            set_opcode_request(FEATURE_REPORT_LEN, 3, 14),
        ]
    );
}

#[test]
fn failed_prepare_write_immediately_restores_the_original_opcode() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let transport = FakeTransport::with_failures(["prepare write failed"]);
    let observer = transport.clone();

    let error = FirmwareBindingGuard::prepare(transport, &layout)
        .err()
        .expect("prepare must fail");

    assert!(error.to_string().contains("prepare write failed"));
    assert_eq!(
        observer.writes(),
        vec![
            set_opcode_request(FEATURE_REPORT_LEN, 3, ROG_OPCODE),
            set_opcode_request(FEATURE_REPORT_LEN, 3, 14),
        ]
    );
}

#[test]
fn failed_prepare_and_restore_report_both_errors() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let transport = FakeTransport::with_failures(["prepare write failed", "restore write failed"]);

    let error = FirmwareBindingGuard::prepare(transport, &layout)
        .err()
        .expect("prepare must fail")
        .to_string();

    assert!(error.contains("prepare write failed"));
    assert!(error.contains("restore write failed"));
}

#[test]
fn invalid_transport_report_length_is_rejected_without_writing() {
    let layout = parse_layout(&[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let transport = FakeTransport::with_report_len(64);
    let observer = transport.clone();

    let error = FirmwareBindingGuard::prepare(transport, &layout)
        .err()
        .expect("invalid report length must fail");

    assert!(error.to_string().contains("65"));
    assert!(observer.writes().is_empty());
}

fn probed_candidate(identity: &str, response: &[u8]) -> ProbedCandidate<()> {
    ProbedCandidate {
        identity: identity.to_string(),
        transport: (),
        layout: parse_layout(response).unwrap(),
    }
}

#[test]
fn capable_selection_ignores_layouts_without_a_rog_slot() {
    let incapable = probed_candidate("three-slot", &[0x5d, 0x9f, 0x02, 0, 3, 2, 3, 14]);
    let capable = probed_candidate("four-slot", &[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]);

    let selected = select_capable_candidate(vec![incapable, capable]).unwrap();

    assert_eq!(selected.identity, "four-slot");
}

#[test]
fn capable_selection_rejects_ambiguous_devices() {
    let first = probed_candidate("/dev/hidraw1", &[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]);
    let second = probed_candidate("/dev/hidraw2", &[0x5d, 0x9f, 0x02, 0, 5, 2, 3, 1, 14, 4]);

    let error = select_capable_candidate(vec![first, second])
        .err()
        .expect("ambiguous candidates must fail")
        .to_string();

    assert!(error.contains("ambiguous"));
    assert!(error.contains("/dev/hidraw1"));
    assert!(error.contains("/dev/hidraw2"));
}

#[test]
fn discovery_finalization_fails_closed_when_any_candidate_probe_failed() {
    let capable = probed_candidate("/dev/hidraw1", &[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]);
    let failures = vec!["/dev/hidraw2: probe request failed: permission denied".to_string()];

    let error = finalize_discovery(vec![capable], failures)
        .err()
        .expect("incomplete discovery must fail")
        .to_string();

    assert!(error.contains("/dev/hidraw1"));
    assert!(error.contains("/dev/hidraw2"));
    assert!(error.contains("permission denied"));
}

#[test]
fn discovery_finalization_accepts_one_capable_candidate_without_failures() {
    let capable = probed_candidate("/dev/hidraw1", &[0x5d, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]);

    let selected = finalize_discovery(vec![capable], Vec::new()).unwrap();

    assert_eq!(selected.identity, "/dev/hidraw1");
}

#[test]
fn discovery_finalization_retains_failures_when_no_candidate_is_capable() {
    let failures = vec!["/dev/hidraw2: read failed: timeout".to_string()];

    let error = finalize_discovery::<()>(Vec::new(), failures)
        .err()
        .expect("failed discovery must retain diagnostics")
        .to_string();

    assert!(error.contains("/dev/hidraw2"));
    assert!(error.contains("timeout"));
}

#[test]
fn exact_usage_candidates_are_deduplicated_by_hid_path() {
    let candidates = vec![
        ("first record", "/dev/hidraw1"),
        ("duplicate record", "/dev/hidraw1"),
        ("other device", "/dev/hidraw2"),
    ];

    let unique = deduplicate_by_path(candidates, |candidate| candidate.1.to_string());

    assert_eq!(
        unique,
        vec![
            ("first record", "/dev/hidraw1"),
            ("other device", "/dev/hidraw2"),
        ]
    );
}

#[test]
fn feature_report_error_context_identifies_the_hid_device() {
    let context = feature_report_context("ASUS HID 0b05:1abe at /dev/hidraw4");

    assert!(context.contains("0b05:1abe"));
    assert!(context.contains("/dev/hidraw4"));
}
