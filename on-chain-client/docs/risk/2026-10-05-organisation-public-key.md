# The Organisation public key, by its name — derived requirement assessment

REQ-54txzh supersedes REQ-2qa5r5 by renaming one field, the Organisation's
"signing key", to the *Organisation public key* (owner, 2026-10-04: the field
is the Organisation's X25519 key-agreement key and never a signing key). The
widths it requires are unchanged, so it realises RC-sxjnx9 against the same
hazard exactly as REQ-2qa5r5 did, and the test that verified REQ-2qa5r5 now
verifies both. No hazard impact: the change is to a name, not to what the
software reads, refuses or reports. This unit never interpreted the field, so
the key's change of scheme from ed25519 to X25519 reaches no code here. The
validity check belongs to org-node, and it covers state reads only: org-node
parses the key into its own type when it reads Organisation state
(`OrgState::from_chain`, in `org-node/src/chain.rs`). The
`GenesisInitialized` and `RootUpdated` events also carry the key, as an
unchecked `OrgPubKey`. Nothing reads those payloads today, so no unchecked key
reaches a decision; a consumer that starts to read them must parse the key
first, or it takes bytes that may not be an X25519 key. One state read in
org-node does not parse the key either (noted 2026-10-05 by review round 3,
finding-12): the preflight check `check_contract` (`org-node/src/preflight.rs`)
calls `get_org_state` and reports only that the call succeeded and the epoch
it returned. It is a diagnostic of reachability, it acts on nothing it read,
and the key never leaves it, so no unchecked key reaches a decision there
either.
assesses: REQ-54txzh
