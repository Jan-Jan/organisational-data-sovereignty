mod common;

use common::member_key;
use org_members::types::{Handle, Name, PersonPublicKey, Surname, MAX_HANDLE_LEN, MAX_NAME_LEN, MAX_SURNAME_LEN};
use org_members::OrgMembersError;


/// verifies: LLR-xzqs9r
#[test]
fn handle_parse_stores_nfc() {
    let h = Handle::parse("jose\u{0301}").unwrap(); // NFD é
    assert_eq!(h.as_str(), "jos\u{00e9}");
    assert_eq!(Handle::try_from("alice").unwrap(), Handle::parse("alice").unwrap());
    assert_eq!(Handle::try_from(String::from("alice")).unwrap(), Handle::parse("alice").unwrap());
    assert!(Handle::parse("jan-jan").is_ok());
}

/// verifies: LLR-xzqs9r
#[test]
fn handle_parse_rejects_every_rule() {
    let over = "a".repeat(MAX_HANDLE_LEN + 1);
    for bad in ["", "Alice", "a.b", "a\u{0430}", over.as_str()] {
        assert!(
            matches!(Handle::parse(bad), Err(OrgMembersError::InvalidHandle(_))),
            "accepted {bad:?}"
        );
        assert!(matches!(Handle::try_from(bad), Err(OrgMembersError::InvalidHandle(_))));
    }
    assert!(Handle::parse(&"a".repeat(MAX_HANDLE_LEN)).is_ok());
}

/// verifies: LLR-w5nkbu
#[test]
fn name_and_surname_parse_nfc_and_bound() {
    assert_eq!(Name::parse("Jose\u{0301}").unwrap().as_str(), "Jos\u{00e9}");
    assert_eq!(Surname::parse("Smith").unwrap().as_str(), "Smith");
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        Name::parse(&over),
        Err(person::IdentityError::FieldTooLong { field: "name", max: 128 })
    );
    assert_eq!(
        Surname::parse(&over),
        Err(person::IdentityError::FieldTooLong { field: "surname", max: 128 })
    );
    assert!(Name::parse(&"a".repeat(MAX_NAME_LEN)).is_ok());
}

/// verifies: LLR-4czn8t
#[test]
fn newtype_debug_redacts() {
    let h = Handle::parse("alice").unwrap();
    let n = Name::parse("Alice").unwrap();
    let s = Surname::parse("Smith").unwrap();
    assert_eq!(format!("{h:?}"), "Handle([REDACTED])");
    assert_eq!(format!("{n:?}"), "Name([REDACTED])");
    assert_eq!(format!("{s:?}"), "Surname([REDACTED])");
}

/// verifies: LLR-c5tzyp
///
/// Normal case: `as_str`, `Display` and `String::from` each output the stored
/// value, unredacted, for all three types.
#[test]
fn newtype_deliberate_output_shows_value() {
    let h = Handle::parse("alice").unwrap();
    let n = Name::parse("Alice").unwrap();
    let s = Surname::parse("Smith").unwrap();
    assert_eq!((h.as_str(), n.as_str(), s.as_str()), ("alice", "Alice", "Smith"));
    assert_eq!(format!("{h} {n} {s}"), "alice Alice Smith");
    assert_eq!(
        (String::from(h), String::from(n), String::from(s)),
        ("alice".to_string(), "Alice".to_string(), "Smith".to_string())
    );
}

/// verifies: LLR-c5tzyp
///
/// Abnormal input: values given in NFD are output in their stored NFC form by
/// all three deliberate-output paths, never as the decomposed input.
#[test]
fn newtype_deliberate_output_is_nfc_form() {
    let h = Handle::parse("jose\u{0301}").unwrap();
    let n = Name::parse("Jose\u{0301}").unwrap();
    let s = Surname::parse("Gode\u{0308}l").unwrap();
    let (hc, nc, sc) = ("jos\u{00e9}", "Jos\u{00e9}", "God\u{00eb}l");
    assert_eq!((h.as_str(), n.as_str(), s.as_str()), (hc, nc, sc));
    assert_eq!((h.to_string(), n.to_string(), s.to_string()), (hc.into(), nc.into(), sc.into()));
    assert_eq!(
        (String::from(h), String::from(n), String::from(s)),
        (hc.to_string(), nc.to_string(), sc.to_string())
    );
}

