#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Value types and the rejection vocabulary (SDD-swtd3w), tested here so the
//! `verifies:` annotations sit under `test_paths`, which this unit's config
//! reads only from `org-node/tests`.

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::sequence::SeqGuard;
use org_node::SequenceNumber;

// verifies: REQ-gju89b, LLR-7gnrnz
#[test]
fn org_id_is_twenty_bytes_and_round_trips_through_postcard() {
    let id = OrgId::new([7u8; 20]);
    assert_eq!(id.as_bytes().len(), 20);
    let bytes = postcard::to_allocvec(&id).unwrap();
    let back: OrgId = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(id, back);
}

// verifies: REQ-gju89b, LLR-7gnrnz
#[test]
fn org_ids_differing_in_one_byte_are_not_equal() {
    let mut raw = [7u8; 20];
    let first = OrgId::new(raw);
    raw[19] = 8;
    assert_ne!(first, OrgId::new(raw));
}

// verifies: LLR-gu6u53
#[test]
fn org_id_debug_is_the_twenty_bytes_in_order_as_forty_lowercase_hex_digits() {
    // The fixture bytes are all DIFFERENT, so a reversal of the iteration in
    // `OrgId`'s Debug is visible; bytes all alike could not see byte order.
    let mut raw = [0u8; 20];
    for (index, byte) in raw.iter_mut().enumerate() {
        *byte = (index as u8) + 0xa0;
    }
    let rendered = format!("{:?}", OrgId::new(raw));

    let expected: String = raw.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(rendered, format!("OrgId(0x{expected})"));

    let inner = rendered.trim_start_matches("OrgId(0x").trim_end_matches(')');
    assert_eq!(inner.len(), 40, "twenty bytes is forty digits");
    assert!(inner.chars().all(|digit| digit.is_ascii_hexdigit() && !digit.is_ascii_uppercase()));
    // First and last byte land at the ends, so a reversal is visible.
    assert!(inner.starts_with("a0"), "first byte must come first: {inner}");
    assert!(inner.ends_with("b3"), "last byte must come last: {inner}");
}

// LLR-3sddey states the variants this lists.
// verifies: REQ-9g6as6, LLR-z8fubr, LLR-3sddey
#[test]
fn every_rejection_variant_is_distinct_from_every_other() {
    let all = [
        OrgNodeError::OrgIdMismatch,
        // `InvalidOrgPublicKey` is the chain-state refusal of REQ-8jb4ny.
        OrgNodeError::InvalidOrgPublicKey,
        OrgNodeError::StaleSeq { got: 1, last_seen: 2 },
        OrgNodeError::MalformedDelta,
        OrgNodeError::DeltaBaseMismatch,
        OrgNodeError::OrgNotOnChain,
        OrgNodeError::RootMismatch,
        OrgNodeError::StaleEpoch { got: 1, last: 2 },
        OrgNodeError::Chain("read failed".into()),
        OrgNodeError::Trie(org_members::OrgMembersError::DuplicateHandle),
        // The parent sequence that is not the next epoch after the held
        // record, refused on receipt (LLR-76bhw9).
        OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 },
        // The two first-admission refusals (REQ-8amu2a, REQ-kt877x).
        OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([1; 20]) },
        OrgNodeError::AdmissionNotOurs { org_id: OrgId::new([1; 20]) },
        // The bound on one Organisation's provisional updates (REQ-fwfku9).
        OrgNodeError::ProvisionalLimit { limit: 1 },
        // The three refusals of a received Wire message LLR-3sddey keeps.
        OrgNodeError::MalformedMessage,
        OrgNodeError::RevocationNotHeld { org_id: OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotForThisDevice { org_id: OrgId::new([1; 20]) },
    ];
    for (first_index, first) in all.iter().enumerate() {
        for (second_index, second) in all.iter().enumerate() {
            if first_index != second_index {
                assert_ne!(first, second, "variants {first_index} and {second_index} compare equal");
            }
        }
    }
}

