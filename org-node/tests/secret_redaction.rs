#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! No secret reaches debug output (REQ-y7tsft, PR-hqwpg9): a Persona record,
//! an Organisation record, the store plaintext and a Wire message holding a
//! member seed, a device seed or an Organisation private key, and a signing key
//! pair, render none of its bytes in any form (LLR-bwb9pu), and the store encryption key renders as
//! its redaction marker (LLR-scgk5j).

use org_members::{Handle, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::revocation::{Acknowledgement, Signature64};
use org_node::store::{self, OrgRecord, PersonaRecord, PersonaStatus, StoreData};
use org_node::test_fixtures::{admin_device, admit_member_delta, bob_absence_notice, genesis_trie, org_public_key};
use org_node::transport::wire::WireMessage;
use org_node::{
    DeviceSeed, Envelope, Epoch, MemberSeed, OrgPrivateKey, PersonaId, SequenceNumber,
};

/// The Organisation private key every `org` record holds (this branch's
/// field, REQ-ech45n): distinctive, so a rendering of it is caught.
fn org_private() -> [u8; 32] {
    sentinel(0x80)
}

/// 32 bytes counting up from `start`, so each secret is distinctive.
fn sentinel(start: u8) -> [u8; 32] {
    core::array::from_fn(|i| start.wrapping_add(i as u8))
}

/// 32 bytes counting down from `start`.
fn sentinel_down(start: u8) -> [u8; 32] {
    core::array::from_fn(|i| start.wrapping_sub(i as u8))
}

/// Every form in which `secret` could appear in debug output, with whitespace
/// removed: hex (lower and upper case) of its leading bytes; the leading
/// elements of a decimal list (`[208, 209, …`) and of a hex list (`[0xd0, …`,
/// `[d0, …`); and the whole of each `{:?}` rendering of the array
/// (`{:?}`, `{:#?}`, `{:x?}`, `{:X?}`, `{:#x?}`, `{:#X?}`).
fn renderings(secret: &[u8; 32]) -> Vec<String> {
    let squash = |s: String| -> String { s.chars().filter(|c| !c.is_whitespace()).collect() };
    let lower: String = secret[..4].iter().map(|b| format!("{b:02x}")).collect();
    let head = &secret[..3];
    let list = |f: &dyn Fn(u8) -> String| head.iter().map(|b| f(*b)).collect::<Vec<_>>().join(",");
    vec![
        lower.to_uppercase(),
        lower,
        list(&|b| b.to_string()),
        list(&|b| format!("{b:#04x}")),
        list(&|b| format!("{b:#04X}")),
        format!("[{}", list(&|b| format!("{b:x}"))),
        format!("[{}", list(&|b| format!("{b:X}"))),
        squash(format!("{secret:?}")),
        squash(format!("{secret:#?}")),
        squash(format!("{secret:x?}")),
        squash(format!("{secret:X?}")),
        squash(format!("{secret:#x?}")),
        squash(format!("{secret:#X?}")),
    ]
}

/// Fails if `rendered` holds `secret` in any of its `renderings`.
fn assert_not_rendered(rendered: &str, secret: &[u8; 32], what: &str) {
    let squashed: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
    for form in renderings(secret) {
        assert!(!squashed.contains(&form), "{what}: debug output renders the secret as {form:?}");
    }
}

fn persona(member: [u8; 32], device: [u8; 32]) -> PersonaRecord {
    PersonaRecord {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse("alice").unwrap(),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        member_seed: MemberSeed::from(member),
        device_seed: DeviceSeed::from(device),
        member_id: None,
        status: PersonaStatus::Active,
    }
}

fn org(private: [u8; 32]) -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: org_public_key(),
        epoch: Epoch::new(1),
        last_seq: SequenceNumber::new(0),
        trie_members: vec![],
        proxy_account: None,
        org_private_key: OrgPrivateKey::from(private),
        kept_change_set: None,
    }
}

fn wire(private: [u8; 32]) -> WireMessage {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let envelope = Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(1), &delta).unwrap();
    WireMessage::OrgInformation { envelope, record_snapshot: vec![], org_private_key: OrgPrivateKey::from(private) }
}

