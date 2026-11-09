#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! No secret reaches debug output (REQ-y7tsft): a Persona record,
//! an Organisation record, the store plaintext and a Wire message holding a
//! member seed, a device seed or an Organisation private key, and a signing key
//! pair, render none of its bytes in any form (LLR-bwb9pu), and the store encryption key renders as
//! its redaction marker (LLR-scgk5j).
//!
//! The store holds one current record and the epoch state beside it
//! (LLR-f5wrjx); the "many records" case holds the second Organisation
//! private key in a provisional organisation.

mod support;

use org_members::{Handle, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::revocation::{Acknowledgement, Signature64};
use org_node::store::{
    self, OrgRecord, OrgRecordParts, Origin, PersonaRecord, PersonaRecordParts, PersonaStatus, ProvisionalOrganisation,
    StoreData,
};
use org_node::test_fixtures::{
    admin_device, admit_member_delta, bob_absence_notice, device_keypair, device_secret, genesis_trie, org_public_key,
};
use org_node::wire::WireMessage;
use org_node::{Envelope, Epoch, MemberSeed, OrgPrivateKey, SequenceNumber};
use support::fixtures::{persona_parts, record_parts};
use support::{assert_not_rendered, squash};

/// The Organisation private key every `org` record holds (REQ-ech45n):
/// distinctive, so a rendering of it is caught.
fn org_private() -> [u8; 32] {
    sentinel(0x80)
}

/// 32 bytes counting up from `start`, so each secret is distinctive.
fn sentinel(start: u8) -> [u8; 32] {
    core::array::from_fn(|index| start.wrapping_add(index as u8))
}

/// 32 bytes counting down from `start`.
fn sentinel_down(start: u8) -> [u8; 32] {
    core::array::from_fn(|index| start.wrapping_sub(index as u8))
}

/// An Active Persona of the member seed `member`. The record holds the
/// Device public key of `device`, never the device seed (LLR-ay8nvu).
fn persona(member: [u8; 32], device: [u8; 32]) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        status: PersonaStatus::Active,
        ..persona_parts("p1", member, device_secret(device).device_key())
    })
}

/// A record holding the Organisation private key `private`.
fn record_holding(private: [u8; 32]) -> OrgRecord {
    OrgRecord::from(OrgRecordParts {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: org_public_key(),
        last_seq: SequenceNumber::new(0),
        org_private_key: OrgPrivateKey::from(private),
        ..record_parts(1, &[])
    })
}

/// A provisional organisation holding the Organisation private key `private`.
fn provisional(private: [u8; 32]) -> ProvisionalOrganisation {
    ProvisionalOrganisation::from(store::ProvisionalOrganisationParts {
        epoch: Epoch::new(2),
        base_root: RootHash::new([0x11u8; 32]),
        members: vec![],
        root: RootHash::new([0x12u8; 32]),
        org_pub_key: org_public_key(),
        org_private_key: OrgPrivateKey::from(private),
        change_set: None,
        base_snapshot: None,
        origin: Origin::Own,
        applied: vec![],
        first_ahead_at: None,
    })
}

fn wire(private: [u8; 32]) -> WireMessage {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let envelope = Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(1), &delta).unwrap();
    WireMessage::OrgInformation { envelope, record_snapshot: vec![], org_private_key: OrgPrivateKey::from(private) }
}

