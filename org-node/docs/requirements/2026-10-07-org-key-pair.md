# The Organisation key pair reaches every Member

Change `worktree-org-node-org-key-pair`, change 3 of the chain-authority
sequence (`org-node/docs/requirements/2026-10-06-chain-authority.md`,
"Sequencing"). Resolves PR-szkat6, PR-ve9zw8 and PR-xwek5e.

Owner rulings (2026-10-06):

- Moving iroh and peer communication out of org-node (change 2 of the
  sequence) is not this sequence's work: the `org-io` session does it. This
  change is built on org-node's current transport.
- There is one Organisation private key per epoch, shared Organisation-wide:
  no node has a key of its own. A node holds the Organisation's key in its
  record and passes on that same key. An administrator's node draws a new one
  at genesis and for every provisional update (below). (Clarified by the owner
  2026-10-06.)
- The Organisation secret is gone. What a node hands a Member is the
  Organisation private key, the secret half of the X25519 Organisation key
  pair whose public half is the on-chain `org_pub_key`.
- A message that sends the latest Organisation information, to a new Member or
  to an existing one, always carries the Organisation private key. Its absence
  is an error in parsing that message, for every receiver alike. There is no
  separate "missing key" check in the admission journey: a joiner is admitted
  provisionally by the administrator from the Invite reply, the administrators
  update the chain, and the Organisation information, key included, is then
  sent to the joiner's Devices, which check it against the chain.
- A revocation is a different message. In the owner's model it carries only a
  proof that the Member's Device is removed and no other Organisation
  information; the `org-io` session builds that. Until then this change splits
  the Wire message into two kinds: *Organisation information*, which always
  carries the key, and *revocation*, which carries none and after which the
  receiver keeps the key it holds. This supersedes the 2026-10-05 ruling that a
  message with no key keeps the stored key only while it matches the chain and
  clears it otherwise.
- The kind depends on the recipient, not on the operation (ruling of
  2026-10-06, later the same day): a Device the committed record no longer
  lists, the one being revoked, receives a revocation; every Device the record
  lists receives Organisation information, key included, whatever the change
  was. Admitting and revoking are the operations org-node has today; updating
  a Member and adding, revoking or cycling a Device key are future ones, and
  they send Organisation information too.
- A revocation is for the revoked Devices alone (ruling 2026-10-06): a change
  in Organisation information goes to every Device of every new and current
  Member, and a revocation, with its proof, goes only to the revoked Devices,
  every Device of a revoked Member. A Device the verified record still lists
  refuses a revocation, so a relay cannot hold a current Device on an earlier
  epoch's key by relabelling Organisation information. Sending to all those
  Devices is the `org-io` session's work (the app's fan-out problem report,
  `app/docs/problems/2026-10-07-update-fan-out.md`).
- The invite identifier never travels between peers: it goes to the joiner in
  the Invite over a third-party channel and comes back to the inviter in the
  reply, which is all the administrator needs to build the provisional
  admission. A joiner's node matches a first admission against the
  Organisation alone. REQ-8amu2a and the app's invitation requirement
  (`app/docs/requirements/2026-10-06-invitation.md`) are amended in place.
- A new Organisation key pair is drawn whenever a new Membership root is
  calculated for an update to be written to the chain (ruling 2026-10-06,
  superseding the 2026-10-05 ruling that rotation is out of scope). A
  provisional update is a batch of changes (adding, updating or removing
  Members; adding, removing or cycling Device keys) made from the current
  Organisation information; calculating its root ends the batch and prepares
  the chain write, and that is where the key pair is drawn, not at each
  change. Today each admission or revocation is a batch of one. An
  administrator's node draws it; the chain write publishes its public half;
  once the update verifies against the chain it is the Organisation's key for
  the new epoch, and only the Devices the new record lists receive it.

**REQ-szq3ud**: The software shall carry, in every Wire message it sends that
holds Organisation information, the Organisation private key of the epoch the
message's update reaches, as the sending node's record of that Organisation
holds it, and shall take no Organisation key or secret from its caller.
satisfies: derived

**REQ-stx9v3**: The software shall, when it calculates the resulting
Membership root of a provisional update, draw a fresh X25519 Organisation key
pair, distinct from the record's current Organisation public key and from
every Member-as-a-group key and DevicePublicKey in the resulting record; keep
its private key with the provisional update in the encrypted store; and give
its public key as the update's Organisation public key for the chain write.
satisfies: derived

**REQ-jy6ybw**: The software shall, when it commits its own provisional update
after it verified against the chain, replace the Organisation private key its
record holds with the one the provisional update holds.
satisfies: derived

**REQ-c29s93**: The software shall refuse with a typed error, leaving the
store unchanged, a received Wire message of the Organisation-information kind
that carries no Organisation private key or whose key does not decode as 32
bytes, before it reads the chain. (implements: RC-9cefcn)
satisfies: derived

**REQ-bwx7eg**: The software shall refuse with a typed error, leaving the
store unchanged, a received Organisation-information Wire message whose
Organisation private key has an X25519 public half different from the
`org_pub_key` the chain holds for that Organisation, whether or not the node
holds a record of it. (implements: RC-9cefcn)
satisfies: derived

**REQ-ju6vn2**: The software shall, when it commits a received
Organisation-information Wire message, store the Organisation private key the
message carries in its record of that Organisation, on a first admission and
on an update alike.
satisfies: derived

**REQ-3dsweu**: The software shall send a committed update as a Wire message
of the Organisation-information kind only to a Device the committed Membership
record lists, and to a Device the previous record listed and the committed one
does not only the revocation of REQ-ps2gy2, which carries no Organisation
private key, Envelope, Change set or record snapshot; it shall send nothing for
that update to any other Device.
satisfies: derived

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3 of
`docs/plans/2026-10-06-org-io-roadmap.md`; owner ruling 2 on the sweep of
`fdf4e77`: a revoked device receives only an absence proof).* This said a
revocation is "a Wire message of the revocation kind, which carries no
Organisation private key" — it carried the committed Envelope — sent "to any
other Device", and that a received revocation is refused when the receiving
Device is still listed after its verified Membership record. The revocation
now holds only the revoked Device's identity and an absence proof
(REQ-ps2gy2), and it goes only to a Device the update removed. The "still
listed" refusal is REQ-m2xh8q's: an absence proof for a Device the chain's
record lists does not verify, so a relay that relabels Organisation
information as a revocation has nothing to relabel it into. "Any other
Device" was narrowed because a Device neither record lists has no notice to
receive.

**REQ-vxqc5g**: The software shall refuse with a typed error, leaving the
store unchanged, a received Wire message of the revocation kind about an
Organisation it holds no record of.
satisfies: derived
