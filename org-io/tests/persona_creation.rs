#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! A new Persona per proposal (SDD-bzu2jg): `propose_founding_organisation` and
//! `propose_member` each create a new Persona — keychain item, then `persona-<hex>.store` —
//! never reusing a held one (LLR-kna4zj), report what org-node built (LLR-4yte89, LLR-6d6wfw),
//! bind its endpoint when receiving (LLR-jvx6j9); the Organisation name is the stored one
//! (LLR-pn3hgc).

use std::path::Path;

use org_node::invitation::EndpointAddresses;
use org_node::invitation::{Invite, InviteId};
use org_node::test_fixtures::device_key;
use org_node::{DevicePublicKey, Epoch, OrgId, OrgNodeError};
use org_io::test_support::{FakeChain, Refusal};
use org_io::{CreateError, HeldTarget, OrgIo, StoreFailure, Verified, WriteReport};

mod common;
use common::handles::{bind_seed, file_of, fresh_keychain, handle, names_in, persona_of, reopened_service, store_dir, NET};

const SERVICE: &str = "ods-device-seed";

/// An Invite of "Acme" from Device `device_key(0x31)`, with no address.
fn invite_blob(org: u8) -> String {
    Invite {
        org_name: "Acme".to_string(),
        org_id: OrgId::new([org; 20]),
        invitee_name: "Bob".to_string(),
        issuer: device_key(0x31),
        issuer_addresses: EndpointAddresses::parse(vec![]).unwrap(),
        invite_id: InviteId::new([0x33; 32]),
    }
    .encode()
    .unwrap()
}

/// Make `founder`'s founding the current record through the handle:
/// written to the handle's chain in its two steps, then adopted by its verification.
async fn adopt_founding(io: &OrgIo, founder: DevicePublicKey) -> OrgId {
    let written = io.write_organisation_to_chain(HeldTarget::Founding(founder)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    let verified = io.verify_organisation(HeldTarget::Founding(founder)).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    io.personas().await.into_iter().find(|persona| persona.device_key == founder).and_then(|persona| persona.org_id).unwrap()
}

// verifies: LLR-kna4zj
#[tokio::test]
async fn each_proposal_creates_a_new_persona_item_first_then_file() {
    let (_guard, keychain) = fresh_keychain();
    let io = handle("creation-new", "a", &FakeChain::new());
    let dir = store_dir("creation-new", "a");
    let first = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let second = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let member = io.propose_member(&invite_blob(0x0a), "Alice", "Doe", &["alice"]).await.unwrap();
    let devices = [first.founder, second.founder, member.member_device];
    assert_eq!(io.personas().await.len(), 3, "three proposals, three Personas: none reused");
    assert!(devices[0] != devices[1] && devices[1] != devices[2] && devices[0] != devices[2], "three Device keys");
    let mut items: Vec<_> = devices.iter().map(|device| (SERVICE.to_string(), hex::encode(device.as_bytes()))).collect();
    items.sort();
    assert_eq!(keychain.items(), items, "one item per Persona");
    for device in devices {
        assert!(file_of(&dir, &device).is_file(), "the store file of {device:?}");
    }
}

// Abnormal: org-node refuses (a Handle, an Invite); the keychain refuses; the file write fails
// after the item (the Persona is dropped, its item stays).
// verifies: LLR-kna4zj
#[tokio::test]
async fn a_refusal_writes_nothing_and_a_keychain_or_file_failure_is_reported() {
    let (_guard, keychain) = fresh_keychain();
    let io = handle("creation-refused", "a", &FakeChain::new());
    let dir = store_dir("creation-refused", "a");
    assert!(matches!(
        io.propose_founding_organisation("Alice", "Doe", "", "Acme").await,
        Err(CreateError::Node(OrgNodeError::InvalidField { .. }))
    ));
    assert!(matches!(
        io.propose_member(&invite_blob(0x0a), "Alice", "Doe", &[]).await,
        Err(CreateError::Node(OrgNodeError::InvalidField { .. }))
    ));
    assert!(keychain.items().is_empty() && names_in(&dir).is_empty(), "nothing written for a refusal");

    keychain.refuse_as(Some(Refusal::Unavailable));
    let refused = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await;
    keychain.refuse_as(None);
    assert!(matches!(refused, Err(CreateError::Keychain(_))), "{refused:?}");
    assert!(io.personas().await.is_empty() && names_in(&dir).is_empty(), "the new store dropped, no file");

    #[cfg(unix)]
    {
        common::handles::make_read_only(&dir);
        let refused = io.propose_member(&invite_blob(0x0a), "Alice", "Doe", &["alice"]).await;
        common::handles::make_writable(&dir);
        assert!(matches!(refused, Err(CreateError::Store(StoreFailure::Dropped { .. }))), "{refused:?}");
        assert!(io.personas().await.is_empty(), "the Persona is dropped");
        assert_eq!(keychain.items().len(), 1, "its item stays");
        assert!(names_in(&dir).is_empty(), "no file");
    }
}

// verifies: LLR-4yte89
#[tokio::test]
async fn propose_founding_organisation_returns_the_founder_epoch_root_and_key_and_touches_no_chain() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let io = handle("creation-founding", "a", &chain);
    let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    assert_eq!(proposed.epoch, Epoch::new(1));
    assert!(proposed.bind.is_none(), "not receiving: nothing bound, nothing to report");
    // What the handle wrote, read apart from it.
    let on_disk = reopened_service("creation-founding", "a", &io);
    let stored = on_disk.store_data(&proposed.founder).unwrap();
    let held = stored.founding.clone().unwrap();
    assert_eq!((held.root, held.org_pub_key), (proposed.root, proposed.org_public_key), "the founding org-node built");
    assert_eq!(stored.org_name, "Acme");
    assert_eq!((chain.reads(), chain.finalised_reads(), chain.update_writes(), chain.geneses()), (0, 0, 0, 0), "no chain call");
}