/// verifies: LLR-xzqs9r, LLR-w5nkbu
#[test]
fn decode_parses_newtype_fields() {
    let enc = |s: &str| postcard::to_allocvec(&String::from(s)).unwrap();
    assert!(postcard::from_bytes::<Handle>(&enc("Alice")).is_err());
    assert!(postcard::from_bytes::<Handle>(&enc("")).is_err());
    assert!(postcard::from_bytes::<Name>(&enc(&"a".repeat(129))).is_err());
    assert!(postcard::from_bytes::<Surname>(&enc(&"a".repeat(129))).is_err());
    let h: Handle = postcard::from_bytes(&enc("jose\u{0301}")).unwrap();
    assert_eq!(h.as_str(), "jos\u{00e9}");
}

/// verifies: LLR-68tka5
///
/// Normal case: a valid member record whose handle and name arrive in NFD
/// decodes, and the decoded record holds their NFC forms -- the wire form
/// went through `Handle::parse` and `Name::parse`, not around them.
#[test]
fn decode_member_record_stores_nfc_fields() {
    use ed25519_dalek::SigningKey;
    use org_members::types::{MemberId, MemberLeaf, DevicePublicKey, DeviceSlots};

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: PersonPublicKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: DeviceSlots,
    }
    let device = DevicePublicKey::try_from(SigningKey::from_bytes(&[9; 32]).verifying_key()).expect("prime-order key");
    let wire = WireLeaf {
        id: MemberId::new([7; 32]),
        handle: "jose\u{0301}",
        p2p_key: member_key("member"),
        name: "Jose\u{0301}",
        surname: "Smith",
        p2p_devices: DeviceSlots::parse(vec![device]).unwrap(),
    };
    let decoded: MemberLeaf = postcard::from_bytes(&postcard::to_allocvec(&wire).unwrap()).unwrap();

    assert_eq!(decoded.handle().as_str(), "jos\u{00e9}");
    assert_eq!(decoded.name().as_str(), "Jos\u{00e9}");
    assert_eq!(decoded.surname().as_str(), "Smith");
    assert_eq!(decoded.id(), &MemberId::new([7; 32]));
}

/// verifies: LLR-yz2gmc, LLR-v9evtu
///
/// `From<[u8; 32]>` and `new` agree and `as_bytes` hands the bytes back
/// unchanged. The all-zero and all-0xff arrays are the boundary values and
/// serve as the abnormal side: construction is infallible, so there is no
/// refused input, only extremes that must survive untouched.
#[test]
fn tag_types_construct_from_bytes() {
    use org_members::types::{MemberId, NodeHash, RootHash};

    let mut mixed = [0u8; 32];
    for (i, b) in mixed.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(37) ^ 0xa5;
    }
    for b in [[0u8; 32], [0xff; 32], mixed] {
        assert_eq!(MemberId::from(b), MemberId::new(b));
        assert_eq!(MemberId::from(b).as_bytes(), &b);
        assert_eq!(NodeHash::from(b), NodeHash::new(b));
        assert_eq!(NodeHash::from(b).as_bytes(), &b);
        assert_eq!(RootHash::from(b), RootHash::new(b));
        assert_eq!(RootHash::from(b).as_bytes(), &b);
    }
}

/// verifies: LLR-377ddr
#[test]
fn newtypes_encode_as_their_string() {
    let h = Handle::parse("alice").unwrap();
    assert_eq!(postcard::to_allocvec(&h).unwrap(), postcard::to_allocvec(&"alice").unwrap());
    let n = Name::parse("Alice").unwrap();
    assert_eq!(postcard::to_allocvec(&n).unwrap(), postcard::to_allocvec(&"Alice").unwrap());
}