// The Organisation record also holds the Organisation private key, which
// renders as its redaction marker and none of its bytes (LLR-2dvhz8).
/// verifies: LLR-bwb9pu, LLR-2dvhz8
#[test]
fn records_and_wire_messages_never_render_secret_bytes() {
    let (member, device, private) = (sentinel(0xd0), sentinel(0x10), org_private());
    let record_of_persona = persona(member, device);
    let record = record_holding(private);
    let store_data = StoreData { record: Some(record.clone()), ..StoreData::new(record_of_persona.clone()) };
    let message = wire(private);
    for rendered in [
        format!("{record_of_persona:?}"),
        format!("{record_of_persona:#?}"),
        format!("{store_data:?}"),
        format!("{store_data:#?}"),
    ] {
        assert_not_rendered(&rendered, &member, "member seed");
        assert_not_rendered(&rendered, &device, "device seed");
        // No device seed is held, so none is rendered, redacted or not; the
        // Member seed renders as its marker.
        assert!(rendered.contains("MemberSeed([REDACTED])"));
    }
    for rendered in [
        format!("{record:?}"),
        format!("{record:#?}"),
        format!("{store_data:?}"),
        format!("{store_data:#?}"),
        format!("{message:?}"),
        format!("{message:#?}"),
    ] {
        assert_not_rendered(&rendered, &private, "Organisation private key");
        assert!(rendered.contains("OrgPrivateKey([REDACTED])"), "{rendered}");
    }
    for rendered in [format!("{record:?}"), format!("{record:#?}"), format!("{store_data:?}"), format!("{store_data:#?}")] {
        assert!(rendered.contains("org_private_key"));
    }
}

/// A revocation notice and an acknowledgement for Bob's Device, which the
/// genesis trie does not list (neither holds an Envelope).
fn revocation_and_acknowledgement() -> [WireMessage; 2] {
    let notice = bob_absence_notice(OrgId::new([5u8; 20]));
    let trie = genesis_trie(&MemberSeed::from([1u8; 32]).x25519_keypair(), &admin_device());
    let acknowledgement = Acknowledgement {
        org_id: notice.org_id,
        member_id: notice.member_id,
        device: notice.device,
        epoch: Epoch::new(2),
        root: trie.root_hash().unwrap(),
        signature: Signature64([0x40; 64]),
    };
    [WireMessage::Revocation(notice), WireMessage::Acknowledgement(acknowledgement)]
}

// LLR-ecxc76: Organisation information renders its key as the redaction
// marker and none of its bytes; a revocation or an acknowledgement holds no
// key and renders none.
/// verifies: LLR-ecxc76, LLR-bwb9pu
#[test]
fn a_wire_message_of_either_kind_never_renders_the_key() {
    for private in [sentinel(0x40), sentinel_down(0xff)] {
        let info = wire(private);
        for rendered in [format!("{info:?}"), format!("{info:#?}")] {
            assert_not_rendered(&rendered, &private, "Organisation private key");
            assert!(rendered.contains("OrgPrivateKey([REDACTED])"), "{rendered}");
        }
        for keyless in revocation_and_acknowledgement() {
            for rendered in [format!("{keyless:?}"), format!("{keyless:#?}")] {
                assert_not_rendered(&rendered, &private, "Organisation private key");
                assert!(!rendered.contains("OrgPrivateKey"), "holds no key: {rendered}");
            }
        }
    }
}

// LLR-2dvhz8: the field is required, so a record renders it as its redacted
// secret type, never its bytes, whatever they are.
/// verifies: LLR-2dvhz8
#[test]
fn a_record_debug_renders_the_organisation_private_key_redacted() {
    for private in [org_private(), [0u8; 32], sentinel_down(0xff)] {
        let record = record_holding(private);
        for rendered in [format!("{record:?}"), format!("{record:#?}")] {
            let squashed = squash(&rendered);
            assert!(squashed.contains("org_private_key:OrgPrivateKey([REDACTED])"), "{rendered}");
            if private != [0u8; 32] {
                assert_not_rendered(&rendered, &private, "Organisation private key");
            }
        }
    }
}

/// verifies: LLR-bwb9pu
#[test]
fn secrets_at_the_high_byte_bound_and_many_records_stay_unrendered() {
    // A store holds one Persona, so the many records are many stores' data,
    // rendered together.
    let personas: Vec<PersonaRecord> =
        (0..3u8).map(|index| persona(sentinel_down(0xff - index), sentinel_down(0xef - index))).collect();
    let store_data: Vec<StoreData> = personas
        .iter()
        .map(|persona| StoreData {
            record: Some(record_holding(sentinel_down(0xdf))),
            provisional_organisations: vec![provisional(sentinel_down(0xcf))],
            ..StoreData::new(persona.clone())
        })
        .collect();
    for rendered in [format!("{store_data:?}"), format!("{store_data:#?}")] {
        for index in 0..personas.len() as u8 {
            assert_not_rendered(&rendered, &sentinel_down(0xff - index), "member seed");
            assert_not_rendered(&rendered, &sentinel_down(0xef - index), "device seed");
        }
        assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation private key");
        assert_not_rendered(&rendered, &sentinel_down(0xcf), "Organisation private key");
    }
}