// Normal: each first-admission refusal names its Organisation; abnormal: two
// Organisations' refusals, and the two refusals of one, are not the same error.
// verifies: REQ-8amu2a, REQ-kt877x
#[test]
fn the_first_admission_refusals_name_their_organisation() {
    let org = OrgId::new([0xa0; 20]);
    let unexpected = OrgNodeError::AdmissionNotExpected { org_id: org };
    let not_ours = OrgNodeError::AdmissionNotOurs { org_id: org };
    for err in [&unexpected, &not_ours] {
        assert!(format!("{err}").contains(&format!("{org:?}")), "{err}");
    }
    assert_ne!(unexpected, OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([0xa1; 20]) });
    assert_ne!(unexpected, not_ours);
}

// Normal: the refusal names the limit in bytes and what it bounds — the
// epoch state's provisional changes and organisations, not provisional
// updates; abnormal: two limits differ.
// verifies: LLR-3sddey, REQ-g2537c
#[test]
fn the_provisional_limit_refusal_names_the_limit_and_the_epoch_state() {
    let err = OrgNodeError::ProvisionalLimit { limit: 1 << 20 };
    let text = err.to_string();
    assert!(text.contains("1048576"), "names the limit: {text}");
    assert!(text.contains("provisional changes and organisations"), "names the epoch state: {text}");
    assert!(!text.contains("provisional updates"), "names no provisional update: {text}");
    assert_ne!(err, OrgNodeError::ProvisionalLimit { limit: 1 });
}

// Normal: the malformed-message refusal says what it refuses, and the other
// two name their Organisation. Abnormal: two Organisations' refusals differ,
// the two of one Organisation are distinct in value and in message, and
// the vocabulary holds no `OrgKeyMismatch` (no operation raises it).
// verifies: LLR-3sddey
#[test]
fn the_received_message_refusals_say_what_they_refuse_without_org_key_mismatch() {
    assert_eq!(OrgNodeError::MalformedMessage.to_string(), "received wire message is malformed");
    let org = OrgId::new([0xa0; 20]);
    let refusals =
        [OrgNodeError::RevocationNotHeld { org_id: org }, OrgNodeError::RevocationNotForThisDevice { org_id: org }];
    for err in &refusals {
        assert!(err.to_string().contains(&format!("{org:?}")), "{err}");
    }
    assert_ne!(refusals[0], OrgNodeError::RevocationNotHeld { org_id: OrgId::new([0xa1; 20]) });
    assert_ne!(refusals[0], refusals[1]);
    assert_ne!(refusals[0].to_string(), refusals[1].to_string());
    let source = include_str!("../src/error.rs");
    assert!(!source.contains("OrgKeyMismatch"), "error.rs names OrgKeyMismatch");
}

/// A DevicePublicKey for the recipient-carrying variant.
fn some_device() -> org_members::DevicePublicKey {
    org_node::keys::SigningKeypair::generate(&mut rand::rngs::OsRng).device_key().unwrap()
}

/// The invite identifier the invite-carrying variants name.
fn some_invite() -> org_node::ids::InviteId {
    org_node::ids::InviteId::new([0xc0; 32])
}

/// The variants LLR-3sddey states, each with a value where it has one.
fn stated_variants(recipient: org_members::DevicePublicKey) -> Vec<OrgNodeError> {
    let org_id = OrgId::new([0xa0; 20]);
    let member_id = org_members::MemberId::new([0xb0; 32]);
    vec![
        OrgNodeError::NoProvisionalChange,
        OrgNodeError::OrganisationBehind { org_id, chain_epoch: org_node::Epoch::new(7) },
        OrgNodeError::NoAdmissionRequestKept { invite_id: some_invite() },
        OrgNodeError::AdmissionAlreadyRecorded { invite_id: some_invite() },
        OrgNodeError::NoFreeHandle,
        OrgNodeError::HandleTaken { handle: org_members::Handle::parse("zed").unwrap() },
        OrgNodeError::MemberNotHeld { member_id },
        OrgNodeError::RevocationAlreadyRecorded { member_id },
        OrgNodeError::NoProvisionalFounding,
        OrgNodeError::FoundingProxyAlreadyRecorded,
        OrgNodeError::FoundingNotWritten,
        OrgNodeError::NoProvisionalMember,
        OrgNodeError::OrganisationNotRequested { org_id },
        OrgNodeError::PersonaNotHeld { device_key: Box::new(recipient) },
        OrgNodeError::PersonaIdNotHeld { persona_id: org_node::types::PersonaId::new("nobody".into()) },
        OrgNodeError::AdmissionKeyNotSender { org_id },
        OrgNodeError::OrganisationRemoved { org_id },
        OrgNodeError::AcknowledgementNotKept { recipient: Box::new(recipient) },
        OrgNodeError::DuplicateOrgPublicKey,
    ]
}

