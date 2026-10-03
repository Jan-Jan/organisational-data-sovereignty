# Problem reports — a removed device can be handed back an earlier key

**PR-z463w5**: the replacement-key check added by PR-zz4exm's fix compares the
replacement only with the member's current member-as-a-group key, so a removal
may still install a key the removed device held earlier, and `rotate_p2p_key`
accepts any key — including one the removed device held — after a removal.
affects: REQ-ewdg2q, REQ-r784fu
opened: 2026-10-03
status: open

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