/// verifies: LLR-4czn8t
///
/// Abnormal side: values at the length bound and with non-ASCII content
/// still render as the bare redaction marker, with no part of the value.
#[test]
fn newtype_debug_redacts_at_the_bound_and_beyond_ascii() {
    let at_bound = "\u{00e9}".repeat(MAX_NAME_LEN / 2); // 128 bytes, all é
    assert_eq!(at_bound.len(), MAX_NAME_LEN);
    assert_eq!(MAX_HANDLE_LEN, MAX_NAME_LEN);
    let cases = [
        (format!("{:?}", Handle::parse("jos\u{00e9}").unwrap()), "Handle([REDACTED])", "jos"),
        (format!("{:?}", Handle::parse(&at_bound).unwrap()), "Handle([REDACTED])", "\u{00e9}"),
        (format!("{:?}", Name::parse(&at_bound).unwrap()), "Name([REDACTED])", "\u{00e9}"),
        (format!("{:?}", Surname::parse(&at_bound).unwrap()), "Surname([REDACTED])", "\u{00e9}"),
    ];
    for (rendered, expected, fragment) in cases {
        assert_eq!(rendered, expected);
        assert!(!rendered.contains(fragment), "Debug leaked {fragment:?}: {rendered}");
    }
}

/// verifies: LLR-4czn8t
///
/// Abnormal side: a member record that arrived as bytes, not one built in
/// process, still redacts handle, name and surname in `Debug`.
#[test]
fn decoded_member_record_debug_redacts_pii() {
    use ed25519_dalek::SigningKey;
    use org_members::types::{MemberId, MemberLeaf, DevicePublicKey};

    let built = MemberLeaf::new(
        MemberId::new([7; 32]),
        Handle::parse("zelda").unwrap(),
        member_key("member"),
        Name::parse("Zoltan").unwrap(),
        Surname::parse("Quixote").unwrap(),
        vec![DevicePublicKey::try_from(SigningKey::from_bytes(&[9; 32]).verifying_key()).expect("prime-order key")],
    )
    .unwrap();
    let decoded: MemberLeaf = postcard::from_bytes(&postcard::to_allocvec(&built).unwrap()).unwrap();

    let rendered = format!("{decoded:?}");
    for value in ["zelda", "Zoltan", "Quixote"] {
        assert!(!rendered.contains(value), "Debug leaked {value:?}: {rendered}");
    }
    assert_eq!(rendered.matches("[REDACTED]").count(), 3, "{rendered}");
}

/// verifies: LLR-w5nkbu
/// `TryFrom<&str>` and `TryFrom<String>` construct `Name`/`Surname` exactly as
/// `parse` does: NFC stored for an NFD input (normal), `FieldTooLong` at
/// `MAX_NAME_LEN + 1` bytes (abnormal).
#[test]
fn name_and_surname_try_from_match_parse() {
    let nfd = "Jose\u{0301}";
    let name = Name::parse(nfd).unwrap();
    assert_eq!(name.as_str(), "Jos\u{00e9}");
    assert_eq!(Name::try_from(nfd).unwrap(), name);
    assert_eq!(Name::try_from(String::from(nfd)).unwrap(), name);
    let surname = Surname::parse(nfd).unwrap();
    assert_eq!(surname.as_str(), "Jos\u{00e9}");
    assert_eq!(Surname::try_from(nfd).unwrap(), surname);
    assert_eq!(Surname::try_from(String::from(nfd)).unwrap(), surname);

    let over = "a".repeat(MAX_NAME_LEN + 1);
    let name_err = Err(person::IdentityError::FieldTooLong { field: "name", max: 128 });
    assert_eq!(Name::try_from(over.as_str()), name_err);
    assert_eq!(Name::try_from(over.clone()), name_err);
    let surname_err = Err(person::IdentityError::FieldTooLong { field: "surname", max: 128 });
    assert_eq!(Surname::try_from(over.as_str()), surname_err);
    assert_eq!(Surname::try_from(over), surname_err);
}

/// verifies: LLR-c5tzyp
/// Serde `Serialize` encodes each newtype as its plain stored string: the
/// postcard bytes equal those of `as_str()` (normal), and an NFD input is
/// serialized in its stored NFC form (abnormal input).
#[test]
fn newtypes_serialize_as_stored_nfc_string() {
    fn serialized_string<T: serde::Serialize>(value: &T, stored: &str) -> String {
        let bytes = postcard::to_allocvec(value).unwrap();
        assert_eq!(bytes, postcard::to_allocvec(stored).unwrap());
        postcard::from_bytes(&bytes).unwrap()
    }
    let handle = Handle::parse("jose\u{0301}").unwrap();
    assert_eq!(serialized_string(&handle, handle.as_str()), "jos\u{00e9}");
    let name = Name::parse("Jose\u{0301}").unwrap();
    assert_eq!(serialized_string(&name, name.as_str()), "Jos\u{00e9}");
    let surname = Surname::parse("Jose\u{0301}").unwrap();
    assert_eq!(serialized_string(&surname, surname.as_str()), "Jos\u{00e9}");
}