// Abnormal (dependency failure): with the chain failing every read and
// write, a founding is still proposed, from the handle's keys alone, and no chain call is made:
// the proposal touches no chain. (`CreateError::FoundingNotListed` is reached by no test: the
// store org-node's `propose_founding` returns always holds its founding, so only a fake of
// org-node, which the project owns, could make the view lack it; LLR-4yte89's note.)
// verifies: LLR-4yte89
#[tokio::test]
async fn a_founding_is_proposed_with_the_chain_failing_and_makes_no_chain_call() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    for switch in [&chain.read_failing, &chain.write_failing, &chain.write_window_failing, &chain.found_at_failing] {
        switch.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    let io = handle("creation-founding-chain-down", "a", &chain);
    let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.expect("proposed with no chain");
    assert_eq!(proposed.epoch, Epoch::new(1));
    assert_eq!((chain.reads(), chain.finalised_reads(), chain.write_windows(), chain.update_writes()), (0, 0, 0, 0), "no chain read or write");
    assert!(chain.founding_calls().is_empty(), "no founding step");
}

// verifies: LLR-6d6wfw
#[tokio::test]
async fn propose_member_parses_the_invite_and_sends_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let io = handle("creation-member", "a", &chain);
    let proposed = io.propose_member(&invite_blob(0x0a), "Bob", "Builder", &["bob", "bobby"]).await.unwrap();
    assert_eq!(
        (proposed.org_id, proposed.invite_id, proposed.org_name.as_str()),
        (OrgId::new([0x0a; 20]), InviteId::new([0x33; 32]), "Acme")
    );
    // What the handle wrote, read apart from it.
    let stored = reopened_service("creation-member", "a", &io).store_data(&proposed.member_device).unwrap().clone();
    assert_eq!(stored.member.as_ref().unwrap().org_id, OrgId::new([0x0a; 20]), "the provisional Member and its expectation");
    assert_eq!(stored.expected_admissions.len(), 1);
    assert_eq!(io.bound_endpoints_for_test().await, 0, "nothing bound, nothing sent");
    assert_eq!((chain.reads(), chain.update_writes()), (0, 0));

    let items = keychain.items();
    for handles in [&["a", "a"][..], &["a", "b", "c", "d"][..]] {
        let refused = io.propose_member(&invite_blob(0x0b), "Bob", "Builder", handles).await;
        assert!(matches!(refused, Err(CreateError::Node(OrgNodeError::InvalidField { .. }))), "{handles:?}: {refused:?}");
    }
    match io.propose_member("not an invite", "Bob", "Builder", &["bob"]).await {
        Err(CreateError::Node(OrgNodeError::InvalidField { field, .. })) => assert!(field.starts_with("invite"), "{field}"),
        other => panic!("not refused naming the field: {other:?}"),
    }
    assert_eq!(keychain.items(), items, "nothing written for a refusal");
}

// verifies: LLR-jvx6j9
#[tokio::test(flavor = "multi_thread")]
async fn a_persona_created_while_receiving_is_bound_and_accepted_on() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("creation-receiving", "a", &FakeChain::new());
    io.start_receiving().await.unwrap();
    let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    assert!(proposed.bind.is_none(), "bound: {:?}", proposed.bind);
    let persona = persona_of(&io, &proposed.founder).await;
    let endpoint = io.bound_endpoint_for_test(&persona).await.expect("bound after the file was written");
    let stranger = bind_seed([0x5e; 32]).await;
    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, io.next_received()).await.unwrap() }, async {
        tokio::time::timeout(NET, stranger.send_body(endpoint.inner().addr(), b"any body")).await.unwrap().unwrap();
    });
    assert!(
        matches!(received, Err(org_io::ReceiveError::Refused(OrgNodeError::MalformedMessage))),
        "accepted on: the body reached the receive step, which judged it: {received:?}"
    );
}

