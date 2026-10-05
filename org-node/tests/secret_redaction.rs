#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! No secret reaches debug output (REQ-y7tsft, PR-hqwpg9): a Persona record,
//! an Organisation record, the store plaintext and a Wire message holding a
//! member seed, a device seed or an Organisation secret, and a signing key
//! pair, render none of its bytes in any form (LLR-bwb9pu), and the store encryption key renders as
//! its redaction marker (LLR-scgk5j).

use org_members::{Handle, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::store::{self, OrgRecord, PersonaRecord, PersonaStatus, StoreData};
use org_node::test_fixtures::{admit_member_delta, member_key};
use org_node::transport::wire::WireMessage;
use org_node::{
    DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber,
    SignedDeltaEnvelope,
};

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

fn org(secret: Option<[u8; 32]>) -> OrgRecord {
    let admin = member_key(0x31);
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPublicKey::from(&admin),
        epoch: Epoch::new(1),
        org_secret: secret.map(OrgSecret::from),
        last_seq: SequenceNumber::new(0),
        admin_member_key: admin,
        trie_members: vec![],
        proxy_account: None,
    }
}

fn wire(secret: [u8; 32]) -> WireMessage {
    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let envelope =
        SignedDeltaEnvelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(1), &delta, &admin).unwrap();
    WireMessage { envelope, org_secret: Some(OrgSecret::from(secret)), genesis_snapshot: None }
}

/// verifies: LLR-bwb9pu
#[test]
fn records_and_wire_messages_never_render_secret_bytes() {
    let (member, device, secret) = (sentinel(0xd0), sentinel(0x10), sentinel(0x40));
    let p = persona(member, device);
    let o = org(Some(secret));
    let data = StoreData { personas: vec![p.clone()], orgs: vec![o.clone()], pending_invites: vec![] };
    let w = wire(secret);
    for rendered in [format!("{p:?}"), format!("{p:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert_not_rendered(&rendered, &member, "member seed");
        assert_not_rendered(&rendered, &device, "device seed");
        assert!(rendered.contains("MemberSeed([REDACTED])") && rendered.contains("DeviceSeed([REDACTED])"));
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{data:#?}"), format!("{w:?}"), format!("{w:#?}")] {
        assert_not_rendered(&rendered, &secret, "Organisation secret");
        assert!(rendered.contains("OrgSecret([REDACTED])"));
    }
}

/// verifies: LLR-bwb9pu
#[test]
fn secrets_at_the_high_byte_bound_and_many_records_stay_unrendered() {
    let personas: Vec<PersonaRecord> =
        (0..3u8).map(|i| persona(sentinel_down(0xff - i), sentinel_down(0xef - i))).collect();
    let data = StoreData {
        personas: personas.clone(),
        orgs: vec![org(Some(sentinel_down(0xdf))), org(None)],
        pending_invites: vec![],
    };
    for rendered in [format!("{data:?}"), format!("{data:#?}")] {
        for i in 0..personas.len() as u8 {
            assert_not_rendered(&rendered, &sentinel_down(0xff - i), "member seed");
            assert_not_rendered(&rendered, &sentinel_down(0xef - i), "device seed");
        }
        assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation secret");
        assert!(rendered.contains("org_secret: None"), "an absent secret still renders as None");
    }
}

/// `SigningKeypair` derives `Debug` over ed25519-dalek's `SigningKey`, whose
/// own `Debug` omits the secret (soup.md, ed25519-dalek); this pins that.
/// verifies: LLR-bwb9pu
#[test]
fn a_signing_key_pair_never_renders_its_seed() {
    for seed in [sentinel(0x60), sentinel_down(0xff)] {
        for kp in [MemberSeed::from(seed).signing_keypair(), DeviceSeed::from(seed).signing_keypair()] {
            for rendered in [format!("{kp:?}"), format!("{kp:#?}")] {
                assert_not_rendered(&rendered, &seed, "key pair seed");
            }
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