/// verifies: LLR-xzqs9r
/// A character outside the UTS#39 identifier set (other than `-`) is refused
/// (abnormal input); identifier characters, `-`, NFC-composed and
/// single-script non-Latin handles are accepted (normal). `_` is in the
/// UTS#39 identifier set, so it is accepted.
#[test]
fn handle_parse_rejects_non_identifier_characters() {
    for bad in ["a b", "a!b", "a@b", "a\u{1F600}", "a\tb"] {
        assert!(
            matches!(Handle::parse(bad), Err(OrgMembersError::InvalidHandle(_))),
            "accepted {bad:?}"
        );
    }
    for good in ["jan-jan", "a_b", "jose\u{0301}", "\u{0430}\u{0431}"] {
        assert!(Handle::parse(good).is_ok(), "refused {good:?}");
    }
}

/// verifies: LLR-xzqs9r, LLR-w5nkbu
/// The byte bound is measured after NFC normalization, not on the raw input:
/// "e\u{0301}" x64 is 192 raw bytes but 128 NFC bytes, so it is accepted and
/// stored in NFC form (boundary); x65 is 130 NFC bytes and is refused.
#[test]
fn length_bounds_apply_after_nfc() {
    let at_bound = "e\u{0301}".repeat(64);
    let stored = "\u{00e9}".repeat(64);
    assert_eq!(at_bound.len(), 192);
    assert_eq!(stored.len(), MAX_HANDLE_LEN);
    assert_eq!(Handle::parse(&at_bound).unwrap().as_str(), stored);
    assert_eq!(Name::parse(&at_bound).unwrap().as_str(), stored);
    assert_eq!(Surname::parse(&at_bound).unwrap().as_str(), stored);

    let over = "e\u{0301}".repeat(65);
    assert!(matches!(Handle::parse(&over), Err(OrgMembersError::InvalidHandle(_))));
    assert_eq!(
        Name::parse(&over),
        Err(person::IdentityError::FieldTooLong { field: "name", max: MAX_NAME_LEN })
    );
    assert_eq!(
        Surname::parse(&over),
        Err(person::IdentityError::FieldTooLong { field: "surname", max: MAX_SURNAME_LEN })
    );
}

fn slot_device(seed: u8) -> org_members::types::DevicePublicKey {
    common::device_key(&format!("slot-{seed}"))
}

// Master's LLR-k6dhz7 and LLR-t3p9zk tests, carried over to `person`'s types
// on the person-shared-types branch. LLR-k6dhz7 is amended in place:
// the constructors still accept exactly what decoding accepts, but decoding
// now refuses small-order and non-canonical keys (LLR-z954wj, LLR-a645bx), so
// "a weak key is accepted" became "a weak key is refused by both". LLR-t3p9zk
// is superseded by LLR-st6j2r, which names `person::DeviceSlots` in place of
// `P2pDeviceSlots`.

/// y = 2 as an ed25519 encoding (off the curve) and u = 2 as an X25519 one (on
/// the twist, accepted).
fn y_or_u_is_two() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = 2;
    bytes
}

/// 2^255 − 18 = p + 1: a non-canonical encoding as either key.
fn p_plus_one() -> [u8; 32] {
    let mut bytes = [0xffu8; 32];
    bytes[0] = 0xee;
    bytes[31] = 0x7f;
    bytes
}