// Abnormal: while the node receives, the keychain takes each
// new Persona's item but will not give it back, so the bind after the creation fails. Neither
// proposal is undone: each Persona is held, with its file, and the failed bind is reported in
// the result; once the keychain answers, `start_receiving` binds both.
// verifies: LLR-jvx6j9
#[tokio::test(flavor = "multi_thread")]
async fn a_bind_that_fails_while_receiving_is_reported_and_undoes_no_creation() {
    let (_guard, keychain) = fresh_keychain();
    let tag = "creation-bind-fails";
    let io = handle(tag, "a", &FakeChain::new());
    io.start_receiving().await.unwrap();
    keychain.refuse_reads_as(Some(Refusal::Locked));
    let founding = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await;
    let member = io.propose_member(&invite_blob(0x35), "Bob", "Builder", &["bob"]).await;
    keychain.refuse_reads_as(None);
    let founding = founding.expect("the founding is not undone");
    let member = member.expect("the Member is not undone");
    for (device, bind) in [(founding.founder, &founding.bind), (member.member_device, &member.bind)] {
        match bind {
            Some(org_io::transport::TransportError::Keychain(refused)) => {
                assert_eq!(**refused, org_io::KeychainError::Locked { device_key: device }, "the bind's own refusal")
            }
            other => panic!("the failed bind was not reported: {other:?}"),
        }
        assert!(io.personas().await.iter().any(|persona| persona.device_key == device), "the Persona is held");
        assert!(file_of(&store_dir(tag, "a"), &device).is_file(), "with its store file");
    }
    assert_eq!(io.bound_endpoints_for_test().await, 0, "nothing bound");
    io.start_receiving().await.unwrap();
    assert_eq!(io.bound_endpoints_for_test().await, 2, "both bound once the keychain answers");
}

// No method of the handle but `propose_founding_organisation` takes an Organisation name: every
// text parameter of every production method of `OrgIo`, in any file of `org-io/src` (its
// subfolders included) and under any parameter name, is one of the reviewed list below, and the
// list's one Organisation name is `propose_founding_organisation`'s `org_name`. The scan reads
// each public signature whole, however it is formatted. Nor do the send, write, chain-read and
// receive files handle the name.
// verifies: LLR-pn3hgc
#[tokio::test]
async fn organisations_report_their_stored_name_and_no_send_carries_it() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("creation-name", "a", &FakeChain::new());
    let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme Co-op").await.unwrap();
    let org_id = adopt_founding(&io, proposed.founder).await;
    assert_eq!(io.organisations().await[0].name, "Acme Co-op");
    assert_eq!(io.organisation(org_id).await.unwrap().name, "Acme Co-op");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in ["send.rs", "write.rs", "chain_read.rs", "receiver.rs"] {
        let text = std::fs::read_to_string(src.join(file)).unwrap();
        assert!(!text.contains("org_name"), "{file} handles the Organisation name");
    }
    let mut text_parameters = vec![];
    for path in rust_files_under(&src) {
        if path.ends_with("test_support.rs") {
            continue;
        }
        text_parameters.extend(handle_text_parameters(&std::fs::read_to_string(&path).unwrap()));
    }
    text_parameters.sort();
    let reviewed = [
        "issue_invite(invitee_name)",
        "propose_admission(chosen_handle)",
        "propose_founding_organisation(handle)",
        "propose_founding_organisation(name)",
        "propose_founding_organisation(org_name)",
        "propose_founding_organisation(surname)",
        "propose_member(handles)",
        "propose_member(invite_blob)",
        "propose_member(name)",
        "propose_member(surname)",
        "propose_person(name)",
        "propose_person(surname)",
        "read_invite(invite_blob)",
    ];
    assert_eq!(text_parameters, reviewed, "the handle's text parameters; only org_name is an Organisation name");
}

/// Every `.rs` file under `dir`, its subfolders included.
fn rust_files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    let mut to_read = vec![dir.to_path_buf()];
    while let Some(folder) = to_read.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                to_read.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files
}

