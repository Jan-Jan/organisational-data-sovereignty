# Problem reports — a removed device can be handed back an earlier key

**PR-z463w5**: the replacement-key check added by PR-zz4exm's fix compares the
replacement only with the member's current member-as-a-group key, so a removal
may still install a key the removed device held earlier, and `rotate_p2p_key`
accepts any key — including one the removed device held — after a removal.
affects: REQ-ewdg2q, REQ-r784fu
opened: 2026-10-03
status: resolved

Found by the independent review of the change that resolved PR-zz4exm
(`worktree-fix-stale-problems`), not by a failing test. REQ-ewdg2q's purpose is
that "a removed device cannot derive access from the key it held while
enrolled". Its stated clause rejects only "the key being replaced", and the
code now meets that letter: `delete_p2p_device` and `emergency_isolate_member`
refuse the current key (`org-members/src/trie.rs`). Three direct-API sequences
still return a removed device to a key it held:

1. Member at key K0 with device D; `rotate_p2p_key(K1)`; then
   `delete_p2p_device(D, K0)` — accepted, and D held K0 while enrolled.
2. `delete_p2p_device(D, K1)` from K0; then `rotate_p2p_key(K0)` — accepted,
   and the member is back on the key D held.
3. The same as 1 with `emergency_isolate_member(K0)` in place of
   `delete_p2p_device` — accepted, so REQ-r784fu and LLR-w92psx are affected
   as LLR-s97ywt is.

The trie records no key history, so neither operation can tell. Whether the
software should refuse a previously held key — which needs a key history in
the leaf or in the trie, and changes the commitment the root makes — or
whether this is the key-distribution layer's responsibility is a requirement
question for the owner (`grill-requirements`), not a fix to make under the
change that found it. The same question covers `rotate_p2p_key` (LLR-k89ahd),
which the PR-zz4exm change deliberately left alone because it removes no
device.

A second, narrower facet of the same question, raised by the second review
round: the new guard compares keys by their 32 encoded bytes
(`VerifyingKey` equality in ed25519-dalek 2.2 is byte equality), not as curve
points. A non-canonical encoding of the current key's point would pass it.
Honest key generation never produces one, so it needs a deliberately odd key
from the administrator's own caller; whether such an encoding is "the same
key" depends on how the key-distribution layer derives access from it. What
counts as "a key the device held" — by encoding, by point, or by history — is
the one requirement this report asks the owner to state.

A third facet, raised by the fifth review round and the most direct of the
three: a removal may install the removed device's *own* key as the member's new
key. `delete_p2p_device(id, d1, P2pMemberKey::new(*d1.verifying_key()))` is
accepted, and so is `emergency_isolate_member` with a removed device's key; the
removed device holds the matching secret and so controls the new key outright.
Confirmed by a probe test in that review, not committed. Whether the member key
must differ from every device key the member held — currently or ever — belongs
to the same owner decision.

Bearing on the register: HAZ-s39gbh's residual is not acceptable and was not
re-scored by the PR-zz4exm change; this report is a further reason it stands,
and it should be weighed with RC-mqtks7's wire-path bypass when HAZ-s39gbh is
next re-evaluated.

Resolved, 2026-10-03, by owner ruling. The ruling: a key **no longer held** — a
member key used earlier and since replaced, or the key of a device removed
earlier — and a non-canonical encoding of a held key are not checked and are
not defects: the requirement's clause — reject "the key being replaced" — is
the whole of what it asks, and key hygiene beyond it belongs to the caller.
None of the three sequences above is refused: each installs a key that is no
longer held anywhere when it is installed. (The ruling as first given said
"only the *current* key is refused"; two further owner decisions the same day,
on branch `worktree-rotate-same-key`, plan
`docs/plans/2026-10-03-rotate-same-key.md`, narrowed it to the wording above.)
Those two decisions: `rotate_p2p_key` refuses the exact current key
(`P2pKeyNotReplaced`, LLR-k89ahd), and every key in the organisation is held in
one place, so a replacement key held anywhere before the operation — another
member's key or any enrolled device key, the key of the device being removed
included — is refused with `DuplicateKey` (LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx).
The third facet (the removed device's own key) is therefore now refused; the
earlier-key sequences and the encoding facet are not.

The duty the ruling transfers is written down where integrators read it:
security check 11 in `org-members/README.md`, and the doc-comments of
`rotate_p2p_key`, `delete_p2p_device` and `emergency_isolate_member`. The risk
is not removed — a device holding the secret of a key no longer held, which the
caller supplies, regains access — it is carried by the caller, and HAZ-s39gbh's
register entry says so.
