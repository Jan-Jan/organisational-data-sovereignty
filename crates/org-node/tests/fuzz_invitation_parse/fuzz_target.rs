//! Fuzz target: `Invite::parse` must never panic on arbitrary text, and every
//! refusal names one of the six fields of the Invite (LLR-9jxwq4,
//! `names_a_field`). Two shapes: the text is the Blob; and the bytes are
//! armoured first, so every iteration reaches the postcard decode and the
//! field parses behind the Base64 check.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features app,test-support --test fuzz_invitation_parse`;
//! deep-fuzz with `cargo bolero test fuzz_invitation_parse --engine libfuzzer`.
//!
//! verifies: LLR-9jxwq4
#![allow(clippy::panic)]

use bolero::check;
use org_node::error::OrgNodeError;
use org_node::invitation::Invite;

/// The fields an Invite refusal may name.
const INVITE_FIELDS: [&str; 7] = [
    "invite",
    "invite.org_name",
    "invite.org_id",
    "invite.invitee_name",
    "invite.issuer",
    "invite.issuer_addresses",
    "invite.invite_id",
];

fn names_a_field(error: OrgNodeError, fields: &[&str]) {
    match error {
        OrgNodeError::InvalidField { field, .. } if fields.contains(&field) => {}
        other => panic!("a refusal that names no field: {other:?}"),
    }
}

/// A round trip: whatever parses encodes back to text that parses to it.
fn check_text(text: &str) {
    match Invite::parse(text) {
        Ok(invite) => {
            let again = Invite::parse(&invite.encode().unwrap_or_default());
            assert!(again.as_ref().is_ok_and(|parsed| parsed == &invite), "an Invite that parses round-trips");
        }
        Err(error) => names_a_field(error, &INVITE_FIELDS),
    }
}

fn main() {
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes, as text, are the Blob.
        check_text(&String::from_utf8_lossy(bytes));
        // Shape 2 — the bytes are the postcard body, armoured.
        check_text(&org_node::invitation::armour_for_test(bytes));
    });
}