/// `method(parameter)` for each parameter of a text type (`str` or `String`, however wrapped) of
/// each public method in an `impl OrgIo` block of `text`.
fn handle_text_parameters(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = vec![];
    let mut in_handle = false;
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("impl") {
            in_handle = line.starts_with("impl OrgIo") || line.starts_with("impl crate::OrgIo");
        }
        let trimmed = line.trim_start();
        if !in_handle || !(trimmed.starts_with("pub async fn") || trimmed.starts_with("pub fn")) {
            continue;
        }
        let mut signature = String::new();
        for continuation in &lines[index..] {
            signature.push_str(continuation.trim());
            signature.push(' ');
            if continuation.contains('{') || continuation.trim_end().ends_with(';') {
                break;
            }
        }
        let after = &signature[signature.find("fn ").unwrap() + 3..];
        let method = after[..after.find(['(', '<']).unwrap()].to_string();
        let opening = signature.find('(').unwrap();
        let closing = signature.find(')').unwrap();
        for parameter in signature[opening + 1..closing].split(',') {
            let Some((name, kind)) = parameter.split_once(':') else { continue };
            if kind.contains("str") || kind.contains("String") {
                found.push(format!("{method}({})", name.trim()));
            }
        }
    }
    found
}

// Boundary: org-node refuses no Organisation name, so the abnormal case
// is the name's edges: an empty name, and one of leading and trailing spaces around text past
// ASCII, are each kept as given and reported unchanged, on two handles (the stored name, not one
// org-io trims or rewrites).
// verifies: LLR-pn3hgc
#[tokio::test]
async fn an_empty_and_a_spaced_non_ascii_organisation_name_are_reported_unchanged() {
    let (_guard, _keychain) = fresh_keychain();
    for (party, name) in [("a", ""), ("b", "  Ärzte \u{1F3E5} Kollektiv  ")] {
        let io = handle("creation-name-edges", party, &FakeChain::new());
        let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", name).await.unwrap();
        let org_id = adopt_founding(&io, proposed.founder).await;
        assert_eq!(io.organisations().await.iter().map(|org| org.name.as_str()).collect::<Vec<_>>(), vec![name], "{name:?}");
        assert_eq!(io.organisation(org_id).await.unwrap().name, name);
    }
}

// Abnormal: the keychain refuses a provisional Member's new item. Nothing is kept for it: no
// Persona, no store file, no item; the handle's other Persona is untouched, and with the
// keychain back the same Invite proposes a Member.
// verifies: LLR-kna4zj, LLR-6d6wfw
#[tokio::test]
async fn propose_member_keeps_nothing_when_the_keychain_refuses_its_item() {
    let (_guard, keychain) = fresh_keychain();
    let io = handle("creation-member-keychain", "a", &FakeChain::new());
    let dir = store_dir("creation-member-keychain", "a");
    let alice = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap().founder;

    keychain.refuse_as(Some(Refusal::Locked));
    let refused = io.propose_member(&invite_blob(0x0b), "Bob", "Builder", &["bob"]).await;
    keychain.refuse_as(None);
    assert!(matches!(refused, Err(CreateError::Keychain(_))), "{refused:?}");
    let personas: Vec<_> = io.personas().await.into_iter().map(|persona| persona.device_key).collect();
    assert_eq!(personas, vec![alice], "the new store dropped: only the founder's Persona is held");
    assert_eq!(names_in(&dir), vec![file_of(&dir, &alice).file_name().unwrap().to_string_lossy().to_string()], "no file");
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(alice.as_bytes()))], "no item");

    let member = io.propose_member(&invite_blob(0x0b), "Bob", "Builder", &["bob"]).await.unwrap();
    assert_eq!(io.personas().await.len(), 2, "the keychain back, the Member is proposed");
    assert!(file_of(&dir, &member.member_device).is_file());
}

// Normal, then abnormal twice: a refused keychain write leaves no file and no Persona; a refused
// file write after the item leaves the item and no Persona, which shows the item came first.
// verifies: LLR-kna4zj
#[tokio::test]
async fn propose_founding_writes_the_item_before_the_file_and_nothing_when_the_keychain_refuses() {
    let (_guard, keychain) = fresh_keychain();
    let io = handle("creation-order", "a", &FakeChain::new());
    let dir = store_dir("creation-order", "a");
    let alice = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap().founder;
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(alice.as_bytes()))]);
    assert!(file_of(&dir, &alice).is_file());

    keychain.refuse_as(Some(Refusal::Unavailable));
    let refused = io.propose_founding_organisation("Bob", "Doe", "bob", "Acme").await;
    keychain.refuse_as(None);
    assert!(matches!(refused, Err(CreateError::Keychain(_))), "{refused:?}");
    assert_eq!((io.personas().await.len(), names_in(&dir).len(), keychain.items().len()), (1, 1, 1), "nothing kept for Bob");

    #[cfg(unix)]
    {
        common::handles::make_read_only(&dir);
        let refused = io.propose_founding_organisation("Carol", "Doe", "carol", "Acme").await;
        common::handles::make_writable(&dir);
        assert!(matches!(refused, Err(CreateError::Store(StoreFailure::Dropped { .. }))), "{refused:?}");
        assert_eq!((io.personas().await.len(), names_in(&dir).len()), (1, 1), "Carol dropped, no file");
        assert_eq!(keychain.items().len(), 2, "the item was written before the file was tried");
    }
}