/// `SigningKeypair` derives `Debug` over ed25519-dalek's `SigningKey`, whose
/// own `Debug` omits the secret (soup.md, ed25519-dalek); this pins that.
/// A member seed yields an `X25519Keypair`, whose hand-written `Debug`
/// renders none of its bytes either (LLR-98ufry).
/// verifies: LLR-bwb9pu, LLR-98ufry
#[test]
fn a_signing_key_pair_never_renders_its_seed() {
    for seed in [sentinel(0x60), sentinel_down(0xff)] {
        let device = device_keypair(seed);
        for rendered in [format!("{device:?}"), format!("{device:#?}")] {
            assert_not_rendered(&rendered, &seed, "device key pair seed");
        }
        let member = MemberSeed::from(seed).x25519_keypair();
        for rendered in [format!("{member:?}"), format!("{member:#?}")] {
            assert_not_rendered(&rendered, &seed, "member key pair seed");
        }
    }
}

/// The Organisation private key, alone and as its key pair, renders none of
/// its bytes, at both byte bounds.
/// verifies: LLR-bwb9pu, LLR-322xfu
#[test]
fn the_organisation_private_key_and_its_key_pair_never_render_its_bytes() {
    for secret in [sentinel(0x90), sentinel_down(0xff), [0u8; 32]] {
        let key = OrgPrivateKey::from(secret);
        let pair = key.x25519_keypair();
        for rendered in [format!("{key:?}"), format!("{key:#?}"), format!("{pair:?}"), format!("{pair:#?}")] {
            if secret != [0u8; 32] {
                assert_not_rendered(&rendered, &secret, "Organisation private key");
            }
            assert!(rendered == "OrgPrivateKey([REDACTED])" || rendered == "X25519Keypair(..)", "{rendered}");
        }
    }
}