/// verifies: LLR-k6dhz7
#[test]
fn key_parse_accepts_exactly_what_decoding_accepts() {
    use org_members::types::DevicePublicKey;
    // A valid key, y = 0 / u = 0 (small order), p + 1 (non-canonical) and 2
    // (off the curve for a DevicePublicKey, on the twist for a
    // PersonPublicKey): each constructor and its decode agree on every one.
    let cases = [common::dual_key_bytes("k"), [0u8; 32], p_plus_one(), y_or_u_is_two()];
    for bytes in cases {
        let encoded = postcard::to_allocvec(&bytes).unwrap();
        let member = PersonPublicKey::parse(&bytes);
        assert_eq!(PersonPublicKey::try_from(bytes), member);
        assert_eq!(postcard::from_bytes::<PersonPublicKey>(&encoded).ok(), member.clone().ok());
        if let Ok(key) = member {
            assert_eq!(key.as_bytes(), &bytes, "parse keeps the bytes as given");
        }
        let device = DevicePublicKey::parse(&bytes);
        assert_eq!(DevicePublicKey::try_from(bytes), device);
        assert_eq!(postcard::from_bytes::<DevicePublicKey>(&encoded).ok(), device.clone().ok());
        if let Ok(key) = device {
            assert_eq!(key.as_bytes(), &bytes, "parse keeps the bytes as given");
        }
    }
    assert!(PersonPublicKey::parse(&cases[0]).is_ok() && DevicePublicKey::parse(&cases[0]).is_ok());
    assert!(PersonPublicKey::parse(&cases[3]).is_ok(), "a twist u-coordinate is accepted");
}

/// verifies: LLR-k6dhz7
#[test]
fn key_parse_refuses_bytes_off_the_curve() {
    use org_members::types::DevicePublicKey;
    use person::IdentityError;
    let bytes = y_or_u_is_two();
    let encoded = postcard::to_allocvec(&bytes).unwrap();
    assert_eq!(DevicePublicKey::parse(&bytes), Err(IdentityError::InvalidDeviceKey));
    assert_eq!(DevicePublicKey::try_from(bytes), Err(IdentityError::InvalidDeviceKey));
    assert_eq!(
        DevicePublicKey::parse(&bytes).map_err(OrgMembersError::from),
        Err(OrgMembersError::InvalidDeviceKey)
    );
    assert_eq!(
        postcard::from_bytes::<DevicePublicKey>(&encoded),
        Err(postcard::Error::SerdeDeCustom)
    );
    for refused in [[0u8; 32], p_plus_one()] {
        assert_eq!(PersonPublicKey::parse(&refused), Err(IdentityError::InvalidPersonKey));
        assert_eq!(PersonPublicKey::try_from(refused), Err(IdentityError::InvalidPersonKey));
        assert_eq!(
            PersonPublicKey::parse(&refused).map_err(OrgMembersError::from),
            Err(OrgMembersError::InvalidPersonKey)
        );
    }
}

/// verifies: LLR-t3p9zk, LLR-st6j2r
#[test]
fn device_slots_parse_holds_keys_sorted() {
    use org_members::types::{DevicePublicKey, DeviceSlots, MAX_DEVICES};
    let keys: Vec<DevicePublicKey> = (1..=MAX_DEVICES as u8).map(slot_device).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    let mut reversed = sorted.clone();
    reversed.reverse();
    let slots = DeviceSlots::parse(reversed.clone()).unwrap();
    assert_eq!(slots.devices(), sorted.as_slice(), "held sorted, at the MAX_DEVICES bound");
    assert_eq!(DeviceSlots::try_from(reversed).unwrap(), slots);
    let empty = DeviceSlots::parse(Vec::new()).unwrap();
    assert_eq!(empty.device_count(), 0, "the empty list is the isolated state");
}

/// verifies: LLR-t3p9zk, LLR-st6j2r
#[test]
fn device_slots_parse_refuses_too_many_and_repeated_keys() {
    use org_members::types::{DevicePublicKey, DeviceSlots, MAX_DEVICES};
    use person::IdentityError;
    let over: Vec<DevicePublicKey> = (1..=MAX_DEVICES as u8 + 1).map(slot_device).collect();
    assert_eq!(DeviceSlots::parse(over.clone()), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(DeviceSlots::try_from(over), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(
        DeviceSlots::parse(vec![slot_device(1), slot_device(2), slot_device(1)]),
        Err(IdentityError::DuplicateDevice)
    );
    assert_eq!(
        DeviceSlots::try_from(vec![slot_device(1), slot_device(1)]).map_err(OrgMembersError::from),
        Err(OrgMembersError::DuplicateDevice)
    );
    let over: Vec<DevicePublicKey> = (1..=MAX_DEVICES as u8 + 1).map(slot_device).collect();
    assert_eq!(
        DeviceSlots::parse(over).map_err(OrgMembersError::from),
        Err(OrgMembersError::DeviceSlotsFull)
    );
}
