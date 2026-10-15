# Problems — S4's independent review, round 2

Recorded 2026-10-15 from round 2 of the independent review of change
`worktree-org-io-transport` (S4), by owner ruling of 2026-10-15: the medium
findings and the mechanical low ones are fixed in this change, the larger
low ones are filed here open.

**PR-aa7prn**: `DeviceSecret::lend_to_keychain` returns whatever its writer
returns, so a caller can hand the 32 seed bytes back out of the lend
(`|seed| *seed`), unzeroised and outliving it.
affects: LLR-yp69jg, LLR-37k2s3, REQ-26mjqa, SDD-fysk96
opened: 2026-10-15
status: resolved
Root cause: the lend's return type was the writer's own, generic over anything; fixed by `lend_to_keychain<E: std::error::Error>(&self, write: impl FnOnce(&[u8; 32]) -> Result<(), E>) -> Result<(), E>` (LLR-yp69jg amended 2026-10-15); reproduced by `lend_to_keychain_returns_no_value_the_writer_could_carry_the_bytes_in` (org-node/tests/key_custody.rs), and the lend's own test now compares the seed inside the writer.

Where: `pub fn lend_to_keychain<R>(&self, write: impl FnOnce(&[u8; 32]) -> R)
-> R` (`org-node/src/keys.rs`), round 2 finding-3. With `R` generic the
lend's return is a channel for the bytes themselves: `key_custody.rs`'s own
test of the lend copied the seed out that way, as did test helpers in
org-node and org-io. LLR-yp69jg's stated signature permits it, and
LLR-37k2s3's "no public operation returns its bytes" does not hold while it
does. The public-operation scan (`device_secret_offers_only_its_four_public_operations`)
looks for `-> [u8` in the signature and cannot see a generic return.

**PR-qpam4k**: a committed update whose outgoing body would exceed
`MAX_MESSAGE` is refused inside `commit_update` and `reconcile`, after
org-io has already written the update to the chain, so the node can never
commit an update of its own that the chain has executed.
affects: LLR-gdfqn9, REQ-6ehufy, SDD-bm27xt
opened: 2026-10-15
status: open

Where: `outgoing_bodies` (`org-node/src/service.rs`), called from the one
commit step that `commit_update` and `reconcile` share, round 2 finding-9.
Each body is `encode_message`'s output, and `encode_message` refuses an
encoding over `MAX_MESSAGE` (1 MiB, `org-node/src/wire.rs`, LLR-sc6zuh). An
`OrgInformation` body carries the Envelope's change set and the whole record
snapshot as it stood before the commit, so its size grows with the
Organisation. The refusal comes after the chain write: the chain holds the
update's root and key, and the commit, which would bring this node's record
up to them, returns the encode error and changes nothing. Every retry and
every startup reconcile builds the same body and is refused again, so a
large enough Organisation could never commit its own executed update; its
record stays one epoch behind the chain, and every later update this node
builds is checked against the stale record. Nothing states this behaviour:
no requirement bounds the Organisation's size against `MAX_MESSAGE`, the
bound is not checked before the chain write, and no abnormal test drives a
commit over it. A fix needs a ruling on where the bound belongs (refuse the
update before it is written; or commit the record and report the bodies it
could not build).

**PR-bjaq7j**: `Invite::parse` and `InviteReply::parse` accept Blobs that
do not encode back to the same text: trailing bytes after the postcard value
are ignored, surrounding whitespace is trimmed, and an Invite's names are
unbounded and its inviter Device key list may be empty.
affects: LLR-b7wgpf, REQ-yazum3, SDD-xshv5b
opened: 2026-10-15
status: open

Where: `unarmour` (`org-node/src/invitation.rs`), round 2 finding-11. It
decodes with `postcard::from_bytes`, which ignores any bytes left after the
value; the Wire decode (`org-node/src/wire.rs`) uses `take_from_bytes` and
refuses leftovers. It trims the text before the Base64 decode. `WireInvite`'s
`org_name` and `invitee_name` are plain `String`s, parsed with no bound, and
`inviter_device_keys` may be empty. LLR-b7wgpf (clarified 2026-10-15) now
states what the parse accepts; the fix is to refuse trailing bytes as the
Wire decode does, bound the two names, and decide whether an Invite with no
inviter Device key is refused.