// Normal: the nineteen variants LLR-3sddey states are distinct from each
// other in value and in message, and from the other variants; each that
// carries an Organisation, a MemberId, an epoch, an invite identifier, a
// Device key, a Persona identifier or a recipient names it in its
// `Display`, and `HandleTaken` carries its Handle and does not show it.
// `DuplicateOrgPublicKey` is org-node's own refusal of a drawn Organisation
// key. Abnormal: the same variant over two Organisations differs.
// verifies: LLR-3sddey
#[test]
fn the_stated_variants_are_distinct_and_their_display_names_the_value() {
    let org_id = OrgId::new([0xa0; 20]);
    let member_id = org_members::MemberId::new([0xb0; 32]);
    let recipient = some_device();
    let stated = stated_variants(recipient);
    assert_eq!(stated.len(), 19);
    for (index, first) in stated.iter().enumerate() {
        for second in stated.iter().skip(index + 1) {
            assert_ne!(first, second);
            assert_ne!(first.to_string(), second.to_string());
        }
        for kept in [
            OrgNodeError::MalformedMessage,
            OrgNodeError::OrgNotHeld { org_id },
            OrgNodeError::InviteNotOutstanding { org_id, invite_id: some_invite() },
            OrgNodeError::Trie(org_members::OrgMembersError::DuplicateKey),
            OrgNodeError::DeviceSecretNotSupplied { org_id },
            OrgNodeError::SenderNotListed { org_id },
        ] {
            assert_ne!(*first, kept);
            assert_ne!(first.to_string(), kept.to_string());
        }
    }
    for err in &stated {
        let text = err.to_string();
        match err {
            OrgNodeError::OrganisationBehind { .. } => {
                assert!(text.contains(&format!("{org_id:?}")) && text.contains("epoch 7"), "{text}")
            }
            OrgNodeError::OrganisationNotRequested { .. }
            | OrgNodeError::AdmissionKeyNotSender { .. }
            | OrgNodeError::OrganisationRemoved { .. } => assert!(text.contains(&format!("{org_id:?}")), "{text}"),
            OrgNodeError::MemberNotHeld { .. } | OrgNodeError::RevocationAlreadyRecorded { .. } => {
                assert!(text.contains(&format!("{member_id:?}")), "{text}")
            }
            OrgNodeError::AcknowledgementNotKept { .. } | OrgNodeError::PersonaNotHeld { .. } => {
                assert!(text.contains(&format!("{recipient:?}")), "{text}")
            }
            OrgNodeError::NoAdmissionRequestKept { .. } | OrgNodeError::AdmissionAlreadyRecorded { .. } => {
                assert!(text.contains(&format!("{:?}", some_invite())), "{text}")
            }
            OrgNodeError::PersonaIdNotHeld { .. } => assert!(text.contains("nobody"), "{text}"),
            OrgNodeError::HandleTaken { handle } => {
                assert_eq!(handle.as_str(), "zed");
                assert!(!text.contains(handle.as_str()), "{text}")
            }
            _ => assert!(!text.is_empty()),
        }
    }
    assert_ne!(
        OrgNodeError::OrganisationRemoved { org_id },
        OrgNodeError::OrganisationRemoved { org_id: OrgId::new([0xa1; 20]) }
    );
}

// Normal: the bound's refusal reads exactly as LLR-3sddey states it.
// verifies: LLR-3sddey
#[test]
fn the_provisional_limit_display_names_changes_and_organisations() {
    assert_eq!(
        OrgNodeError::ProvisionalLimit { limit: 1 << 20 }.to_string(),
        "provisional changes and organisations for one Organisation would exceed 1048576 bytes"
    );
}