// The store key is derived from the device secret by HKDF (LLR-2qqqx6); the
// probe takes the secret.
/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_as_its_marker() {
    assert_eq!(store::store_key_debug_for_test(&device_secret([0x5a; 32])).unwrap(), "StoreKey([REDACTED])");
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_the_same_for_every_passphrase() {
    for seed in [[0u8; 32], [0xff; 32], sentinel(0x20)] {
        assert_eq!(store::store_key_debug_for_test(&device_secret(seed)).unwrap(), "StoreKey([REDACTED])");
    }
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_has_no_display_and_is_not_copy() {
    let (display, copy) = store::store_key_traits_for_test();
    assert!(!display, "StoreKey must not implement Display");
    assert!(!copy, "StoreKey must not be Copy");
}

// Normal: the `Debug` rendering of a Wire message of every one of the six
// kinds holds no byte of an Organisation private key in any form; the two
// kinds that carry one render it as `OrgPrivateKey([REDACTED])`.
// verifies: LLR-4ces44
#[test]
fn no_kind_debug_shows_an_organisation_private_key_byte() {
    // A mid-range key, and keys holding the low and the high byte bound.
    let mut low = sentinel(0x9b);
    low[0] = 0x00;
    for private in [org_private(), low, sentinel_down(0xff)] {
        assert_no_kind_renders(private);
    }
}

/// Every kind of Wire message, the two that carry one holding `private`:
/// none renders a byte of it under `Debug`, and only those two name the
/// redacted key.
fn assert_no_kind_renders(private: [u8; 32]) {
    let org_id = OrgId::new([5u8; 20]);
    let kinds = [
        wire(private),
        WireMessage::Revocation(bob_absence_notice(org_id)),
        WireMessage::Acknowledgement(Acknowledgement {
            org_id,
            member_id: org_members::MemberId::new([2u8; 32]),
            device: admin_device().device_key().unwrap(),
            epoch: Epoch::new(1),
            root: RootHash::new([0x11u8; 32]),
            signature: Signature64([0x22; 64]),
        }),
        WireMessage::AdmissionRequest {
            org_id,
            invite_id: org_node::ids::InviteId::new([0x61; 32]),
            name: Name::parse("Carol").unwrap(),
            surname: Surname::parse("Coder").unwrap(),
            preferred_handles: vec![Handle::parse("carol").unwrap()],
            member_key: MemberSeed::from([0x36u8; 32]).x25519_keypair().member_key().unwrap(),
            device_key: device_keypair([0x24; 32]).device_key().unwrap(),
            addresses: vec!["198.51.100.7:7000".into()],
        },
        WireMessage::OrganisationRequest { org_id, addresses: vec!["203.0.113.9:7000".into()] },
        WireMessage::Organisation {
            org_id,
            epoch: Epoch::new(4),
            record_snapshot: vec![4, 5, 6],
            org_private_key: OrgPrivateKey::from(private),
        },
    ];
    for message in &kinds {
        for rendered in [format!("{message:?}"), format!("{message:#?}")] {
            assert_not_rendered(&rendered, &private, "Organisation private key");
            let carries = matches!(message, WireMessage::OrgInformation { .. } | WireMessage::Organisation { .. });
            assert_eq!(rendered.contains("OrgPrivateKey([REDACTED])"), carries, "{rendered}");
        }
    }
}

// ---- LLR-qw8tyu: the Organisation private key's custody ---------------------------------------
mod organisation_key_custody {
    use org_members::DevicePublicKey;
    use org_node::service::{Checked, OrgService};
    use org_node::store::{OrgRecord, OrgRecordParts, Origin, ProvisionalOrganisation, ProvisionalOrganisationParts};
    use org_node::test_fixtures::{device_key, empty_store};
    use org_node::wire::{decode_message, encode_message, WireMessage};
    use org_node::{Epoch, OrgNodeError, OrgPrivateKey};
    use rand::rngs::OsRng;

    use crate::support::fixtures::{alice, alice_persona, record_parts, snapshot, OWN};
    use crate::support::{contains, the_org, trie_of_members};

    const PROVISIONAL: [u8; 32] = [0x6d; 32];
    const CURRENT: [u8; 32] = [0x60; 32];

    /// An Active store whose record (epoch 3, key `CURRENT`) lists this Device and Carol's (0x22), with
    /// an own provisional organisation for epoch 4 holding the key `PROVISIONAL`. The record keeps the
    /// change set of its epoch and the base snapshot it extended, which also listed Dave (0x33), so
    /// `messages_for_current_epoch` builds Organisation information for Carol and a revocation for Dave.
    fn service() -> OrgService {
        let members = vec![alice(), snapshot(3, "carol", 0x12, &[0x22])];
        let base = vec![alice(), snapshot(3, "carol", 0x12, &[0x22]), snapshot(5, "dave", 0x13, &[0x33])];
        let root = trie_of_members(&members).root_hash().unwrap();
        let current = OrgPrivateKey::from(CURRENT);
        let mut store_data = empty_store(alice_persona());
        store_data.record = Some(OrgRecord::from(OrgRecordParts {
            org_pub_key: current.x25519_keypair().org_public_key().unwrap(),
            org_private_key: current,
            kept_change_set: Some(vec![1]),
            kept_base_snapshot: Some(postcard::to_allocvec(&base).unwrap()),
            ..record_parts(3, &members)
        }));
        let provisional = OrgPrivateKey::from(PROVISIONAL);
        store_data.provisional_organisations = vec![ProvisionalOrganisation::from(ProvisionalOrganisationParts {
            epoch: Epoch::new(4),
            base_root: root,
            members,
            root,
            org_pub_key: provisional.x25519_keypair().org_public_key().unwrap(),
            org_private_key: provisional,
            change_set: Some(vec![1]),
            base_snapshot: Some(vec![2]),
            origin: Origin::Own,
            applied: vec![],
            first_ahead_at: None,
        })];
        crate::support::fixtures::service(OWN, store_data)
    }

    // Fails if the organisation answer carries the last provisional organisation's key, or if
    // `OrgRecord`'s key accessor is ungated from `test-support`.
    // verifies: LLR-qw8tyu
    #[test]
    fn a_provisional_private_key_leaves_the_store_only_inside_a_message_to_a_listed_device() {
        // No operation returns an OrgPrivateKey: none names it after its `->`. verify_org.rs's
        // functions, which `verify` calls, are all `pub(crate)` (merge review MR1, finding-6).
        for file in ["service.rs", "store.rs", "removal.rs", "verify_org.rs"] {
            let mut signatures = crate::support::public_signatures(file);
            if file == "verify_org.rs" {
                signatures.extend(crate::support::crate_signatures(file));
            }
            assert!(!signatures.is_empty(), "{file}: the scan reads its public functions");
            for signature in signatures {
                if let Some(returns) = signature.split("->").nth(1) {
                    assert!(!returns.contains("OrgPrivateKey"), "{file}: `{signature}` returns an Organisation private key");
                }
            }
        }
        let mut svc = service();
        let own = device_key(OWN);
        // The views clone no key: their Debug holds no byte of either.
        let views = format!("{:?}", svc.provisional_organisations(&own).unwrap());
        for key in [PROVISIONAL, CURRENT] {
            assert!(!views.contains(&format!("{:02x}{:02x}", key[0], key[1])) && !views.contains(&key[0].to_string()));
        }
        // A never-listed requester gets nothing; a listed one gets the current key, inside the encoded body.
        let request = encode_message(&WireMessage::OrganisationRequest { org_id: the_org(), addresses: vec![] }).unwrap();
        assert!(matches!(
            svc.check_received(&mut OsRng, &own, device_key(0x2f), &request),
            Err(OrgNodeError::SenderNotListed { .. })
        ));
        let (Checked::OrganisationRequest { answer }, _) = svc.check_received(&mut OsRng, &own, device_key(0x22), &request).unwrap()
        else {
            panic!("an answer")
        };
        assert_eq!(answer.recipient, device_key(0x22));
        let WireMessage::Organisation { org_private_key, .. } = decode_message(&answer.body).unwrap() else {
            panic!("the Organisation kind")
        };
        assert_eq!(org_private_key.expose_secret(), &CURRENT, "the current key, never the provisional one");
        assert!(!contains(&answer.body, &PROVISIONAL), "no provisional key leaves on an answer");
        // messages_for_current_epoch carries keys only to listed Devices: Carol gets the current key
        // inside Organisation information; Dave, removed this epoch, gets a revocation and no key.
        // The record keeps a change set, so the loop has messages to check.
        let listed: Vec<DevicePublicKey> = vec![device_key(0x22)];
        let messages = svc.messages_for_current_epoch(&own).unwrap();
        let (mut informed, mut revoked) = (vec![], vec![]);
        for message in &messages {
            match decode_message(&message.body).unwrap() {
                WireMessage::OrgInformation { org_private_key, .. } => {
                    assert!(listed.contains(&message.recipient), "an Organisation key goes only to a listed Device");
                    assert_eq!(org_private_key.expose_secret(), &CURRENT, "the current key, never the provisional one");
                    informed.push(message.recipient);
                }
                WireMessage::Revocation(_) => revoked.push(message.recipient),
                other => panic!("an unexpected kind for the current epoch: {other:?}"),
            }
            assert!(!contains(&message.body, &PROVISIONAL), "no provisional key leaves");
            if !listed.contains(&message.recipient) {
                assert!(!contains(&message.body, &CURRENT), "no key to an unlisted Device");
            }
        }
        assert_eq!(informed, listed, "Organisation information for each listed Device");
        assert_eq!(revoked, vec![device_key(0x33)], "a revocation for the removed Device");
    }
}