// Adapted at the merge of master `1feb608` into worktree-person-shared-types:
// the Organisation record also holds this branch's Organisation private key,
// which renders as its redaction marker and none of its bytes (LLR-2dvhz8).
/// verifies: LLR-bwb9pu, LLR-2dvhz8
#[test]
fn records_and_wire_messages_never_render_secret_bytes() {
    let (member, device, private) = (sentinel(0xd0), sentinel(0x10), org_private());
    let p = persona(member, device);
    let o = org(private);
    let data = StoreData {
        personas: vec![p.clone()],
        orgs: vec![o.clone()],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
    let w = wire(private);
    for rendered in [format!("{p:?}"), format!("{p:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert_not_rendered(&rendered, &member, "member seed");
        assert_not_rendered(&rendered, &device, "device seed");
        assert!(rendered.contains("MemberSeed([REDACTED])") && rendered.contains("DeviceSeed([REDACTED])"));
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{data:#?}"), format!("{w:?}"), format!("{w:#?}")] {
        assert_not_rendered(&rendered, &private, "Organisation private key");
        assert!(rendered.contains("OrgPrivateKey([REDACTED])"), "{rendered}");
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert!(rendered.contains("org_private_key"));
    }
}

/// A revocation notice and an acknowledgement for Bob's Device, which the
/// genesis trie does not list (S3 T4: neither holds an Envelope).
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
// *Adapted 2026-10-07 (change worktree-org-io-commit-workflow, S3 T4):* the
// revocation was the Organisation information's Envelope alone; it is a
// notice now, and the acknowledgement is the third kind.
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

// LLR-2dvhz8 as amended: the field is no longer optional, so a record renders
// it as its redacted secret type, never its bytes, whatever they are.
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_record_debug_says_whether_the_organisation_private_key_is_set`.
/// verifies: LLR-2dvhz8
#[test]
fn a_record_debug_renders_the_organisation_private_key_redacted() {
    for private in [org_private(), [0u8; 32], sentinel_down(0xff)] {
        let record = org(private);
        for rendered in [format!("{record:?}"), format!("{record:#?}")] {
            let squashed: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
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
    let personas: Vec<PersonaRecord> =
        (0..3u8).map(|i| persona(sentinel_down(0xff - i), sentinel_down(0xef - i))).collect();
    let data = StoreData {
        personas: personas.clone(),
        orgs: vec![org(sentinel_down(0xdf)), org(sentinel_down(0xcf))],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
    for rendered in [format!("{data:?}"), format!("{data:#?}")] {
        for i in 0..personas.len() as u8 {
            assert_not_rendered(&rendered, &sentinel_down(0xff - i), "member seed");
            assert_not_rendered(&rendered, &sentinel_down(0xef - i), "device seed");
        }
        assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation private key");
        assert_not_rendered(&rendered, &sentinel_down(0xcf), "Organisation private key");
    }
}

/// `SigningKeypair` derives `Debug` over ed25519-dalek's `SigningKey`, whose
/// own `Debug` omits the secret (soup.md, ed25519-dalek); this pins that.
/// Adapted at the merge of master `1feb608`: a member seed yields an
/// `X25519Keypair` on this branch, whose hand-written `Debug` renders none of
/// its bytes either (LLR-98ufry).
/// verifies: LLR-bwb9pu, LLR-98ufry
#[test]
fn a_signing_key_pair_never_renders_its_seed() {
    for seed in [sentinel(0x60), sentinel_down(0xff)] {
        let device = DeviceSeed::from(seed).signing_keypair();
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
/// its bytes, at both byte bounds. LLR-bwb9pu, before its amendment of
/// 2026-10-05, named neither.
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

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_as_its_marker() {
    assert_eq!(store::store_key_debug_for_test("hunter2").unwrap(), "StoreKey([REDACTED])");
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_the_same_for_every_passphrase() {
    let long = "p".repeat(4096);
    for passphrase in ["", "\u{1F511} \u{fc}n\u{ef}c\u{f8}d\u{e9}", long.as_str()] {
        assert_eq!(store::store_key_debug_for_test(passphrase).unwrap(), "StoreKey([REDACTED])");
    }
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_has_no_display_and_is_not_copy() {
    let (display, copy) = store::store_key_traits_for_test();
    assert!(!display, "StoreKey must not implement Display");
    assert!(!copy, "StoreKey must not be Copy");
}