// Normal: the refusal of a received Change set whose parent sequence is not
// the next epoch after the held record says so, offered number first.
// Abnormal: it names no chain, since receipt reads none (merge review MR2-N2),
// and two exchanged numbers read differently.
// verifies: LLR-76bhw9
#[test]
fn the_seq_not_epoch_refusal_names_the_next_epoch_after_the_held_record_and_no_chain() {
    let text = OrgNodeError::SeqNotEpoch { seq: 4, epoch: 3 }.to_string();
    assert_eq!(text, "the Change set's parent sequence 4 is not 3, the next epoch after the held record");
    assert!(!text.contains("chain"), "receipt reads no chain: {text}");
    assert_ne!(text, OrgNodeError::SeqNotEpoch { seq: 3, epoch: 4 }.to_string());
}

// verifies: REQ-9g6as6, LLR-z8fubr
#[test]
fn a_stale_sequence_rejection_carries_the_offered_number_and_the_mark() {
    // The error comes from a real producer, and the test pins the ORDER of
    // the two numbers in the message, which is the only part a reader relies
    // on: a check that both merely appear could not see them exchanged.
    let guard = SeqGuard::from_last_seen(SequenceNumber::new(9));
    let err = guard.check(SequenceNumber::new(4)).unwrap_err();
    assert_eq!(err, OrgNodeError::StaleSeq { got: 4, last_seen: 9 });

    let rendered = format!("{err}");
    let at_offered = rendered.find('4').expect("the offered number must be rendered");
    let at_mark = rendered.find('9').expect("the mark must be rendered");
    assert!(
        at_offered < at_mark,
        "the message must name the offered number before the mark it was judged against: {rendered}"
    );
}

// verifies: REQ-9g6as6, LLR-7cgg8a
#[test]
fn a_provider_error_is_carried_into_the_trie_variant_without_loss() {
    let provider = org_members::OrgMembersError::InvalidHandle("bad handle".into());
    let carried: OrgNodeError = provider.clone().into();
    match carried {
        OrgNodeError::Trie(inner) => assert_eq!(inner, provider),
        other => panic!("wrong variant: {other:?}"),
    }
}

// ---- the replay guard (sequence.rs, SDD-d8ktxa) ----------------------------
//
// Moved here 2026-10-30 from `verify_against_chain.rs`, deleted with
// `verify_envelope_against_chain` (merge review MR1-N1 of change
// worktree-org-io-domain-api). The receive path builds its guard with
// `from_last_seen` and asks it with `check` (`service::keep_information`).

// verifies: REQ-6yu72z, LLR-wx3php
#[test]
fn rejects_equal_and_lower_seq() {
    let guard = SeqGuard::from_last_seen(SequenceNumber::new(5));
    assert!(guard.check(SequenceNumber::new(6)).is_ok());
    assert_eq!(guard.check(SequenceNumber::new(5)), Err(OrgNodeError::StaleSeq { got: 5, last_seen: 5 }));
    assert_eq!(guard.check(SequenceNumber::new(4)), Err(OrgNodeError::StaleSeq { got: 4, last_seen: 5 }));
}

// verifies: REQ-mr5abb, LLR-duwz79
#[test]
fn from_last_seen_starts_the_guard_at_the_given_mark() {
    for mark in [0u64, 1, 7, u64::MAX - 1] {
        let guard = SeqGuard::from_last_seen(SequenceNumber::new(mark));
        assert_eq!(guard.last_seen(), SequenceNumber::new(mark), "last_seen reports the mark the guard was started at");
        assert_eq!(guard.check(SequenceNumber::new(mark)), Err(OrgNodeError::StaleSeq { got: mark, last_seen: mark }));
        assert!(guard.check(SequenceNumber::new(mark + 1)).is_ok());
    }
}

// verifies: REQ-mr5abb, LLR-f5kq88
#[test]
fn check_does_not_advance_the_mark() {
    // `check` takes `&self`: a rejection (or an acceptance that is later
    // rejected downstream) can never move the high-water mark. The assertion
    // documents the contract the type system enforces.
    let guard = SeqGuard::from_last_seen(SequenceNumber::new(5));
    guard.check(SequenceNumber::new(6)).unwrap();
    assert_eq!(guard.last_seen(), SequenceNumber::new(5));
}
