use org_members::types::{Handle, Name, Surname, MAX_HANDLE_LEN, MAX_NAME_LEN, MAX_SURNAME_LEN};
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
        Err(OrgMembersError::FieldTooLong { field: "name", max: 128 })
    );
    assert_eq!(
        Surname::parse(&over),
        Err(OrgMembersError::FieldTooLong { field: "surname", max: 128 })
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
    use org_members::types::{MemberId, MemberLeaf, P2pDeviceKey, P2pDeviceSlots, P2pMemberKey};

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: P2pMemberKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: P2pDeviceSlots,
    }
    let device = P2pDeviceKey::new(SigningKey::from_bytes(&[9; 32]).verifying_key());
    let wire = WireLeaf {
        id: MemberId::new([7; 32]),
        handle: "jose\u{0301}",
        p2p_key: P2pMemberKey::new(SigningKey::from_bytes(&[8; 32]).verifying_key()),
        name: "Jose\u{0301}",
        surname: "Smith",
        p2p_devices: P2pDeviceSlots::new(vec![device]).unwrap(),
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
    use org_members::types::{MemberId, MemberLeaf, P2pDeviceKey, P2pMemberKey};

    let built = MemberLeaf::new(
        MemberId::new([7; 32]),
        Handle::parse("zelda").unwrap(),
        P2pMemberKey::new(SigningKey::from_bytes(&[8; 32]).verifying_key()),
        Name::parse("Zoltan").unwrap(),
        Surname::parse("Quixote").unwrap(),
        vec![P2pDeviceKey::new(SigningKey::from_bytes(&[9; 32]).verifying_key())],
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
    let name_err = Err(OrgMembersError::FieldTooLong { field: "name", max: 128 });
    assert_eq!(Name::try_from(over.as_str()), name_err);
    assert_eq!(Name::try_from(over.clone()), name_err);
    let surname_err = Err(OrgMembersError::FieldTooLong { field: "surname", max: 128 });
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
        Err(OrgMembersError::FieldTooLong { field: "name", max: MAX_NAME_LEN })
    );
    assert_eq!(
        Surname::parse(&over),
        Err(OrgMembersError::FieldTooLong { field: "surname", max: MAX_SURNAME_LEN })
    );
}
