# Trim worktree-person-shared-types to what it and chain-authority agree on: Implementation Plan

**Goal:** Remove from this branch every behaviour, ledger item and test that contradicts the chain-authority rulings (the sender checks, the required Invite, the Invite checks this branch added, the Invite-sourced administrator key), turn this branch's supersessions into in-place amendments wherever the old item's text is no longer true, and keep the shared types, the X25519 keys and the separate Organisation key pair, so that chain-authority can build on this branch afterwards.
**Implements:** amended in place: REQ-ag6kqm, REQ-nhe2zu, REQ-xa6smf, REQ-ztdza4 (chain-authority's text, copied), REQ-gju89b (reverted to master's text), RC-pm9kmx, RC-b6mydy (proposed text, open question Q2); reworded and kept unmerged: REQ-txvtm9 (the epoch rule alone), REQ-ech45n ("only" dropped); kept: REQ-8jb4ny, RC-95dgg8. LLRs and SDDs: the inventory below. App: REQ-kn5rtx (rationale amendment rewritten).
**Safety class:** C (org-node, app, org-members, on-chain-client, person; no per-item overrides)
**Verification:** every `verify_commands` entry of `org-node/.guardrails/config.yaml`, `app/.guardrails/config.yaml`, `org-members/.guardrails/config.yaml`, `on-chain-client/.guardrails/config.yaml` and `person/.guardrails/config.yaml`; `check-trace.sh` and `check-ids.sh` per unit; `check-units.sh`; the three chopsticks targets compile (`--no-run`). Exact commands in T13.

Written 2026-10-05 with the `plan-change` skill. Base: branch
`worktree-worktree-person-shared-types`, HEAD `ef6e261`, tree `edab395`, master
`06c357b` merged.

## Why this change, and what it is not

The owner ruled on 2026-10-05, for change `worktree-org-node-chain-authority`
(worktree `.claude/worktrees/org-node-chain-authority`, read only), that
nothing about the sender of an Envelope is checked, that an Envelope carries
no authority signature, that the on-chain root at a newer epoch is the sole
authority, and that every administrator field leaves org-node. Those rulings
are recorded in that worktree's
`org-node/docs/requirements/DRAFT-worktree-org-node-chain-authority-chain-authority.md`,
`app/docs/requirements/DRAFT-worktree-org-node-chain-authority-invitation.md`
and `on-chain-client/docs/requirements/DRAFT-worktree-org-node-chain-authority-chain-write.md`,
and in its commit `48969ab`, which amends `2026-09-09-verify-and-commit.md`,
`2026-10-03-member-identity.md` and `2026-10-04-type-safety.md` in place.

This branch was written before those rulings. It replaced the signature with
a sender-device check on every Receive operation (REQ-7h7qp3, RC-2e6k44),
made an imported Invite mandatory for a first admission, validated the Invite
against the chain, and took the record's administrator key from the Invite.
The owner chose to trim this branch to what both designs agree on, so no item
that contradicts the chain-authority rulings reaches master. Chain-authority
then builds on this branch.

This change does **not** remove the Invite, the Join request, the pending
Invite, `admin_member_key`, `admin_device_key` or `admin_persona_for_org`.
Those stay as master `1feb608` left them, with `admin_member_key` typed as a
`PersonPublicKey` because member keys are X25519. Removing them is
chain-authority's change 1. This change does not distribute the Organisation
private key either. That is chain-authority's change 2.

## Ledger convention (owner, 2026-10-05)

Retired behaviour is amended **in place** with a dated note, as
chain-authority does. It is not superseded. Every supersession this branch
wrote is therefore judged one of four ways:

- **Retired (P1).** The replacement states behaviour this trim removes. The
  replacement is deleted (it is unmerged). The old item loses its
  `superseded-by:` and is amended in place to a testable statement of the
  behaviour after the trim, with a dated note.
- **Folded (P2).** The replacement states behaviour that remains, and no test
  can honestly verify both the old text and the new. The old item is amended
  in place to the replacement's text, trimmed of anything this trim removes,
  with a dated note. The replacement is deleted. Tests carry the old ID only.
- **Kept, honest (P3).** One test honestly verifies both texts. The
  supersession stays.
- **Kept by the owner's KEEP rule (P4).** The owner named the replacement in
  the KEEP list. It stays as it is, supersession included. Where the old
  item's text is no longer true, that is open question Q4.

A dated note on an amended item never names a deleted ID: `check-trace.sh`
reports any reference to an undefined ID as `DANGLING-REF`. Notes say "an
item withdrawn before merge" instead.

The note every in-place amendment in this plan uses, unless a different text
is given for the item:

> *Amended 2026-10-05 (owner ruling of that day, change
> `worktree-org-node-chain-authority`; written by change
> `worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).*
> <one or two sentences: what the item said before, and why that no longer
> holds.>

Every sentence a task below writes into such a note is given in full in the
task.

## Texts copied from chain-authority

Copied byte for byte from
`/Users/jan-jan/Coding/2-tier-access-control/.claude/worktrees/org-node-chain-authority/org-node/docs/requirements/2026-09-09-verify-and-commit.md`
at that worktree's commit `48969ab` (its working tree was clean when this plan
was written; `git -C <that worktree> diff master...HEAD` shows the hunks). The
four item blocks and their notes go into this branch's copy of the same file,
replacing this branch's text of the same items, so both branches carry
identical bytes and the later merge of chain-authority onto this branch does
not conflict on them.

**C1 — REQ-ag6kqm** (replaces lines 42–47 of this branch's file, including
this branch's `superseded-by: REQ-7h7qp3`):

```markdown
    **REQ-ag6kqm**: The software shall decide whether to commit a received
Envelope from the Organisation it names, its Sequence number, its Change set
and the Organisation state on the chain alone, and shall check no signature
and no Member's or device's key in doing so. (implements: RC-pm9kmx)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item first required the Envelope's signature to verify under the
Published signing key before decoding. The right to change an Organisation's
data lies in its on-chain multisig proxy, and org-node has no concept of an
administrator, so an Envelope carries no signature and the on-chain Membership
root at a newer epoch is the sole authority. The item keeps its ID and states
the rule that replaced it; retiring it is not expressible in the trace gate.
```

**C2 — REQ-nhe2zu** (replaces lines 55–63, including this branch's
`superseded-by: REQ-txvtm9`):

```markdown
    **REQ-nhe2zu**: The software shall, when an Envelope names the expected
Organisation, carries a Sequence number greater than the highest committed,
and carries a Change set whose recomputed Membership root equals the root of
an Organisation state with an epoch greater than the last committed, commit
the applied Change set as its record together with that epoch and that
Sequence number as the new high-water mark.
(implements: RC-6a2dke, RC-e5atck, RC-m4r75s)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):*
the clause "carries a valid signature under the published signing key" is
removed, with REQ-ag6kqm's amendment.
```

**C3 — REQ-xa6smf and REQ-ztdza4, with their shared note** (replaces the
block from `**REQ-xa6smf**` through REQ-ztdza4's `satisfies: derived`, lines
99–113; this branch never edited either item, so the block is master's):

```markdown
    **REQ-xa6smf**: The software shall commit a first admission to an Organisation
that verifies against the chain whichever Device key the connection
authenticated, and shall require no Invite to have been imported for it.
(implements: RC-b6mydy)
satisfies: derived

    **REQ-ztdza4**: The software shall commit an update to an Organisation it holds
a record of that verifies against the chain whichever Device key the
connection authenticated, whether or not that key is in the Membership record
before or after the update. (implements: RC-b6mydy)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
REQ-xa6smf first required a first admission's sender to be the administrator's
Device key named by an imported Invite; REQ-ztdza4 required an update's sender
to be in the verified Membership record. The owner ruled that nothing about the
sender is checked: authority is the chain's, Invites leave org-node for the
app, and org-node has no administrator. A chain-valid update delivered by any
peer is harmless because it matches the chain. Both items keep their IDs and
state the rule that replaced them.
```

**Not copied, on purpose.**

- REQ-qn2erx: chain-authority removes the Join request and Invite imports from
  it. This trim keeps both imports, so REQ-qn2erx stays as master has it.
- REQ-d9g6nt: chain-authority rewords "the founding administrator". This trim
  does not touch REQ-d9g6nt.
- RC-pm9kmx and RC-b6mydy: chain-authority's DRAFT says both are amended in
  place, but no text exists in its worktree yet. This plan writes proposed
  text (T9) for chain-authority to copy back. That is open question Q2.

**REQ-nhe2zu and REQ-txvtm9 (decided here).** Chain-authority's REQ-nhe2zu
keeps "greater than the highest committed" and has no epoch clause. This
branch's REQ-txvtm9 adds "equal to the epoch" (RC-95dgg8, owner ruling
2026-10-05, review round 1). REQ-txvtm9 stays as a separate item that states
only the epoch rule, implements RC-95dgg8 alone, and supersedes nothing.
REQ-nhe2zu gets chain-authority's exact text. The two together are the
commit rule. Folding the epoch clause into REQ-nhe2zu would make the two
branches' texts of a shared item differ, which is what the owner asked to
avoid.

## Inventory of ledger items touched

Line numbers are this branch's at `ef6e261`. "Del" deletes the definition
and every reference to the ID in the unit's ledgers, `src` and `tests`.

### org-node requirements

| ID | File:line | Action | New text |
|---|---|---|---|
| REQ-ag6kqm | `org-node/docs/requirements/2026-09-09-verify-and-commit.md:42` | amend in place, un-supersede | C1 |
| REQ-nhe2zu | same file `:55` | amend in place, un-supersede | C2 |
| REQ-xa6smf | same file `:99` | amend in place | C3 |
| REQ-ztdza4 | same file `:106` | amend in place | C3 |
| REQ-gju89b | same file `:37` | revert `(implements: RC-pm9kmx, RC-2e6k44)` to master's `(implements: RC-pm9kmx)` | master's |
| REQ-7h7qp3 | `org-node/docs/requirements/2026-10-05-envelope-authenticity.md:8` | Del (P1) | — |
| REQ-txvtm9 | same file `:17` | reword, drop `supersedes:` | T8 text |
| REQ-8jb4ny | same file `:29` | keep | — |
| REQ-ech45n | same file `:35` | reword: drop "only" | T8 text |
| REQ-qn2erx, REQ-d9g6nt | `2026-10-04-type-safety.md:59`, `2026-10-03-member-identity.md:33` | keep (not touched) | — |

### org-node risk controls

| ID | File:line | Action |
|---|---|---|
| RC-pm9kmx | `org-node/docs/risk/2026-09-09-org-node-hazards.md:168` | amend in place, un-supersede (T9 text, Q2) |
| RC-b6mydy | same file `:437` | amend in place (T9 text, Q2) |
| RC-2e6k44 | `org-node/docs/risk/2026-10-05-envelope-authenticity.md:16` | Del (P1) |
| RC-95dgg8 | same file `:80` | keep; its surrounding prose reworded (T9) |

### org-node software items and low-level requirements

`dec` = `org-node/docs/architecture/2026-10-03-decomposition.md`,
`ts` = `org-node/docs/architecture/2026-10-04-type-safety.md`,
`ue` = `org-node/docs/architecture/2026-10-05-unsigned-envelope.md`.

| Old ID (master) | Old at | Replacement (this branch) | New at | Action |
|---|---|---|---|---|
| SDD-sxp8hb | dec:165 | SDD-wxu2cg | ue:50 | P2 fold |
| LLR-e58j8m | dec:187 | LLR-bg6xty | ue:66 | P2 fold |
| LLR-ctzkv7 | dec:200 | LLR-9umxv2 | ue:73 | P2 fold |
| LLR-na7p4w | dec:206 | LLR-by65qx | ue:117 | P2 fold (own text) |
| LLR-9fvb3y | dec:211 | LLR-by65qx | ue:117 | P2 fold (own text) |
| SDD-kk2y3e | dec:220 | SDD-m5uxzf | ue:110 | P2 fold |
| LLR-e7s4ye | dec:227 | LLR-by65qx | ue:117 | P2 fold (own text) |
| LLR-p8uu47 | dec:233 | LLR-w8ny8z | ue:125 | P2 fold |
| LLR-ybn5pr | dec:242 | LLR-by65qx | ue:117 | P2 fold (own text) |
| LLR-cs4mpb | dec:247 | LLR-by65qx | ue:117 | P2 fold (own text) |
| LLR-9sknpa | dec:252 | LLR-by65qx | ue:117 | P2 fold (own text) |
| LLR-pzde8b | dec:257 | LLR-by65qx | ue:117 | P2 fold (own text) |
| SDD-na9nc3 | dec:291 | SDD-my6mvj | ue:139 | P2 fold |
| LLR-mcdh85 | dec:305 | LLR-qgjw4n | ue:151 | P1 |
| LLR-g9vmbx | dec:556 | LLR-wg4hj2 | ue:382 | P2 fold |
| LLR-rv4vux | dec:585 | LLR-h57d27 | ue:389 | P2 fold |
| LLR-s7yu4k | dec:705 | LLR-23kqye | ue:375 | P2 fold |
| SDD-rx2yvy | dec:763 | SDD-5sxp7v | ue:183 | P2 fold |
| LLR-rb8r65 | dec:782 | LLR-2ad4du | ue:194 | P2 fold |
| LLR-ghja3x | dec:789 | LLR-53phvg | ue:201 | P2 fold |
| LLR-vdyu65 | dec:837 | LLR-28qhcd | ue:208 | P2 fold |
| SDD-8cpyfa | dec:914 | SDD-zqc75b | ue:226 | P2 fold |
| LLR-37cj3n | dec:950 | LLR-ds2gyj | ue:246 | P1 |
| LLR-mbjfq8 | dec:972 | LLR-fj3f24 | ue:257 | P1 |
| LLR-xq9nrq | dec:1002 | LLR-rys5nx | ue:270 | P1 (Q1) |
| LLR-e5c9ud | dec:1015 | LLR-p9xjze | ue:277 | P2 fold |
| SDD-72ddm6 | dec:1068 | SDD-cp8g7z | ue:324 | P2 fold |
| LLR-3q63zv | dec:1103 | LLR-6wpk63 | ue:332 | P1 |
| LLR-6dc598 | dec:1170 | LLR-8az5vt | ue:346 | P2 fold |
| LLR-tax3pm | dec:1177 | LLR-3jybhn | ue:353 | P2 fold |
| LLR-8hdu9x | dec:1215 | LLR-tpb9xc | ue:359 | P2 fold |
| LLR-ayrdr8 | ts:103 | LLR-vp62sc | ue:496 | P2 fold |
| LLR-8bum44 | ts:256 | LLR-9s3nqp | ue:560 | P2 fold (InvalidInvite clause removed) |
| LLR-mmdu38 | ts:118 | LLR-en9p5c | ue:475 | P4 keep (Q4) |
| LLR-56hc77 | ts:170 | LLR-x35s6p | ue:529 | P4 keep (Q4) |
| LLR-bwb9pu | ts:205 | LLR-zyw5r2 | ue:548 | P4 keep (Q4) |

Master items this branch never superseded, whose text the trim makes false:

| ID | At | Action |
|---|---|---|
| LLR-j83kc8 | dec:939 | amend in place (invite cross-check removed) |
| LLR-u6rq4s | dec:945 | amend in place (post-verification check removed) |
| LLR-y2v8v2 | dec:984 | amend in place (drop "the pending invite it cross-checks the sender against") |
| LLR-9zfnmb | dec:892 | amend in place (drop "so the cross-check LLR-j83kc8 performs has one answer") |
| LLR-zj88e6 | dec:873 | amend in place (drop the "pin the first admission's sender" purpose) |

This branch's new LLRs:

| ID | At | Action |
|---|---|---|
| LLR-qgjw4n, LLR-ds2gyj, LLR-fj3f24, LLR-6wpk63 | ue:151, 246, 257, 332 | Del (P1) |
| LLR-5svrw8 | ue:286 | Del (P1) |
| LLR-rys5nx | ue:270 | Del (P1, Q1) |
| LLR-3jjgtw, LLR-98ufry, LLR-9f5hmr | ue:81, 86, 158 | keep |
| LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8, LLR-322xfu | ue:400, 407, 413, 515 | keep |
| LLR-en9p5c, LLR-x35s6p, LLR-zyw5r2 | ue:475, 529, 548 | keep (P4) |

### org-node problem reports

| ID | At | Action |
|---|---|---|
| PR-szkat6 | `org-node/docs/problems/2026-10-04-org-key-conflation.md:3` | back to `status: open`, `resolution:` removed, dated note (T10) |
| PR-u4c2vp | `org-node/docs/problems/2026-09-09-org-node-problems.md:192` | resolution rewritten: not a defect by ruling (T10, Q3) |
| PR-vkw22m | `org-node/docs/problems/2026-10-04-non-strict-verify.md:3` | stays resolved; resolution reworded to cite surviving IDs (T10) |
| PR-ve9zw8 | `org-node/docs/problems/2026-10-05-org-secret-unauthenticated.md:6` | stays open; `affects:` retargeted, text reworded for "any peer", closing path noted (T10, Q2) |

### Other units

| Item | At | Action |
|---|---|---|
| org-members, on-chain-client, person | all | untouched (KEEP) |
| REQ-2qa5r5 → REQ-54txzh, LLR-nq7nhg → LLR-hezpr7 | on-chain-client | P3 keep: both pairs rename "signing key" to "Organisation public key" over the same bytes and offsets, and `type_widths.rs:124` / `decode_org_state.rs:110` verify both texts |
| LLR-t3p9zk → LLR-st6j2r | org-members | P3 keep: the same rule under `person`'s type names, mapped back by `From` |
| LLR-k6dhz7 → LLR-4sm3b5 | org-members | P4 keep (Q4): LLR-k6dhz7 says neither constructor refuses a small-order key; LLR-4sm3b5 says both do |
| REQ-kn5rtx rationale amendment | `app/docs/requirements/2026-09-14-tauri-shell.md:229–277` | rewritten (this branch's own, unmerged) |
| RC-3rddh7 note, HAZ-9fmhm4 note, REQ-kn5rtx assessment note | `app/docs/risk/2026-09-14-app-hazards.md:527–544`, `:959–963`, `:1184–1200` | rewritten |

### Counts

| Action | Count | IDs |
|---|---|---|
| Delete, P1 (behaviour retired) | 8 | REQ-7h7qp3, RC-2e6k44, LLR-qgjw4n, LLR-ds2gyj, LLR-fj3f24, LLR-6wpk63, LLR-5svrw8, LLR-rys5nx |
| Delete, P2 (folded into the old item) | 22 | SDD-wxu2cg, SDD-m5uxzf, SDD-my6mvj, SDD-5sxp7v, SDD-zqc75b, SDD-cp8g7z, LLR-bg6xty, LLR-9umxv2, LLR-by65qx, LLR-w8ny8z, LLR-2ad4du, LLR-53phvg, LLR-28qhcd, LLR-p9xjze, LLR-8az5vt, LLR-3jybhn, LLR-tpb9xc, LLR-23kqye, LLR-wg4hj2, LLR-h57d27, LLR-vp62sc, LLR-9s3nqp |
| Amend in place, un-supersede | 38 | REQ-ag6kqm, REQ-nhe2zu, RC-pm9kmx; the 6 old SDDs; LLR-e58j8m, ctzkv7, na7p4w, 9fvb3y, e7s4ye, p8uu47, ybn5pr, cs4mpb, 9sknpa, pzde8b, mcdh85, g9vmbx, rv4vux, s7yu4k, rb8r65, ghja3x, vdyu65, 37cj3n, mbjfq8, xq9nrq, e5c9ud, 3q63zv, 6dc598, tax3pm, 8hdu9x, ayrdr8, 8bum44 (29 LLRs) |
| Amend in place, never superseded | 8 | REQ-xa6smf, REQ-ztdza4, RC-b6mydy, LLR-j83kc8, LLR-u6rq4s, LLR-y2v8v2, LLR-9zfnmb, LLR-zj88e6 |
| Reword, this branch's unmerged item | 3 | REQ-txvtm9, REQ-ech45n, REQ-gju89b (revert) |
| Keep, P3 (honest supersession) | 3 pairs | REQ-54txzh, LLR-hezpr7, LLR-st6j2r |
| Keep, P4 (owner KEEP, Q4) | 4 pairs | LLR-en9p5c, LLR-x35s6p, LLR-zyw5r2, LLR-4sm3b5 |
| Keep, untouched | 9 | REQ-8jb4ny, RC-95dgg8, LLR-3jjgtw, LLR-98ufry, LLR-9f5hmr, LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8, LLR-322xfu |
| Problem reports | 4 | PR-szkat6 reopened; PR-u4c2vp, PR-vkw22m, PR-ve9zw8 reworded |

No ID is minted by this plan.

## Code sites

| Site | Change | Task |
|---|---|---|
| `org-node/src/verify.rs:21-33` (`VerifyContext`) | drop `sender` and `accepted_senders` | T1 |
| `org-node/src/verify.rs:63-66` | drop the sender check; renumber the step comments | T1 |
| `org-node/src/verify.rs:1-4` | module doc: no "came from a device the receiver accepts" | T1 |
| `org-node/src/service.rs:518` (`record_devices`) | delete | T1 |
| `org-node/src/service.rs:1203-1212` (self-delete ctx) | no sender, no accepted set | T1 |
| `org-node/src/service.rs:919-962` (receive_and_verify, record and ctx) | no accepted set; first-admission arm: no Invite lookup, no sender check, no `InviteOrgKeyMismatch`; `admin_member_key` from the chain (Q1) | T1 (ctx), T2 (arm) |
| `org-node/src/service.rs:968-972` | delete the post-verification sender check | T3 |
| `org-node/src/service.rs:878-884` (doc comment) | no sender checks | T3 |
| `org-node/src/service.rs:974-977` (persona selection comment) | administrator key from the chain, not the Invite | T2 |
| `org-node/src/service.rs:706-717` (`import_invite`) | `crate::blobs::decode(blob)?` (master's error), comment without LLR-5svrw8 | T4 |
| `org-node/src/error.rs:10-15, 31-46` | delete `UnknownSender`, `InvalidInvite`, `InviteOrgKeyMismatch`, `NoImportedInvite` | T4 |
| `org-node/src/envelope.rs:1-4` | module doc: no "verification checks that device (REQ-7h7qp3)" | T1 |
| `org-node/src/blobs.rs:17-20` | doc comments: "or no Invite" stays true; no change | — |
| `app/src-tauri/src/events.rs:66-115` | drop the four variants from `classify_receive_error`; rewrite its comments | T5 |
| every `src` comment naming a deleted ID | reworded | T11 sweep |

## Tests

| Test (file:line at `ef6e261`) | Action | Task |
|---|---|---|
| `verify_against_chain.rs:118` `rejects_a_sender_outside_the_accepted_devices` | delete | T1 |
| `verify_against_chain.rs:134` `an_empty_accepted_set_accepts_no_sender` | delete | T1 |
| `verify_against_chain.rs:308` `rejects_unknown_sender_before_decoding_delta` | rewrite as `undecodable_change_set_bytes_are_refused_as_malformed_delta` | T1 |
| `verify_against_chain.rs` helpers `admin_sender`, `imposter`, `ctx` | `ctx(org)` only; the other two deleted | T1 |
| `admission_sender.rs:463` `a_revocation_relayed_by_a_non_member_is_refused_on_the_self_delete_path` | flip, master's name `a_revocation_relayed_by_a_non_member_is_still_acted_on` | T1 |
| `admission_sender.rs:2401` `pr_u4c2vp_…_is_refused_on_the_self_delete_path` | flip, master's name `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path` | T1 |
| `admission_sender.rs:2743` `a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_decoding` | flip to `MalformedDelta`, renamed `…_is_refused_as_malformed` | T1 |
| `service_stories.rs:453` `revocation_from_an_unknown_device_leaves_the_record_in_place` | refusal is now the chain's (`StaleEpoch`) | T1 |
| `chain_genesis_e2e.rs:313, :478`, `transport_handshake.rs:126`, `transport_networked.rs:154`, `fuzz_verify_against_chain/fuzz_target.rs:128` | `VerifyContext` without `sender` / `accepted_senders` | T1 |
| `admission_sender.rs:273` `first_admission_from_a_device_other_than_the_invites_admin_is_rejected` | flip, `…_is_committed` | T2 |
| `admission_sender.rs:2439` `a_first_admission_with_no_imported_invite_is_refused` | flip, master's name `…_rests_on_the_chain_alone` | T2 |
| `admission_sender.rs:685` `first_admission_without_an_imported_invite_is_rejected` | delete (the case is 2439's) | T2 |
| `admission_sender.rs:717` `first_admission_from_a_wrong_sender_is_rejected_before_its_snapshot_is_decoded` | delete (orders a removed check) | T2 |
| `admission_sender.rs:2844` `a_first_admission_whose_invite_names_another_organisation_key_is_refused` | flip, `…_is_committed` | T2 |
| `admission_sender.rs:2921` `an_invite_org_key_mismatch_is_refused_before_the_snapshot_is_decoded` | delete (orders a removed check) | T2 |
| `admission_sender.rs:2044` `a_first_admission_records_the_invites_administrator_…` | administrator key from the chain (Q1), renamed `a_first_admission_records_the_chains_key_the_secret_and_the_member` | T2 |
| `admission_sender.rs` `admit_b_directly` (`:250-255`) | its administrator-key assertion follows Q1 | T2 |
| `admission_sender.rs:381` `update_relayed_by_a_non_member_after_admission_is_rejected` | flip, `…_is_committed` | T3 |
| `admission_sender.rs:1977` `a_removal_relayed_by_the_member_it_removes_is_refused` | flip, `…_is_committed` | T3 |
| `admission_sender.rs:759` `update_relayed_by_the_device_it_removes_is_rejected` | delete (the case is 1977's) | T3 |
| `admission_sender.rs:2823`, `:2884` (invalid administrator keys refused at import as `InvalidInvite`) | delete (`persona_records.rs:510` covers all three keys) | T4 |
| `persona_records.rs:510` `an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored` | master's assertion `Chain("blob decode…")` | T4 |
| `value_types.rs:57` `every_rejection_variant_is_distinct_from_every_other` | drop the four deleted variants | T4 |
| `app/src-tauri/tests/receiver_events.rs` (five sites) | T5 table | T5 |
| every other `verifies:` line naming an affected ID | T6 table | T6 |

## Harness rules

One plain git command per Bash call; no compound line that mentions git; no
heredoc. Write files with Write/Edit; never `sed` a Rust test file. `~/.cargo`
is read-only: every cargo command runs with `CARGO_HOME=/tmp/cargo_home_fuzz`.
Quint simulator runs need `QUINT_HOME` set to a writable scratch directory
seeded from `~/.quint/rust-evaluator-v0.7.0`; never write `~/.quint`. Commits
use `git -c commit.gpgsign=false commit`, no Co-Authored-By line. Logs go to
`$S/trim-…` with
`S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad`.
All commands run from
`/Users/jan-jan/Coding/2-tier-access-control/.claude/worktrees/worktree-person-shared-types`.
Never write in `.claude/worktrees/org-node-chain-authority`.

Named commands:

- **ONODE(t)**: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test t`
- **ORG-NODE-GATE**: the first `verify_commands` line of `org-node/.guardrails/config.yaml`, prefixed with `CARGO_HOME=/tmp/cargo_home_fuzz`.
- **APP-GATE**: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy --test connection_status --test org_id_parsing --test receiver_events --test receiver_guard --test ipc`

## Tasks

T1 to T4 change org-node code and share `org-node/src/service.rs` and
`org-node/tests/admission_sender.rs`, so they run in order. T5 and T6 start
after T4. T7 to T10 touch only ledger files, disjoint from each other and
from T1–T6, and may run alongside anything. T11 and T12 run after all of
them. T13 is the gate.

### T1 — Verification checks no sender; the self-delete path checks nothing about the sender

**Files touched:** `org-node/src/verify.rs`, `org-node/src/service.rs`, `org-node/src/envelope.rs`, `org-node/tests/verify_against_chain.rs`, `org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`, `org-node/tests/chain_genesis_e2e.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`
**Parallel:** no (first)
**Trace:** REQ-ag6kqm, LLR-mcdh85, LLR-3q63zv, LLR-9sknpa, LLR-9fvb3y (texts from T7/T8; the annotations below name them)

Step 1 — red, the self-delete path (delete branch). In
`org-node/tests/admission_sender.rs`, replace the comment block and function
`a_revocation_relayed_by_a_non_member_is_refused_on_the_self_delete_path`
(`:450-500`) with master's test, adapted to this branch only where its API
changed:

```rust
// On the removal path nothing about the sender is checked (REQ-ztdza4 and
// LLR-3q63zv, amended 2026-10-05 by owner ruling): a revocation relayed by a
// rogue device R is acted on, because the chain decides whether the removal
// is real. This branch refused it for a while (a sender check the owner
// retired the same day); this is master's test, restored.
// verifies: LLR-3q63zv, LLR-6p4pj2, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_relayed_by_a_non_member_is_still_acted_on() {
    let mut s = admit_b_directly(setup("revoke-relay").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");

    let (rogue_addr, rogue_task) = spawn_recv_one(ROGUE_SEED).await;
    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;

    tokio::time::timeout(
        NET,
        s.svc_a.revoke_member(&mut OsRng, s.org_id, b_member_id, Some(rogue_addr)),
    )
    .await
    .expect("revoke_member timed out")
    .expect("revoke_member failed");

    let (rogue_ep, sender, captured) = rogue_task.await.unwrap();
    assert_eq!(
        sender.as_bytes(),
        s.svc_a.endpoint().expect("A endpoint").device_key().as_bytes(),
        "R must have received the revocation from A itself"
    );
    rogue_ep.send(b_addr, &captured).await.expect("rogue relay to B failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B must act on a chain-valid revocation whoever relayed it");
    assert!(
        matches!(outcome, SelfDeleteOutcome::SelfDeleted { .. }),
        "expected SelfDeleted, got {outcome:?}"
    );
    assert!(svc_b.list_orgs().is_empty(), "B's record of the org must be gone");
    assert_eq!(svc_b.list_personas()[0].status, PersonaStatus::Revoked);
}
```

Step 2 — red, the self-delete path (update branch). Replace
`pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path`
(`:2390-2430`) with master's test, comment rewritten:

```rust
// PR-u4c2vp, resolved by owner ruling 2026-10-05: nothing about the sender is
// checked on either Receive operation (REQ-ztdza4, LLR-3q63zv). A rogue relay
// R forwards the administrator's genuine admission of C to B's self-delete
// path, and B commits it as an ordinary update because it matches the chain.
// verifies: LLR-3q63zv, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path() {
    let mut s = admit_b_directly(setup("pr-u4c2vp").await).await;
    let epoch_before = s.svc_b.list_orgs()[0].epoch;

    let jr_c = join_request_for_c(&mut s.svc_a);
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, r_addr, org_secret()))
        .await
        .unwrap()
        .unwrap();
    let (r_ep, _sender, msg) = r_task.await.unwrap();

    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    r_ep.send(b_addr, &msg).await.expect("relay to B");
    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("a chain-valid update is committed whoever relays it"),
            SelfDeleteOutcome::UpdatedNotRevoked { .. }
        ),
        "committed as an ordinary update"
    );
    assert!(svc_b.list_orgs()[0].epoch > epoch_before, "and B's record moved");
}
```

Step 3 — red, an undecodable Change set from a device outside the record.
Replace `a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_decoding`
(`:2737-2758`): keep its body to the `rogue.send` line, rename it
`a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed`,
and replace its comment, annotation and final assertions with:

```rust
// REQ-ag6kqm as amended: nothing about the sender is checked, so Change set
// bytes that do not decode are refused as what they are, whoever sends them,
// and B's record does not move.
// verifies: REQ-ag6kqm, LLR-mcdh85, LLR-9sknpa
// … body unchanged up to and including `rogue.send(b_addr, &msg).await.expect("rogue send");`
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.last_seq, after.root_hash), (before.epoch, before.last_seq, before.root_hash));
}
```

Step 4 — red, the forged revocation in `org-node/tests/service_stories.rs`.
In `revocation_from_an_unknown_device_leaves_the_record_in_place` (`:441-…`),
replace the comment above the annotation, the annotation, and the
`matches!(err, OrgNodeError::UnknownSender)` assertion (`:626-631`):

```rust
// Abnormal-input case of the self-delete rule: a revocation Change set that
// removes this node's own DevicePublicKey, well formed and naming the right
// Organisation, but never published on chain, delivered by a device in no
// member's slots of the node's record. Nothing about the sender is checked
// (LLR-3q63zv, amended 2026-10-05); the chain refuses it, and the refusal
// leaves the OrgRecord exactly as it was. The node must never delete its
// record of the Organisation on a message it refused.
// verifies: REQ-uxv2x2, LLR-6qmq2g, LLR-vw2jn6, LLR-3q63zv
// …
    let err = r.expect_err("a revocation the chain never published must be rejected");
    assert!(
        matches!(err, OrgNodeError::StaleEpoch { .. }),
        "expected StaleEpoch (the chain holds no newer state), got {err:?}"
    );
```

Step 5 — run, expect red: `ONODE(admission_sender)` and
`ONODE(service_stories)` with `-- --nocapture 2>&1 | tee $S/trim-t1-red.log`.
Expected: the four tests above fail (`UnknownSender` returned where `Ok` or
`MalformedDelta` or `StaleEpoch` is asserted); every other test passes.

Step 6 — verify-level tests, compile-red. In
`org-node/tests/verify_against_chain.rs`:

- Replace the module comment's last paragraph (`:11-15`) with: "2026-10-05:
  the Envelope carries no signature and nothing about its sender is checked
  (REQ-ag6kqm, amended in place by owner ruling). After the org binding, the
  replay check is the only check before the decode."
- Delete `admin_sender` and `imposter` (`:39-50`) and the `DevicePublicKey`
  and `DeviceSeed` imports if nothing else uses them.
- Replace `ctx` (`:51-66`) with:

```rust
/// The receiver's context: expects `org`, has committed `parent_seq` 1 at
/// epoch 1. It holds no key: verification decides from the Envelope, this
/// context and the chain reader alone (LLR-na7p4w).
fn ctx(org: OrgId) -> VerifyContext {
    VerifyContext {
        expected_org_id: org,
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    }
}
```

- Every call `ctx(x, &sender, &accepted)` becomes `ctx(x)`, and the
  `let (sender, accepted) = admin_sender();` lines go.
- Delete `rejects_a_sender_outside_the_accepted_devices` and
  `an_empty_accepted_set_accepts_no_sender` (`:113-145`).
- Replace `rejects_unknown_sender_before_decoding_delta` (`:302-320`) with:

```rust
// Garbage Change set bytes for the expected org at a fresh Sequence number:
// no signature and no sender is checked before the decode (REQ-ag6kqm as
// amended), so the refusal is the decode's own.
// verifies: REQ-ag6kqm, LLR-mcdh85, LLR-9sknpa
#[test]
fn undecodable_change_set_bytes_are_refused_as_malformed_delta() {
    let (org, local, _env, _) = setup();
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage(org, 2), &ctx(org), &MockChain::new())
            .unwrap_err(),
        OrgNodeError::MalformedDelta
    );
}
```

Run `ONODE(verify_against_chain)`. Expected: compile error, `struct
VerifyContext has no field named …` absent and `missing fields sender and
accepted_senders` present. That is the red.

Step 7 — green. In `org-node/src/verify.rs`:

- Module doc (`:1-4`): "verify-against-chain: the single security property of
  the PoC. A received envelope is committed only if applying its delta
  reproduces a root that independently matches the on-chain root at a newer
  epoch. Nothing about who delivered it is checked (REQ-ag6kqm). See spec
  §5.2."
- `VerifyContext` loses `sender` and `accepted_senders` and its lifetime:
  `pub struct VerifyContext { pub expected_org_id: OrgId, pub seq_guard: SeqGuard, pub last_committed_epoch: Epoch }`,
  with the existing doc comments on the three fields.
- `verify_envelope_against_chain(…, ctx: &VerifyContext, …)`.
- Delete step 2 (`:63-66`) and renumber the step comments 2–7. The function
  doc: "Order is security-critical: cheap checks on what the envelope claims
  first, chain read and root match last."
- Drop `use org_members::DevicePublicKey;`.

In `org-node/src/service.rs`:

- Delete `fn record_devices` (`:518` and its doc comment).
- `receive_and_verify`: the existing-record arm stops computing `senders`;
  the tuple loses `accepted_senders`; the first-admission arm's last tuple
  element `vec![admin_device]` goes (the arm's own checks stay until T2).
  `VerifyContext { expected_org_id: org_id, seq_guard: …, last_committed_epoch: last_epoch }`.
- `receive_and_self_delete_if_revoked`: delete the REQ-7h7qp3 comment and
  `let accepted_senders = …` (`:1203-1206`); the context loses `sender` and
  `accepted_senders`; `let (remote_device_key, msg)` becomes
  `let (_sender, msg)` with the comment "// The authenticated sender is not
  checked: the chain decides (REQ-ztdza4, LLR-3q63zv)."

In `org-node/src/envelope.rs` the module doc (`:1-4`) becomes: "Envelope: the
wire form for a trie change, bound to an Organisation and a Sequence number.
It carries no signature (REQ-ag6kqm): the chain's Membership root at a newer
epoch decides whether the change is committed."

In `chain_genesis_e2e.rs:308-316` and `:473-481`,
`transport_handshake.rs:120-130`, `transport_networked.rs:148-158` and
`fuzz_verify_against_chain/fuzz_target.rs:122-132`: build `VerifyContext`
with the three remaining fields and delete the `sender` / `accepted` locals
that only fed it.

Step 8 — green. Run `ONODE(verify_against_chain)`, `ONODE(admission_sender)`,
`ONODE(service_stories)`, `ONODE(fuzz_verify_against_chain)`,
`ONODE(transport_handshake)`, and
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test transport_networked --no-run`.
Expected: all pass (`test result: ok`), the `--no-run` line exits 0. The
tests that still expect `UnknownSender` from the receive path's own checks
(`:273`, `:381`, `:685`, `:717`, `:759`, `:1977`) still pass: those checks are
in `service.rs` until T2 and T3.

Step 9 — commit: `git add` the ten files, then
`git -c commit.gpgsign=false commit -m "trim(org-node): verification and the self-delete path check nothing about the sender"`.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`).
red -> green: a_revocation_relayed_by_a_non_member_is_still_acted_on — watched fail (`UnknownSender` from the self-delete path) before the sender check was removed
red -> green: pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path — watched fail (`UnknownSender`) before the code change
red -> green: a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed — watched fail (`UnknownSender` where `MalformedDelta` asserted) before the code change
red -> green: revocation_from_an_unknown_device_leaves_the_record_in_place — watched fail (`UnknownSender` where `StaleEpoch` asserted) before the code change
red -> green: undecodable_change_set_bytes_are_refused_as_malformed_delta (and every `ctx(org)` call in verify_against_chain.rs) — watched fail to compile (`missing fields accepted_senders and sender`) before `VerifyContext` lost them
Also changed, not in the plan: `a_rejected_verification_leaves_the_callers_trie_untouched` rejected with an imposter sender; it now rejects on a chain root the Change set does not reach.

### T2 — A first admission commits whoever sends it, with or without an Invite

**Files touched:** `org-node/src/service.rs`, `org-node/tests/admission_sender.rs`
**Parallel:** no (serial, after T1)
**Trace:** REQ-xa6smf, LLR-j83kc8, LLR-mbjfq8, LLR-xq9nrq, LLR-e5c9ud

Step 1 — red. In `org-node/tests/admission_sender.rs`:

(a) `first_admission_from_a_device_other_than_the_invites_admin_is_rejected`
(`:269-322`): keep the body through the relay send; rename it
`first_admission_from_a_device_other_than_the_invites_admin_is_committed`;
replace the comment, annotation and everything after
`let (svc_b, result) = b_task.await.unwrap();` with:

```rust
// REQ-xa6smf as amended 2026-10-05: a first admission that verifies against
// the chain is committed whichever device delivers it. A rogue relay R
// forwards A's genuine admission of B; B commits it and consumes its Invite.
// verifies: REQ-xa6smf, LLR-j83kc8, LLR-9fvb3y
// … body unchanged through the relay send …
    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("a chain-valid first admission is committed whoever relays it");
    assert_eq!(outcome.org_id, s.org_id);
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs().len(), 1, "B committed the OrgRecord");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
    assert!(svc_b.list_pending_invites().is_empty(), "the Invite is consumed by the commit");
```

(b) Replace `a_first_admission_with_no_imported_invite_is_refused`
(`:2432-2470`) with master's test, renamed back and annotated:

```rust
// A first admission with NO imported Invite is committed on the chain anchor
// alone (REQ-xa6smf, LLR-mbjfq8, amended 2026-10-05): no Invite is required.
// verifies: REQ-xa6smf, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_with_no_imported_invite_rests_on_the_chain_alone() {
    let mut s = setup_with("no-invite-chain-alone", false).await;
    assert!(s.svc_b.list_pending_invites().is_empty(), "B imports no invite");

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    assert_eq!(outcome.expect("committed with no Invite").org_id, s.org_id);
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
}
```

(c) `a_first_admission_whose_invite_names_another_organisation_key_is_refused`
(`:2840-2867`): keep the body through the send; rename it
`a_first_admission_whose_invite_names_another_organisation_key_is_committed`;
replace comment, annotation and the four final assertions with:

```rust
// The Invite is not compared with the chain: REQ-xa6smf requires no Invite,
// so an Invite whose Organisation public key differs from the chain's does not
// stop a chain-valid first admission (owner ruling 2026-10-05).
// verifies: REQ-xa6smf, LLR-j83kc8
// … body unchanged through the send …
    assert_eq!(result.expect("committed: the Invite is not compared with the chain").org_id, s.org_id);
    assert_eq!(svc_b.list_orgs().len(), 1, "B committed the OrgRecord");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
```

(d) The administrator key (default answer to Q1). In `admit_b_directly`
(`:250-255`) replace the assertion with:

```rust
    assert_eq!(
        svc_b.list_orgs()[0].admin_member_key.as_bytes(),
        s.chain.get(&s.org_id).unwrap().org_pub_key.as_bytes(),
        "a member's record holds the chain's Organisation public key in admin_member_key (LLR-xq9nrq)"
    );
```

Replace the comment block, annotation, name and first block of
`a_first_admission_records_the_invites_administrator_the_chains_key_the_secret_and_the_member`
(`:2022-2058`):

```rust
// What a first admission writes: the administrator-key field, the
// Organisation public key and the Organisation secret the message carried,
// and the admitted Persona's member id and status (LLR-xq9nrq, LLR-ckk5nz,
// LLR-e5c9ud). LLR-xq9nrq, amended 2026-10-05: the record's admin_member_key
// is the Organisation public key read from the chain in the same operation,
// never a value from the Wire message or the Invite; the chain no longer
// names an administrator. A member's record holds no Organisation private key
// (LLR-3fwykc).
// verifies: LLR-ckk5nz, LLR-e5c9ud, LLR-xq9nrq, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_chains_key_the_secret_and_the_member() {
    let s = admit_b_directly(setup("first-admission-fields").await).await;
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let rec = s.svc_b.list_orgs()[0].clone();
    assert_eq!(rec.admin_member_key.as_bytes(), published.as_bytes(), "the chain's key, not the Invite's");
    assert_ne!(rec.admin_member_key, s.svc_a.list_orgs()[0].admin_member_key, "not the Invite's administrator key");
    assert_eq!(rec.org_pub_key, published);
```

and in its last block replace `admin_member_key` by
`rec.admin_member_key` in the reload assertion. The rest is unchanged.

(e) Delete `first_admission_without_an_imported_invite_is_rejected`
(`:677-710`), `first_admission_from_a_wrong_sender_is_rejected_before_its_snapshot_is_decoded`
(`:712-748`) and `an_invite_org_key_mismatch_is_refused_before_the_snapshot_is_decoded`
(`:2914-2940`), each with its comment block.

Run `ONODE(admission_sender) 2>&1 | tee $S/trim-t2-red.log`. Expected red:
(a) `UnknownSender`, (b) `NoImportedInvite`, (c) `InviteOrgKeyMismatch`, (d)
the administrator-key assertions fail in `admit_b_directly` and therefore in
every test that calls it. Everything else in the file compiles.

Step 2 — green. In `org-node/src/service.rs`, `receive_and_verify`, replace
the comment above `let existing` and the whole `match existing { … }`
(`:917-955` after T1) with:

```rust
        // The record this message extends. Nothing about the sender is
        // checked and no Invite is required (REQ-xa6smf, REQ-ztdza4, owner
        // ruling 2026-10-05): the chain decides.
        let existing = self.store.data().orgs.iter().find(|o| o.org_id == org_id).cloned();
        let (local_trie, last_seq, last_epoch, admin_member_key, is_first_admission) =
            match existing {
                Some(existing) => {
                    let trie = trie_from_snapshots(&existing.trie_members)?;
                    (trie, existing.last_seq, existing.epoch, existing.admin_member_key, false)
                }
                None => {
                    // The record a first admission extends (REQ-d9g6nt).
                    let trie = first_admission_base(msg.genesis_snapshot.as_deref())?;
                    // LLR-xq9nrq: the administrator-key field is the
                    // Organisation public key read from the chain in this
                    // operation, never a value from the Wire message.
                    let admin_member_key = PersonPublicKey::parse(chain_state.org_pub_key.as_bytes())?;
                    (trie, SequenceNumber::new(0), Epoch::new(0), admin_member_key, true)
                }
            };
```

and the persona-selection comment (`:974-977`): "This node's persona: one
whose DevicePublicKey is in the new record and whose Member-as-a-group key is
not the record's `admin_member_key` (LLR-e5c9ud)." The `retain` that consumes
the pending Invite stays.

Step 3 — green: `ONODE(admission_sender)`, `ONODE(service_stories)`,
`ONODE(fuzz_first_admission_base)`. Expected: all pass, except that T3's
three flips have not been made yet and the old `UnknownSender` tests at
`:381`, `:759`, `:1977` still pass because the post-verification check is
still there.

Step 4 — commit: `git -c commit.gpgsign=false commit -m "trim(org-node): a first admission commits whoever sends it, with or without an Invite"` after `git add` of the two files.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`). Q1 as the owner answered it: `admin_member_key` is the imported Invite's, else the chain's Organisation public key.
red -> green: first_admission_from_a_device_other_than_the_invites_admin_is_committed — watched fail (`UnknownSender`) before the first-admission arm lost its checks
red -> green: a_first_admission_with_no_imported_invite_rests_on_the_chain_alone — watched fail (`NoImportedInvite`) before the code change
red -> green: a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key (new, Q1, verifies LLR-xq9nrq) — watched fail (`NoImportedInvite`) before the code change
red -> green: a_first_admission_whose_invite_names_another_organisation_key_is_committed — watched fail (`InviteOrgKeyMismatch`) before the code change
Not changed (Q1): `admit_b_directly`'s Invite assertion and `a_first_admission_records_the_invites_administrator_…` keep their name and Invite assertion; its annotation is LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc.
Also changed, not in the plan: `pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key` went red once the check was gone (the joiner now commits); it now pins the defect by the key a relay sees the push sent under, and asserts the joiner commits.

### T3 — An update commits whoever sends it

**Files touched:** `org-node/src/service.rs`, `org-node/tests/admission_sender.rs`
**Parallel:** no (serial, after T2)
**Trace:** REQ-ztdza4, REQ-ag6kqm, LLR-u6rq4s, LLR-37cj3n, LLR-9fvb3y

Step 1 — red. (a) `update_relayed_by_a_non_member_after_admission_is_rejected`
(`:371-423`): rename `update_relayed_by_a_non_member_after_admission_is_committed`;
keep the body through the relay send; replace comment, annotation and the
assertions after `let (svc_b, result) = b_task.await.unwrap();` with:

```rust
// REQ-ztdza4 and REQ-ag6kqm as amended 2026-10-05: an update that verifies
// against the chain is committed whichever device relays it. A rogue device R
// relays A's genuine admission of C; B commits epoch 3.
// verifies: REQ-ag6kqm, REQ-ztdza4, LLR-u6rq4s, LLR-37cj3n, LLR-9fvb3y
// … body unchanged through the relay send …
    let outcome = result.expect("a chain-valid update is committed whoever relays it");
    assert_eq!(outcome.epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].last_seq, SequenceNumber::new(3), "the mark is the epoch committed");
    assert_ne!(svc_b.list_orgs()[0].root_hash, root_at_2, "B's root moved to the chain's");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "admin + B + C");
```

(the `last_seq_at_2` local goes).

(b) `a_removal_relayed_by_the_member_it_removes_is_refused` (`:1966-2020`):
rename `a_removal_relayed_by_the_member_it_removes_is_committed`; replace its
comment, annotation and final assertions with:

```rust
// LLR-u6rq4s as amended 2026-10-05: the delivering device is compared with no
// Membership record. C relays its own removal to B on the ordinary receive
// path; C is in B's record before the change and not after it, and B commits
// the change because it matches the chain.
// verifies: REQ-ztdza4, LLR-u6rq4s
// … body unchanged through `let (svc_b, result) = b_task.await.unwrap();` …
    let committed = result.expect("a chain-valid removal is committed whoever relays it");
    assert!(committed.epoch > before.epoch);
    let after = svc_b.list_orgs()[0].clone();
    assert!(!after.trie_members.iter().any(|m| m.id == c_id), "C is gone from B's record");
```

(c) Delete `update_relayed_by_the_device_it_removes_is_rejected` (`:750-812`)
with its comment block.

Run `ONODE(admission_sender)`: (a) and (b) fail with `UnknownSender`.

Step 2 — green. In `service.rs` delete the REQ-ztdza4 block after
verification (`:968-972` at `ef6e261`), replace the doc comment of
`receive_and_verify` (`:878-884`) with "Accept one inbound `WireMessage`,
verify its envelope against the chain, and commit the new state. Nothing
about the sender is checked (REQ-xa6smf, REQ-ztdza4).", and change
`let (remote_device_key, msg)` to `let (_sender, msg)`.

Step 3 — green: `ONODE(admission_sender)`, `ONODE(service_stories)`. All pass.

Step 4 — commit the two files: `trim(org-node): an update commits whoever sends it`.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`).
red -> green: update_relayed_by_a_non_member_after_admission_is_committed — watched fail (`UnknownSender`) before the post-verification check was removed
red -> green: a_removal_relayed_by_the_member_it_removes_is_committed — watched fail (`UnknownSender`) before the code change

### T4 — The refusals only the removed checks raised are gone; an Invite import fails as master's did

**Files touched:** `org-node/src/error.rs`, `org-node/src/service.rs`, `org-node/tests/persona_records.rs`, `org-node/tests/admission_sender.rs`, `org-node/tests/value_types.rs`
**Parallel:** no (serial, after T3)
**Trace:** LLR-8bum44, LLR-z8fubr

Step 1 — red. In `org-node/tests/persona_records.rs:506-525` replace the doc
comment, annotation and the `assert_eq!(err, OrgNodeError::InvalidInvite, …)`
line:

```rust
/// An Invite whose Organisation public key, administrator's Member-as-a-group
/// key or administrator's DevicePublicKey is not a valid key of its kind fails
/// to decode, as master reports it (`Chain("blob decode…")`), and nothing is
/// stored (LLR-8bum44 as amended 2026-10-05).
/// verifies: LLR-8bum44
// …
        assert!(matches!(&err, OrgNodeError::Chain(m) if m.starts_with("blob decode")), "{what}: got {err:?}");
```

In `org-node/tests/value_types.rs:57-84` the list becomes master's order with
this branch's two kept variants:

```rust
    let all = [
        OrgNodeError::OrgIdMismatch,
        // `InvalidOrgPublicKey` is the chain-state refusal REQ-8jb4ny added.
        OrgNodeError::InvalidOrgPublicKey,
        OrgNodeError::StaleSeq { got: 1, last_seen: 2 },
        OrgNodeError::MalformedDelta,
        OrgNodeError::DeltaBaseMismatch,
        OrgNodeError::OrgNotOnChain,
        OrgNodeError::RootMismatch,
        OrgNodeError::StaleEpoch { got: 1, last: 2 },
        OrgNodeError::Chain("read failed".into()),
        OrgNodeError::Trie(org_members::OrgMembersError::DuplicateHandle),
        // The Sequence number that is not the chain's epoch (REQ-txvtm9).
        OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 },
    ];
```

Delete `an_invite_naming_an_invalid_administrator_member_key_is_refused_at_import`
and `an_invite_naming_an_invalid_administrator_device_key_is_refused_at_import`
from `admission_sender.rs` with their comment blocks, and `tampered` and
`non_canonical_device_key` if nothing else uses them.

Run `ONODE(persona_records)`: the Invite test fails (`InvalidInvite` returned).

Step 2 — green. `service.rs` `import_invite` (`:706-717`): doc comment "Decode
and persist an incoming `Invite` blob. Its keys are parsed by their own
types, so an Invite holding an invalid key is refused whole and nothing is
stored (LLR-8bum44)." and
`let inv: crate::blobs::Invite = crate::blobs::decode(blob)?;`.
`error.rs`: delete `UnknownSender`, `InvalidInvite`, `InviteOrgKeyMismatch`,
`NoImportedInvite` with their doc comments.

Step 3 — green: ORG-NODE-GATE, then
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test finality_polling --test preflight --no-run`.
Both pass. Record the pass counts per target in `$S/trim-t4-gate.log`.

Step 4 — commit the five files: `trim(org-node): drop the refusals of the removed sender and Invite checks`.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`).
red -> green: an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored — watched fail (`InvalidInvite` where `Chain("blob decode…")` asserted) before `import_invite` dropped its `map_err`
ORG-NODE-GATE exit 0 (183 tests, 3 fuzz targets); chain_genesis_e2e, finality_polling, preflight `--no-run` exit 0.

### T5 — The app classifies only the refusals that remain

**Files touched:** `app/src-tauri/src/events.rs`, `app/src-tauri/tests/receiver_events.rs`, `app/docs/requirements/2026-09-14-tauri-shell.md`, `app/docs/risk/2026-09-14-app-hazards.md`, `app/.guardrails/config.yaml`
**Parallel:** yes, with T6–T10 (starts after T4)
**Trace:** REQ-kn5rtx, REQ-jfxah3

Step 1 — red (compile). APP-GATE fails: `no variant named UnknownSender`
(and the three others) in `events.rs` and `receiver_events.rs`.

Step 2 — `receiver_events.rs`:

- `verification_verdicts()` (`:382-414`): doc comment back to master's ("each
  is produced only by `verify_envelope_against_chain`", which is true again);
  list: `OrgIdMismatch`, `StaleSeq { got: 3, last_seen: 7 }`, `MalformedDelta`,
  `DeltaBaseMismatch`, `RootMismatch`, `StaleEpoch { got: 2, last: 9 }`,
  `SeqNotEpoch { seq: 9, epoch: 3 }` with the comment "Added 2026-10-05: a
  Sequence number that is not the chain's epoch (an org-node commit rule)
  refuses the received update itself."
- `verify_failure_carries_the_message` (`:311-320`): message
  `"recomputed root does not match the on-chain root"`.
- Delete `an_invalid_invite_is_classified_as_a_receiver_error` (`:499-507`)
  and `a_first_admission_refused_on_a_local_fact_is_a_receiver_error`
  (`:768-784`) with their comments.
- `a_non_terminal_failure_does_not_stop_the_loop` (`:658-660`): verdict
  `OrgNodeError::RootMismatch`, comment "a root mismatch is a reason to reject
  that envelope, not a reason to stop receiving."
- `a_refusal_this_change_added_does_not_stop_the_loop` (`:786-817`):
  `receiver_errors` is `[OrgNodeError::InvalidOrgPublicKey]`; its comment drops
  "the two first-admission refusals".

`events.rs` `classify_receive_error`: verdict arm `OrgIdMismatch | StaleSeq |
MalformedDelta | DeltaBaseMismatch | RootMismatch | StaleEpoch | SeqNotEpoch`
with the comment "Verdicts on an incoming update: each one means 'this did
not verify', and each is produced only by `verify_envelope_against_chain`.";
receiver-error arm `Chain | OrgNotOnChain | Trie | InvalidOrgPublicKey |
InvalidField` with its current comment minus the Invite lines.

Step 3 — ledgers. `app/docs/requirements/2026-09-14-tauri-shell.md:229-277`:
replace the two amendment notes with one:

> *(Amended 2026-10-05.)* The list of verdicts above describes the code before
> org-node dropped the Envelope signature; `classify_receive_error`
> (`app/src-tauri/src/events.rs`) is the current list. The rule is unchanged:
> a verdict is a refusal of what the sender delivered, judged against the
> chain or the node's own record, and each verdict is still produced only by
> `verify_envelope_against_chain`. The seven verdicts are `OrgIdMismatch`,
> `StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch`, `RootMismatch`,
> `StaleEpoch` and `SeqNotEpoch` (a Sequence number other than the chain's
> epoch). `BadSignature` is gone with the signature, and nothing replaces it:
> org-node checks nothing about the sender (owner ruling 2026-10-05). The
> receiver errors are `Chain(_)`, `OrgNotOnChain`, `Trie(_)`,
> `InvalidOrgPublicKey` (the chain's Organisation state carries a key that is
> not a valid X25519 key, refused before anything is verified against it) and
> `InvalidField { .. }`. `StaleEpoch` stays a verdict for the reason given
> above.

Keep the paragraph on how `StaleEpoch` fits from the current text, moved under
this note. `app/docs/risk/2026-09-14-app-hazards.md`: the RC-3rddh7 note
(`:527-544`) becomes "*Amended 2026-10-05 (change
`worktree-person-shared-types`).* The set of verdicts this control classifies
changed when org-node dropped the Envelope signature: `BadSignature` is gone,
and `SeqNotEpoch` is a new verdict. `InvalidOrgPublicKey` is a new receiver
error. The rule and the list are in REQ-kn5rtx's rationale. The control's
wording, its requirements and the residual below are unchanged." The HAZ-9fmhm4
note (`:959-963`) is deleted (the misfiling it records was in variants this
trim removes). The REQ-kn5rtx assessment note (`:1184-1200`) becomes
"*Reassessed 2026-10-05 (change `worktree-person-shared-types`).* That compile
error happened: org-node's unsigned Envelope added `SeqNotEpoch` and
`InvalidOrgPublicKey` and removed `BadSignature`. Each is classified
explicitly. Neither new variant concerns the transport endpoint, so neither
is terminal (REQ-jfxah3), and `a_refusal_this_change_added_does_not_stop_the_loop`
pins that. No new hazard."

`app/.guardrails/config.yaml`: replace this branch's added comment
(`:83-89`) with the counts T13 measures (receiver_events 32 → 30).

Step 4 — green: APP-GATE, `npm --prefix app run check`, `npm --prefix app run test`;
`GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`.

Step 5 — commit the five files: `trim(app): classify only the refusals org-node still raises`.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`).
red -> green: APP-GATE — watched fail to compile (`no variant named UnknownSender`, `InvalidInvite`, `InviteOrgKeyMismatch`, `NoImportedInvite` in events.rs) before `classify_receive_error` dropped them
APP-GATE exit 0 (receiver_events 30, 84 cargo tests); `npm run check` 0 errors; `npm run test` 30 passed.
The REQ-kn5rtx reassessment note keeps the sentence on a persistent invalid Organisation public key (still true).

### T6 — Tests carry only IDs whose text they verify

**Files touched:** `org-node/tests/admission_sender.rs`, `org-node/tests/blob_exchange.rs`, `org-node/tests/calldata_typed.rs`, `org-node/tests/chain_write_pure.rs`, `org-node/tests/encoding_golden.rs`, `org-node/tests/envelope_binding.rs`, `org-node/tests/key_custody.rs`, `org-node/tests/persona_records.rs`, `org-node/tests/service_lifecycle.rs`, `org-node/tests/service_stories.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/verify_against_chain.rs`
**Parallel:** yes, with T5 and T7–T10 (starts after T4)
**Trace:** every ID below

Replace each `verifies:` line with the one given. Lines T1–T4 rewrote are
already right and are listed only for completeness. Comments above a test
that name a deleted ID are reworded in the same edit (T11's sweep checks).

| File:line (`ef6e261`) | Test | New `verifies:` |
|---|---|---|
| admission_sender.rs:330 | update_from_the_admin_after_admission_is_committed | REQ-ztdza4, LLR-u6rq4s, LLR-cja9zv, LLR-9f5hmr |
| admission_sender.rs:536 | a_committed_admission_reaches_the_disk_and_consumes_the_invite | REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, LLR-cja9zv, LLR-q8emds |
| admission_sender.rs:583 | an_admission_reaches_the_administrators_disk | LLR-t4znbk, LLR-rb8r65, LLR-ghja3x |
| admission_sender.rs:987 | a_revocation_reaches_the_administrators_disk | LLR-6dc598, LLR-tax3pm, LLR-qg9utu, LLR-pw369n, LLR-8hdu9x |
| admission_sender.rs:1189 | an_imported_invite_reaches_the_joiners_disk | LLR-9zfnmb |
| admission_sender.rs:1231 | the_admission_envelope_carries_the_epoch_its_update_produced | REQ-txvtm9, LLR-ghja3x |
| admission_sender.rs:1294 | in_loopback_mode_the_joiner_is_dialled_at_the_full_address | LLR-jn5jeh |
| admission_sender.rs:1331 | a_failed_revocation_push_leaves_the_administrators_record_where_it_was | LLR-6dc598, LLR-qg9utu |
| admission_sender.rs:1472 | a_second_organisation_is_admitted_into_without_touching_the_first | LLR-vdyu65, LLR-w3fhhg |
| admission_sender.rs:1768 | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names | LLR-cja9zv, LLR-q8emds, LLR-y2v8v2, LLR-9zfnmb, LLR-e5c9ud |
| admission_sender.rs:2087 | pr_xwek5e_another_members_revocation_clears_the_receivers_secret | LLR-ckk5nz, LLR-8hdu9x |
| admission_sender.rs:2764 | the_administrators_own_persona_is_never_the_one_a_receive_marks | LLR-e5c9ud |
| blob_exchange.rs:51 | an_invite_round_trips_carrying_every_field | LLR-g9vmbx |
| blob_exchange.rs:66 | a_join_request_round_trips_carrying_every_field | LLR-kkj64b |
| calldata_typed.rs:58, :66 | both | LLR-ayrdr8 |
| chain_write_pure.rs:28 | update_calldata_is_exactly_one_hundred_bytes_in_the_declared_order | LLR-rv4vux |
| encoding_golden.rs:60, :102, :124, :147, :156 | all five | LLR-ayrdr8 |
| envelope_binding.rs:40, :54 | build_binds_…, build_transmits_… | LLR-p8uu47 |
| envelope_binding.rs:76 | the_wire_form_has_no_signature_field | LLR-e7s4ye, LLR-pzde8b |
| key_custody.rs:42, :105 | a_device_keypair_rebuilt_…, x25519_seed_round_trip_preserves_key | LLR-e58j8m |
| key_custody.rs:52, :72, :94 | device_key_wraps_…, x25519_public_key_matches_rfc_7748, member_key_is_the_x25519_… | LLR-ctzkv7 |
| key_custody.rs:62 | two_different_keypairs_do_not_share_a_device_key | none: the annotation goes, with the comment "No requirement states this since REQ-ztdza4's amendment; kept as a regression check." |
| persona_records.rs:200, 320, 338, 372, 388, 403, 417, 443, 454, 477, 490 | eleven tests | LLR-8bum44 |
| service_lifecycle.rs:49 | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key | REQ-hzm4kt, LLR-tev8h8, LLR-s7yu4k, LLR-v82xds |
| service_lifecycle.rs:185 | the_endpoint_is_bound_once_… | LLR-6zjzn2 |
| service_lifecycle.rs:219 | the_endpoint_binds_the_named_personas_device_… | LLR-cns6q6 |
| service_stories.rs:56 | five_stories_full_e2e | REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, REQ-uxv2x2, LLR-rb8r65, LLR-bg3vsw, LLR-t4znbk, LLR-q8emds, LLR-68yd3j, LLR-6zjzn2, LLR-cns6q6, LLR-6p4pj2 |
| transport_handshake.rs:302 | endpoint_id_equals_device_key | LLR-ygn78w |
| verify_against_chain.rs:88 | happy_path_commits_when_root_matches_chain | REQ-nhe2zu, REQ-txvtm9, REQ-ag6kqm, LLR-8n95rf, LLR-d6kvbx, LLR-8hwqru, LLR-na7p4w, LLR-9f5hmr |
| verify_against_chain.rs:101, :289 | rejects_wrong_org_id, rejects_wrong_org_before_decoding_delta | REQ-gju89b, LLR-4fbuy8, LLR-ybn5pr |
| verify_against_chain.rs:147 | rejects_stale_seq | REQ-6yu72z, LLR-xpbkp5, LLR-wx3php, LLR-cs4mpb |
| verify_against_chain.rs:364 | rejects_a_delta_whose_base_root_is_not_the_local_root | REQ-wp2nyc, LLR-992mbf, LLR-9sknpa |
| verify_against_chain.rs:451 | a_sequence_number_other_than_the_chain_epoch_is_refused | REQ-txvtm9, REQ-mr5abb, LLR-9f5hmr, LLR-cs4mpb |

Files that keep their annotations unchanged (P4): `chain_read_state.rs`,
`node_value_types.rs`, `secret_redaction.rs`.

Step — `ONODE` on each edited target compiles and passes (annotations are
comments; this catches a slipped edit). Commit the twelve files:
`trim(org-node): tests carry only IDs whose text they verify`.

**Done 2026-10-05** (task branch `worktree-worktree-person-shared-types-trim-code`). Annotations only, no behaviour change: no red. Q4 applied (LLR-en9p5c, LLR-x35s6p, LLR-zyw5r2 → LLR-mmdu38, LLR-56hc77, LLR-bwb9pu); every folded or withdrawn ID retargeted in org-node/tests and in org-node/src comments (the src part of T11's sweep for these IDs). `key_custody.rs` two_different_keypairs_do_not_share_a_device_key carries no annotation. ORG-NODE-GATE exit 0; org-node and app check-trace and check-ids exit 0 after merging the ledger branch.

### T7 — org-node architecture ledger: folds, amendments, deletions

**Files touched:** `org-node/docs/architecture/2026-10-03-decomposition.md`, `org-node/docs/architecture/2026-10-04-type-safety.md`, `org-node/docs/architecture/2026-10-05-unsigned-envelope.md`, `org-node/docs/architecture/soup.md`
**Parallel:** yes, with T5, T6, T8–T10

In `2026-10-03-decomposition.md` and `2026-10-04-type-safety.md`, for each
item below: replace the definition paragraph with the text given, delete its
`superseded-by:` line, set the `satisfies:`/`traces:` line given, and replace
this branch's "Note 2026-10-05, at the merge of master `1feb608` …
superseded by …" paragraph (where there is one) with the dated note given.
Master's own older notes stay.

**SDD-sxp8hb** (dec:165):
> **SDD-sxp8hb**: the key pairs a node holds and how each is made from its
> secret: the ed25519 device keypair, whose verifying key is the node's
> DevicePublicKey in the trie and its transport identity, and the X25519 key
> pairs behind a Member-as-a-group key and behind the Organisation public
> key, each from its own secret (the member seed and the Organisation private
> key). The Organisation public key a node publishes is built from its key
> pair only through `OrgPublicKey`'s parse, `person`'s X25519 validity check.
> No key here signs or verifies anything.
> traces: REQ-ag6kqm, REQ-8jb4ny, REQ-ech45n, REQ-y7tsft
>
> Note: "…This item described one ed25519 keypair whose verifying key both
> signed changes to membership as the `P2pMemberKey` and served as the
> `P2pDeviceKey`. Nothing signs now, and a Member-as-a-group key is an X25519
> key from its own seed. LLR-3jjgtw, LLR-98ufry, LLR-sj7cd5, LLR-x35s6p and
> LLR-322xfu also constrain this item; they are in
> `2026-10-05-unsigned-envelope.md`."

**LLR-e58j8m** (dec:187), `satisfies: derived`:
> a keypair rebuilt from its persisted seed has the same public key as the
> keypair it came from: for a `SigningKeypair`, the same verifying key and
> the same DevicePublicKey; for an `X25519Keypair`, the same X25519 public
> key.
>
> Note: "…This item also required the same signature over the same message.
> Nothing in org-node signs. It is derived now, since no requirement asks for
> a signature; it is assessed in `org-node/docs/risk/2026-10-05-envelope-authenticity.md`."

**LLR-ctzkv7** (dec:200), `satisfies: REQ-ztdza4, REQ-xa6smf` (unchanged; Q5):
> a Persona's two public keys come from two seeds by two algorithms.
> `X25519Keypair::member_key` is the X25519 public key of the member seed
> (RFC 7748: the clamped seed times the base point) and is always a valid
> `PersonPublicKey`. `SigningKeypair::device_key` wraps the ed25519
> verifying key of the device seed.
>
> Note: "…This item said `member_key()` and `device_key()` wrap one verifying
> key. A Member-as-a-group key is now an X25519 key from its own seed."

**LLR-na7p4w** (dec:206), `satisfies: REQ-ag6kqm`:
> `VerifyContext` holds the expected Organisation, the Sequence-number guard
> and the last committed epoch, and no key: `verify_envelope_against_chain`
> decides from the Envelope, that context and the chain reader alone.
>
> Note: "…This item stated that `verify` accepts a signature the keypair
> produced. org-node verifies no signature (REQ-ag6kqm); `keys::verify` is
> gone."

**LLR-9fvb3y** (dec:211), `satisfies: REQ-ag6kqm`:
> a genuine Envelope relayed by a device other than the one that built it is
> committed on both Receive operations when it verifies against the chain:
> the delivering device is not an input to verification.
>
> Note: "…This item stated that `verify` refuses a signature by any other
> key. Nothing is signed, and the device that delivers an Envelope decides
> nothing (REQ-ag6kqm, REQ-xa6smf, REQ-ztdza4)."

**SDD-kk2y3e** (dec:220), `traces: REQ-ag6kqm, REQ-gju89b, REQ-9g6as6`:
> **SDD-kk2y3e**: the wire form of a change to membership. It binds the
> Organisation identifier, the Sequence number and the encoded Change set,
> and carries no signature. The item also owns the decode a receiver
> performs on it.
>
> Note: "…This item described a signed transcript and the signature check a
> receiver ran before anything else."

**LLR-e7s4ye** (dec:227), `satisfies: REQ-ag6kqm`:
> an Envelope's wire form is the postcard encoding of the Organisation
> identifier's twenty bytes, then the Sequence number, then the Change set
> bytes, in that order and nothing else; no transcript is signed.
>
> Note: "…This item described the signed transcript, which no longer exists.
> The three fields it bound are what the wire form carries."

**LLR-p8uu47** (dec:233), `satisfies: REQ-gju89b`:
> `Envelope::build` sets the Organisation identifier and the Sequence number
> it is given, and sets as the Change set bytes the postcard encoding of the
> delta. That encoding is canonical: decoding the transmitted bytes and
> encoding them again reproduces them exactly.
>
> Note: "…This item said `build` signs the transcript over the encoded bytes.
> Nothing is signed."

**LLR-ybn5pr** (dec:242), `satisfies: REQ-ag6kqm, REQ-gju89b`:
> an Envelope built for one Organisation is refused with `OrgIdMismatch` by a
> receiver expecting another, before its Change set is decoded.
>
> Note: "…This item said altering the Organisation identifier breaks the
> signature. With no signature, the receiver's expected Organisation is what
> refuses it."

**LLR-cs4mpb** (dec:247), `satisfies: REQ-ag6kqm`:
> an Envelope whose Sequence number is at or below the receiver's mark is
> refused with `StaleSeq` before its Change set is decoded, and one whose
> Sequence number is above the mark but not the epoch of the chain state it
> is verified against is refused with `SeqNotEpoch`.
>
> Note: "…This item said altering the Sequence number breaks the signature.
> The mark (REQ-6yu72z) and the chain's epoch (REQ-txvtm9) refuse it now."

**LLR-9sknpa** (dec:252), `satisfies: REQ-ag6kqm`:
> Change set bytes that do not decode are refused with `MalformedDelta`, and
> a Change set that decodes but does not extend the receiver's record is
> refused with `DeltaBaseMismatch`.
>
> Note: "…This item said altering the delta bytes breaks the signature. The
> decode and the base-root check refuse such bytes now, and the root match
> refuses any that survive both."

**LLR-pzde8b** (dec:257), `satisfies: REQ-ag6kqm`:
> an `Envelope` holds exactly the Organisation identifier, the Sequence
> number and the Change set bytes; it has no signature field and no
> `verify_signature`.
>
> Note: "…This item stated that `verify_signature` refuses any key but the
> signer's. It is gone with the signature."

**SDD-na9nc3** (dec:291), `traces: REQ-wp2nyc, REQ-bvh8v6, REQ-8gz8bu, REQ-gju89b, REQ-ag6kqm, REQ-6yu72z, REQ-mr5abb, REQ-nhe2zu, REQ-txvtm9, REQ-bcxz96`:
> **SDD-na9nc3**: the single decisive property of this unit — a received
> change is committed only if applying it to the local trie reproduces a
> root that independently matches the root the chain reports at an epoch
> newer than the last one committed, with the Sequence number equal to that
> epoch. The item owns both the checks and **the order they run in**, which
> is itself security-critical: the cheap checks on what the Envelope names
> and its Sequence number come first, so stale or misaddressed bytes are
> never decoded, and the watermark moves only after the decisive check has
> passed. Nothing about who delivered the Envelope is checked.
>
> Note: "…This item put an authenticity check first, on an unauthenticated
> sender. Nothing about the sender is checked (REQ-ag6kqm). LLR-9f5hmr, in
> `2026-10-05-unsigned-envelope.md`, also refines this item."

**LLR-mcdh85** (dec:305), `satisfies: REQ-ag6kqm`:
> `verify_envelope_against_chain` checks no signature and no sender: after
> the Organisation binding, the only check before the Change set bytes are
> decoded is the Sequence-number mark, and bytes that do not decode are
> refused with `MalformedDelta`.
>
> Note: "…This item required the signature to be checked before the delta
> bytes are decoded, refusing with `BadSignature`."

**LLR-g9vmbx** (dec:556), `satisfies: REQ-xa6smf` (unchanged; Q5):
> an Invite encoded and decoded yields an Invite equal to the original. It
> carries the Organisation identifier, its Organisation public key, and the
> administrator's Member-as-a-group key, DevicePublicKey and dialling
> address.
>
> Note: "…This item said the Invite carries the Published signing key; the
> chain's key is the Organisation public key."

**LLR-rv4vux** (dec:585), `satisfies: derived`:
> `build_update_calldata` emits exactly one hundred bytes: the four-byte
> selector, then the new Membership root, then the Organisation public key,
> then the expected epoch, in that order.
>
> Note: "…The third field was the Published signing key; it is the
> Organisation public key."

**LLR-s7yu4k** (dec:705), `satisfies: derived`:
> a Persona's identifier is the first sixteen bytes of its Member-as-a-group
> key, the X25519 public key of its member seed, written as thirty-two
> lowercase hexadecimal digits. Two Personas share an identifier only if
> their member keys share those bytes.
>
> Note: "…This item derived the identifier from the member verifying key."

**SDD-rx2yvy** (dec:763), `traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-d9g6nt, REQ-qn2erx`:
> **SDD-rx2yvy**: the administrator's side of letting someone in — minting
> the new member's leaf, moving the on-chain root forward, and handing the
> new member the change, unsigned, with everything they need to check it.
>
> Note: "…This item handed the new member the signed change."

**LLR-rb8r65** (dec:782): text of the deleted LLR-2ad4du (ue:194-197),
`satisfies: derived`; note "…This item said the signed change."
**LLR-ghja3x** (dec:789): text of LLR-53phvg (ue:201-204) with "(LLR-by65qx)"
replaced by "(LLR-pzde8b)", `satisfies: derived`; note "…This item said the
Envelope is signed by the administrator's Member-as-a-group key and carries
the record's last sequence number plus one. Nothing signs, and the Sequence
number is the epoch the update produced (REQ-txvtm9)."
**LLR-vdyu65** (dec:837): text of LLR-28qhcd (ue:208-211), `satisfies: derived`;
note "…This item also said `admit_member` signs with that Organisation's
administrator Persona. Nothing is signed; the Persona it looks up names the
endpoint to bind (LLR-cns6q6, PR-8qsnhx)."

**SDD-8cpyfa** (dec:914), `traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6, REQ-d9g6nt, REQ-qn2erx`:
> **SDD-8cpyfa**: the member's side — accept one Wire message, verify it
> against the chain, and commit. It takes the sender from the connection and
> checks nothing about it, on a first admission and on every later update.
>
> Note: "…This item owned the two sender cross-checks, the only place the
> transport's authenticated identity was compared with the membership record.
> Both are removed (REQ-xa6smf, REQ-ztdza4)."

**LLR-j83kc8** (dec:939), `satisfies: REQ-xa6smf`:
> on a first admission, the Wire message is committed when it verifies
> against the chain whichever Device key the connection authenticated,
> whether or not that key is the administrator's Device key an imported
> Invite names.
>
> Note: "…This item refused a first admission whose sender was not the
> Invite's administrator device."

**LLR-u6rq4s** (dec:945), `satisfies: REQ-ztdza4`:
> on a Wire message about an Organisation already held, the Device key the
> connection authenticated is compared with no Membership record: an update
> that verifies against the chain is committed whether that key is in the
> record before the update, after it, or in neither.
>
> Note: "…This item required that key to be in the trie the Envelope
> verified into, checked after verification."

**LLR-37cj3n** (dec:950), `satisfies: REQ-nhe2zu`:
> what `receive_and_verify` commits is decided by the Organisation state it
> reads from the chain itself and by its own record, never by a key or a
> value carried in the Wire message, and never by the device that delivered
> it.
>
> Note: "…This item named the author key the signature was verified under.
> There is no author key."

**LLR-mbjfq8** (dec:972), `satisfies: derived`:
> a first admission to an Organisation for which no Invite has been imported
> is committed on the chain anchor alone; the missing Invite is not a
> reason to refuse it.
>
> Note: "…This item said such an admission rests on the chain anchor and the
> signature alone, and that the Invite cross-check applies when an Invite
> exists. There is no signature and no cross-check."

**LLR-y2v8v2** (dec:984): delete "the pending invite it cross-checks the
sender against," from the list. Note: "…the pending Invite is no longer
compared with the sender; the one consumed is still the named
Organisation's."

**LLR-9zfnmb** (dec:892): replace ", so the cross-check LLR-j83kc8 performs
has one answer and survives a restart." with ", so the Invite a first
admission consumes is the latest one imported and survives a restart."
Note: "…the Invite is no longer a cross-check."

**LLR-zj88e6** (dec:873): replace ", so the joiner has a key to pin the first
admission's sender against." with "." Note: "…nothing on the joiner's side
compares that key with a sender since the owner's ruling."

**LLR-xq9nrq** (dec:1002), `satisfies: derived` (default answer to Q1):
> on first admission the new Organisation record holds, as its
> `admin_member_key`, the Organisation public key read from the chain in the
> same operation, as a `PersonPublicKey`, never a value carried in the Wire
> message or the Invite. It names no administrator: the chain publishes
> none.
>
> Note: "…This item said the field holds the Published signing key, which was
> the administrator's Member-as-a-group key. The chain's key is now the
> Organisation public key, an X25519 key distinct from every member key
> (REQ-ech45n). The field stays, as master left it, until chain-authority's
> change 1 removes every administrator field."

**LLR-e5c9ud** (dec:1015): text of LLR-p9xjze (ue:277-282), `satisfies: derived`;
note "…This item compared the Persona's device key with the administrator's
member key, correct only while both were one ed25519 key. The code compares
the two Member-as-a-group keys."

**SDD-72ddm6** (dec:1068), `traces: REQ-uxv2x2, REQ-nhe2zu, REQ-txvtm9`:
> **SDD-72ddm6**: what a node does when the change it receives removes
> **it** — the one path where committing a verified change means deleting
> the record rather than updating it. The item also owns the
> administrator's side that sends that change. Nothing about the sender is
> checked on this path.

**LLR-3q63zv** (dec:1103), `satisfies: derived`:
> `receive_and_self_delete_if_revoked` does not check the sender's
> authenticated Device key, on the branch that deletes the record and on the
> branch that updates it. A removal or an update relayed by any device is
> acted on when it verifies against the chain.
>
> Note: "…This item said the path does not cross-check the sender because a
> node being removed cannot be required to find the remover in a record it
> is no longer part of, and booked the update branch's missing check as
> PR-u4c2vp. The owner ruled that nothing about the sender is checked on
> either branch, so PR-u4c2vp is resolved by that ruling."

**LLR-6dc598** (dec:1170): text of LLR-8az5vt (ue:346-349); note "…said the
signed revocation".
**LLR-tax3pm** (dec:1177): text of LLR-3jybhn (ue:353-355) with "(LLR-by65qx)"
→ "(LLR-pzde8b)"; note "…said the revocation is signed by the
administrator's Member-as-a-group key and carries the last sequence number
plus one."
**LLR-8hdu9x** (dec:1215): text of LLR-tpb9xc (ue:359-362); note "…said the
signed envelope".

In `2026-10-04-type-safety.md`:

**LLR-ayrdr8** (ts:103): text of LLR-vp62sc (ue:496-508), with "(LLR-ayrdr8's)"
→ "master's", "(LLR-3fwykc)" kept and "(LLR-by65qx)" → "(LLR-e7s4ye)",
`satisfies: derived`; note "…The pinned values were master's; two changed,
each by one of the format changes this note names."
**LLR-8bum44** (ts:256): text of LLR-9s3nqp (ue:560-574) with its last
sentence replaced by "Importing an Invite whose Organisation public key,
administrator's Member-as-a-group key or administrator's DevicePublicKey is
not a valid key of its kind fails (`Chain("blob decode: …")`) and stores no
pending Invite.", `satisfies: REQ-qn2erx`; note "…This item said 'curve point'
for every key; the keys now have three kinds, each parsed by its own type."
**LLR-mmdu38, LLR-56hc77, LLR-bwb9pu**: unchanged (P4, Q4).
The dated notes on SDD-swtd3w and SDD-af5vnt that this branch added
(`dec:105-112`, `:509-513`) are reworded to name only surviving IDs:
LLR-322xfu, LLR-3jjgtw, LLR-sj7cd5, LLR-en9p5c, LLR-zyw5r2, LLR-3fwykc,
LLR-2dvhz8, and "LLR-ayrdr8 and LLR-8bum44 are amended in place". The
robustness paragraph note at `dec:1523-1527` names "LLR-9fvb3y, amended in
place" instead of the deleted items.

`2026-10-05-unsigned-envelope.md` is rewritten as
`# org-node — the X25519 keys and the Organisation key pair` with:

- an introduction: the owner's rulings of 2026-10-04 and 2026-10-05 (X25519
  member keys, the Organisation key pair, no signature, no sender check); the
  items of `2026-10-03-decomposition.md` and `2026-10-04-type-safety.md` that
  described a signed Envelope are amended in place there; this file holds the
  items new on this change.
- `## Under SDD-sxp8hb (keys.rs)`: LLR-3jjgtw, LLR-98ufry (with its review
  note), LLR-x35s6p (with its paragraph).
- `## Under SDD-na9nc3 (verify.rs)`: LLR-9f5hmr with its review note.
- `## Under SDD-swtd3w (types.rs)`: LLR-en9p5c (with its paragraph),
  LLR-322xfu.
- `## Under SDD-af5vnt (store.rs)`: LLR-zyw5r2.
- `## The Organisation private key's custody (REQ-ech45n)`: LLR-sj7cd5,
  LLR-3fwykc, LLR-2dvhz8, unchanged.
- `## Robustness`: one row per remaining item, taken from the two existing
  tables.
- Every other section, item and paragraph is deleted: the six replacement
  SDDs, the twenty-two LLRs this plan deletes, the "Signing wording" and
  "Tests" sections and the deleted items' table rows.

`soup.md`: the `ed25519-dalek` row's REQ column becomes `REQ-ag6kqm,
REQ-xa6smf, REQ-ztdza4`; its sentence "The sender-device check (REQ-7h7qp3)
compares the DevicePublicKey the iroh handshake authenticated, converted at …"
becomes "The iroh handshake authenticates the DevicePublicKey, converted at
`transport/endpoint.rs` by `VerifyingKey::from_bytes`; nothing compares it
with a record (REQ-ag6kqm)"; "(superseded by LLR-zyw5r2, which keeps the
reliance)" stays. The `serde` row's last sentence reports `Chain("blob
decode…")` and LLR-8bum44, not `InvalidInvite` and LLR-9s3nqp. The
`thiserror` row's note drops ", an Invite as `InvalidInvite`".

Step — `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-ids.sh`
exits 0. Commit the four files: `trim(org-node): architecture items amended in place, replacements withdrawn`.

### T8 — org-node requirements

**Files touched:** `org-node/docs/requirements/2026-09-09-verify-and-commit.md`, `org-node/docs/requirements/2026-10-05-envelope-authenticity.md`
**Parallel:** yes, with T5–T7, T9, T10

`2026-09-09-verify-and-commit.md`: C1, C2, C3 as given above; REQ-gju89b
back to `(implements: RC-pm9kmx)`. This branch's term note at `:11-18` stays.

`2026-10-05-envelope-authenticity.md` becomes:

```markdown
# The epoch rule and the Organisation public key

The on-chain `orgPubKey` is the Organisation public key, an X25519
key-agreement key (root `docs/CONTEXT.md`), and the Envelope carries no
signature (owner, 2026-10-05). The reasons and the risk assessment are in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

    **REQ-txvtm9**: The software shall commit a received Envelope only when its
Sequence number equals the epoch of the Organisation state on the chain it is
verified against, and shall otherwise refuse it, leaving its record unchanged.
(implements: RC-95dgg8)
satisfies: derived

*Amended 2026-10-05 (docs/plans/2026-10-05-switch-trim.md).* This item first
restated REQ-nhe2zu's commit conditions with a sender clause and the epoch
rule, and superseded it. By owner ruling REQ-nhe2zu is amended in place
instead, and this item states only the rule REQ-nhe2zu lacks.

    **REQ-8jb4ny**: (unchanged)

    **REQ-ech45n**: The software shall, when it creates an Organisation, publish
as its Organisation public key the X25519 public key of a freshly generated
secret, distinct from every Member-as-a-group key and DevicePublicKey in the
genesis record, and keep that secret in its encrypted store.
satisfies: derived
```

REQ-7h7qp3 is deleted.

Commit the two files: `trim(org-node): requirements, chain-authority's texts of the shared items`.

### T9 — org-node risk file

**Files touched:** `org-node/docs/risk/2026-09-09-org-node-hazards.md`, `org-node/docs/risk/2026-10-03-architecture-derived.md`, `org-node/docs/risk/2026-10-05-envelope-authenticity.md`
**Parallel:** yes, with T5–T8, T10

`2026-09-09-org-node-hazards.md`:

- **RC-pm9kmx** (`:168`), proposed text (Q2), `superseded-by:` deleted:

  ```markdown
  **RC-pm9kmx**: before a received Change set is decoded, the node requires the
  Envelope to name the Organisation the node expected, and rejects one that
  does not with a typed error and without decoding the Change set; it checks
  no signature and no key of the sender, and leaves authority over the Change
  set to the Membership root and epoch it reads from the chain (RC-6a2dke,
  RC-e5atck). mitigates: HAZ-tawvm2

  *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`;
  text written by change `worktree-person-shared-types`, which merges first).*
  This control first also required a valid signature over the Organisation
  identifier, the Sequence number and the Change set bytes by the published
  signing key. The right to change an Organisation's data lies in its on-chain
  multisig proxy, so the Envelope carries no signature and the chain is the
  sole authority.
  ```

- The `*Amended 2026-10-05. The Envelope carries no signature, and RC-2e6k44
  supersedes RC-pm9kmx …*` paragraph (`:207-219`) becomes: "*Amended
  2026-10-05 (RC-pm9kmx amended in place).* Step 2 of the order above, the
  signature check, is gone, and nothing replaces it: nothing about the sender
  is checked. The order of the rest is unchanged: the cheap checks still run
  before any attacker-chosen bytes are decoded, and RC-6a2dke and RC-e5atck
  still decide."
- The stranger-work paragraph's amendment (`:409-415`): "*(Amended
  2026-10-05: there is no signature check and no sender check. A stranger
  obtains per connection a chain read, a record rebuild and, on a first
  admission, the decode of the snapshot it sent.)*"
- **RC-b6mydy** (`:437`), proposed text (Q2):

  ```markdown
  **RC-b6mydy**: the node commits a received Wire message only through the
  checks of RC-pm9kmx, RC-6a2dke, RC-e5atck, RC-m4r75s and RC-95dgg8, and
  checks nothing about the Device key the connection authenticated: not on a
  first admission, with or without an imported Invite, and not on a later
  update, whether or not that key is in the Membership record before or after
  it. A chain-valid update is committed whoever delivers it; one the chain has
  not published is refused whoever delivers it.
  mitigates: HAZ-ep6uzs, HAZ-vxabf9
  ```

  followed by a dated note of the same form as RC-pm9kmx's, stating what the
  control was (the invite cross-check and the post-verification membership
  check), and the residual: "**Residual risk: not acceptable.** What the
  connection delivers beside the Change set is not covered: the Organisation
  secret (PR-ve9zw8) can now be substituted by any peer that relays a genuine
  admission or update, not only by a device in the record. Chain-authority's
  change 2 replaces the secret with the Organisation private key, checked on
  receipt against the chain's `org_pub_key`, and that is the control this
  hazard waits for." The existing three-places paragraphs stay as history
  under a heading line "*Before 2026-10-05:*".
- The HAZ-vxabf9 note (`:541-548`): "a device the receiver accepts setting the
  Sequence number to `u64::MAX`" → "any peer setting the Sequence number to
  `u64::MAX`".
- The administrator-authority note (`:885-895`): "A receiver accepts an
  Envelope from any DevicePublicKey in its record" → "A receiver accepts an
  Envelope from any peer"; "any member device can deliver" → "any peer can
  deliver".
- The RC-2e6k44 summary note (`:987-994`) becomes: "*Amended 2026-10-05.* Two
  rows above no longer match the code. For HAZ-tawvm2, "one author key" is
  gone: there is no author key, and nothing about the sender is checked; the
  decisive read is unchanged. For HAZ-ep6uzs, the sender checks are gone
  (RC-b6mydy amended in place); its residual is recorded there. PR-u4c2vp is
  resolved by the owner's ruling. PR-2dmjzj is still open. A re-scoring is
  the next risk analysis's job."

`2026-10-03-architecture-derived.md`: every note this branch added (`:108-117`,
`:224-231`, `:376-385`, `:511-517`, `:598-607`, `:630-638`, `:662-665`,
`:712-718`, `:764-766`) is rewritten to name the amended old item instead of
its deleted replacement, and to say "nothing about the sender is checked"
where it said "the sender check" or "the receiver checks instead that the
sending device is in its current record". The LLR-3q63zv section note
(`:108-117`): "*Amended 2026-10-05.* The carve-out this section defends is
now the rule on every path: the owner ruled that nothing about the sender is
checked (REQ-xa6smf, REQ-ztdza4). LLR-3q63zv is amended in place to say so,
and PR-u4c2vp is resolved by that ruling. The residual named above (any peer
that can reach the endpoint and relay a genuine, chain-anchored envelope
causes the node to act on it) is accepted by that ruling: the envelope
matches the chain." The LLR-xq9nrq note (`:598-607`) states Q1's default. The
LLR-mbjfq8 note (`:712-718`): "*Amended 2026-10-05.* The opposite happened:
the owner ruled that no Invite is required (REQ-xa6smf). LLR-mbjfq8 is
amended in place to the chain-anchor rule without the signature. The first of
the three places where RC-b6mydy was weaker than its wording is now its
wording."

`2026-10-05-envelope-authenticity.md` keeps its title and becomes:

- the owner's rulings list, with the third bullet: "the Envelope signature is
  dropped, and nothing about the sender is checked (owner ruling 2026-10-05,
  change `worktree-org-node-chain-authority`): what the sender delivers is
  verified against the chain, which is the sole authority";
- a paragraph "**What changes in HAZ-tawvm2's controls.**" saying RC-pm9kmx
  is amended in place (no signature), RC-6a2dke and RC-e5atck unchanged, and
  that any peer, not only a device in the record, reaches the decoder, whose
  surface is org-members' class C deserialisation (REQ-shk82j); it cannot make
  the node commit anything the chain has not published;
- the Sequence-number section, RC-95dgg8 and its paragraph, with "(any device
  in its record, under RC-2e6k44)" → "(any peer, since nothing about the
  sender is checked)";
- the Organisation-secret section, with "any device in the receiver's record
  reaches the write" → "any peer reaches the write";
- `## Derived requirements assessment`: REQ-txvtm9 (the epoch rule, RC-95dgg8,
  no new hazard) `assesses: REQ-txvtm9`; REQ-8jb4ny and REQ-ech45n unchanged
  with "the private key is held only in the creating node's encrypted store"
  kept (LLR-3fwykc still says so);
- `## Derived low-level requirements`: the existing passages, each
  `assesses:` line retargeted to the old ID (LLR-rb8r65, LLR-ghja3x,
  LLR-6dc598, LLR-tax3pm, LLR-8hdu9x; LLR-rv4vux; LLR-vdyu65; LLR-e5c9ud;
  LLR-s7yu4k; LLR-ayrdr8), their "A sender-side mistake that used to fail the
  receiver's signature check now fails its sender check" sentence replaced
  by "A sender-side mistake that used to fail the receiver's signature check
  now fails the chain checks, or is committed if it matches the chain";
  LLR-98ufry, LLR-en9p5c, LLR-x35s6p unchanged; a new passage "**A keypair
  rebuilt from its seed: LLR-e58j8m.** Amended in place to drop the
  signature clause; it is derived now. A rebuilt keypair with a different
  public key would make a Persona's key in the trie differ from the one its
  node uses; the round-trip tests refuse that. No new hazard. assesses:
  LLR-e58j8m";
- deleted: RC-2e6k44, "What it gives up", "The removal path", the LLR-rys5nx
  passage, the LLR-5svrw8 passage.

Commit the three files: `trim(org-node): risk file, controls amended in place`.

### T10 — org-node problem reports

**Files touched:** `org-node/docs/problems/2026-10-04-org-key-conflation.md`, `org-node/docs/problems/2026-09-09-org-node-problems.md`, `org-node/docs/problems/2026-10-04-non-strict-verify.md`, `org-node/docs/problems/2026-10-05-org-secret-unauthenticated.md`
**Parallel:** yes, with T5–T9

- **PR-szkat6**: `status: open`; delete the `resolution:` line; replace the
  "**Resolved 2026-10-05 …**" paragraph with: "**Reopened 2026-10-05**
  (docs/plans/2026-10-05-switch-trim.md). Change `worktree-person-shared-types`
  makes `org_pub_key` the public half of a fresh X25519 Organisation key pair,
  distinct from every genesis key (REQ-ech45n, LLR-sj7cd5, LLR-3fwykc,
  LLR-322xfu), parses it by `person`'s X25519 rule (LLR-3jjgtw, LLR-en9p5c),
  and verifies nothing under it (REQ-ag6kqm as amended). What the design
  intends is not done yet: the Organisation private key is not given to the
  Members. Chain-authority's change 2 (admission sends the Organisation
  private key, which a receiver checks against the chain's `org_pub_key`)
  resolves this report." `opened: 2026-10-04` stays.
- **PR-u4c2vp** (default answer to Q3): `status: resolved`; `resolution:` "not
  a defect, by owner ruling 2026-10-05 (change
  `worktree-org-node-chain-authority`): nothing about the sender of an
  Envelope is checked on either Receive operation, because authority is the
  chain's and a chain-valid update delivered by any peer matches the chain
  (REQ-ztdza4 as amended). LLR-3q63zv is amended in place to state it; test
  `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`
  (`org-node/tests/admission_sender.rs`)."
- **PR-vkw22m**: `resolution:` "root cause — `keys::verify` checked Envelope
  signatures with ed25519-dalek's non-strict `VerifyingKey::verify`; fixed by
  change `worktree-person-shared-types`, which removes the Envelope signature
  (REQ-ag6kqm, LLR-na7p4w and LLR-9fvb3y amended in place), so org-node
  checks no signature and `keys::verify` is gone; test
  `the_wire_form_has_no_signature_field` (`org-node/tests/envelope_binding.rs`)."
  The "**Resolved 2026-10-05**" paragraph stays.
- **PR-ve9zw8**: `affects: SDD-8cpyfa, LLR-ckk5nz, RC-b6mydy`; definition
  "…A node takes whatever secret the Wire message carries, from any sender:
  nothing about the sender is checked (REQ-xa6smf, REQ-ztdza4). Any peer that
  relays a genuine admission or update can therefore give a member a secret
  of its choosing…"; "What was observed" says "any peer now reaches this
  write"; "What closing it looks like" adds: "*Noted 2026-10-05.* The owner's
  chain-authority rulings make the Organisation secret the Organisation
  private key, refused on receipt unless its X25519 public half equals the
  on-chain `org_pub_key`. That is chain-authority's change 2, and it closes
  this report. Whether that supersedes the CGKA wording above is Q2 of
  docs/plans/2026-10-05-switch-trim.md."
- The resolution lines of PR-szkat6's "Found" notes and of any other report
  that names a deleted ID: T11's sweep finds them.

Commit the four files: `trim(org-node): problem reports after the trim`.

### T11 — Glossaries, READMEs, comments: no deleted ID, no sender check

**Files touched:** `org-node/docs/CONTEXT.md`, `docs/CONTEXT.md`, `org-node/README.md`, `org-node/AGENTS.md`, `org-node/src/service.rs`, `org-node/src/verify.rs`, `org-node/src/error.rs`, `org-node/src/keys.rs`, `org-node/src/types.rs`, `org-node/src/store.rs`, `org-node/src/blobs.rs`, `org-node/src/test_fixtures.rs`, `org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`, `org-node/tests/verify_against_chain.rs`, `org-node/tests/organisation_key.rs`, `org-node/tests/node_value_types.rs`, `org-node/tests/secret_redaction.rs`
**Parallel:** no (serial, after T1–T10)

Step 1 — find every remaining reference:

```
grep -rnE 'REQ-7h7qp3|RC-2e6k44|LLR-(qgjw4n|ds2gyj|fj3f24|6wpk63|5svrw8|rys5nx|bg6xty|9umxv2|by65qx|w8ny8z|2ad4du|53phvg|28qhcd|p9xjze|8az5vt|3jybhn|tpb9xc|23kqye|wg4hj2|h57d27|vp62sc|9s3nqp)|SDD-(wxu2cg|m5uxzf|my6mvj|5sxp7v|zqc75b|cp8g7z)|UnknownSender|NoImportedInvite|InviteOrgKeyMismatch|InvalidInvite|accepted_senders|sender check|sender-device' org-node/docs org-node/src org-node/tests org-node/README.md org-node/AGENTS.md docs/CONTEXT.md app/docs app/src-tauri
```

Step 2 — reword each hit. In the glossaries and README: "after checking org
binding + sender + sequence" → "after checking org binding + sequence";
the *Envelope* entry: "It carries no signature: the chain's root at a newer
epoch decides, and nothing about the sender is checked."; `docs/CONTEXT.md`'s
*Invite* entry keeps the administrator's keys (they stay until
chain-authority's change 1) without saying they are checked. In `src` and
`tests` comments: the amended old ID replaces its deleted replacement, using
the inventory table; a comment about a removed check is deleted.

Step 3 — the grep prints nothing. ORG-NODE-GATE passes.

Commit: `trim: no reference to a withdrawn item or a removed check`.

### T12 — Configuration notes and counts

**Files touched:** `org-node/.guardrails/config.yaml`, `docs/plans/2026-10-04-person-shared-types.md`
**Parallel:** no (serial, after T11)

`org-node/.guardrails/config.yaml`: in the "Merged 2026-10-05 into
worktree-person-shared-types" comment, replace "The Envelope lost its
signature; the sender-device rule (REQ-7h7qp3) took its place in
verify_against_chain and admission_sender." by "The Envelope lost its
signature; by owner ruling nothing replaced it (trimmed 2026-10-05,
docs/plans/2026-10-05-switch-trim.md)."; append a dated line with the pass
counts T13 measures. Also the `problem_open_max` comment: "PR-szkat6 is open
again after the trim; PR-vkw22m is resolved." The list stays contiguous (no
comment between `- item` lines).

`docs/plans/2026-10-04-person-shared-types.md`: append a dated line under
step 4: "Trimmed 2026-10-05 to what chain-authority agrees with:
docs/plans/2026-10-05-switch-trim.md."

Commit: `trim: configuration notes and measured counts`.

### T13 — Gates

**Files touched:** none (records `$S/trim-gate-*.log`)
**Parallel:** no (last)

Run, each to a log, each must exit 0:

1. ORG-NODE-GATE, then the seven quint lines of `org-node/.guardrails/config.yaml`
   with `QUINT_HOME` set.
2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test finality_polling --test preflight --no-run`
3. APP-GATE; `npm --prefix app run check`; `npm --prefix app run test`.
4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-members` and the five
   quint lines of `org-members/.guardrails/config.yaml`.
5. The cargo line of `on-chain-client/.guardrails/config.yaml`.
6. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p person`;
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p person --all-targets -- -D warnings`.
7. For each unit `u` in `org-members on-chain-client org-node app person`:
   `GR_CONFIG=$u/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
   and `GR_CONFIG=$u/.guardrails/config.yaml .guardrails/scripts/check-ids.sh`.
   Expected: exit 0; `UNRESOLVED-PR` warnings only (org-node gains PR-szkat6).
8. `.guardrails/scripts/check-units.sh`.

Paste the tail of each log into the change's verification record when
`verify-before-merge` runs.

## Order and parallelism

| Task | Title | Parallel |
|---|---|---|
| T1 | Verification and the self-delete path check nothing about the sender | no, first |
| T2 | A first admission commits whoever sends it, with or without an Invite | no, after T1 |
| T3 | An update commits whoever sends it | no, after T2 |
| T4 | Refusals of the removed checks gone; Invite import fails as master's | no, after T3 |
| T5 | App classifies only the remaining refusals | yes, after T4, with T6–T10 |
| T6 | Tests carry only IDs whose text they verify | yes, after T4, with T5, T7–T10 |
| T7 | Architecture ledger | yes, any time, with T5, T6, T8–T10 |
| T8 | Requirements | yes, any time |
| T9 | Risk file | yes, any time |
| T10 | Problem reports | yes, any time |
| T11 | Sweep: no deleted ID, no removed check | no, after T1–T10 |
| T12 | Configuration notes and counts | no, after T11 |
| T13 | Gates | no, last |

T5–T10 have pairwise disjoint file sets; T1–T4 share `service.rs` and
`admission_sender.rs`; T11 touches files of T1–T7 and so waits for them.

## Self-review (plan-change step 9)

1. Every ID in **Implements** has a task whose test verifies it: REQ-ag6kqm
   (T1 step 6, T3 step 1), REQ-nhe2zu (T6 happy path), REQ-xa6smf (T2 a–c),
   REQ-ztdza4 (T3 a–b), REQ-txvtm9 (T6 `:451`), REQ-ech45n and REQ-8jb4ny
   (`organisation_key.rs`, unchanged), REQ-gju89b (T6 `:101`), REQ-kn5rtx
   (T5). RC-pm9kmx and RC-b6mydy are implemented by REQ-ag6kqm/REQ-gju89b
   and REQ-xa6smf/REQ-ztdza4.
2. Each behaviour change is a red test first: T1 steps 1–6, T2 step 1, T3
   step 1, T4 step 1, T5 step 1 (compile).
3. Names used across tasks: `VerifyContext { expected_org_id, seq_guard,
   last_committed_epoch }`; `admin_member_key` stays `PersonPublicKey`; the
   renamed tests are named identically in T1–T3 and T6.
4. Every task states **Files touched** and **Parallel**; T5–T10 share no file.

## Open questions for the owner

- **Q1 — `admin_member_key` on a member's record.** With LLR-rys5nx gone, a
  first admission has no administrator key to record: the chain's key is no
  longer one. This plan's default writes the chain's Organisation public key
  into the field (master's source, LLR-xq9nrq amended), which names no
  administrator; on a member's node `admin_persona_for_org` then finds none,
  and LLR-e5c9ud's exclusion never excludes. Alternatives: keep taking it from
  the Invite when one was imported (LLR-rys5nx, which the trim rules
  removed); or make the field `Option` (a format change chain-authority
  undoes when it removes the field).
- **Q2 — RC-pm9kmx and RC-b6mydy, and HAZ-ep6uzs in the interim.**
  Chain-authority says both controls are amended in place but has written no
  text. T9 proposes text; chain-authority should copy it so the merge is
  clean. RC-b6mydy was HAZ-ep6uzs's only control. Once this trim merges, any
  peer that relays a genuine admission or update can substitute the
  Organisation secret (PR-ve9zw8 widens from "any device in the record" to
  "any peer"), and that stays so until chain-authority's change 2. Is master
  allowed to carry that state, or should the trim merge only together with
  change 2? Also: does change 2's receipt check supersede the earlier ruling
  that the secret is replaced by CGKA keys?
- **Q3 — PR-u4c2vp.** This plan's default resolves it as "not a defect, by
  ruling", citing chain-authority's REQ-ztdza4 rationale. The alternative is
  to reopen it until chain-authority merges.
- **Q4 — KEEP-named replacements whose old text is no longer true.**
  LLR-mmdu38 (Edwards rule, `From<&P2pMemberKey>`) → LLR-en9p5c; LLR-56hc77
  (`MemberSeed::signing_keypair`) → LLR-x35s6p; org-members' LLR-k6dhz7
  ("neither refuses a small-order key") → LLR-4sm3b5. Their tests carry both
  IDs, and no test honestly verifies the old texts. LLR-bwb9pu → LLR-zyw5r2
  is borderline: the test does verify the five values the old item lists,
  but the old item calls them every value that holds a secret. The KEEP rule
  named the replacements, so this plan keeps all four pairs. The convention
  would fold each into its old item and delete the replacement.
- **Q5 — `satisfies:` lines of master LLRs that name REQ-xa6smf or
  REQ-ztdza4** (LLR-ygn78w, LLR-kkj64b, LLR-bg3vsw, LLR-jn5jeh, LLR-9zfnmb,
  LLR-g9vmbx, LLR-ctzkv7, LLR-6zjzn2, LLR-cns6q6 and others) now refine REQs
  that say nothing is checked about the sender. This plan leaves them for
  chain-authority's change 1, which removes Invites and Join requests and
  touches most of them. Tests no longer annotate those two REQs except where
  they commit a first admission or an update.
- **Q6 — the in-place texts of master's signature LLRs** (LLR-na7p4w,
  LLR-9fvb3y, LLR-e7s4ye, LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa, LLR-pzde8b,
  LLR-mcdh85, LLR-37cj3n) are this plan's own wording. Chain-authority's
  plan counted "about twenty-five LLRs" to amend; since it builds on this
  branch, it inherits these texts. Does the owner want chain-authority's
  interview to review them first?

## Owner answers (2026-10-05)

These override the defaults above wherever the tasks say "default answer to Qn".

- **Q1 — keep from the Invite.** On a first admission for which an Invite was
  imported, `admin_member_key` is the Invite's administrator Member key
  (LLR-rys5nx's source stays). The Invite's keys keep the type they have on
  this branch (`PersonPublicKey`, `DevicePublicKey`, parsed when the Invite is
  decoded), but this branch's added checks go: no `InviteOrgKeyMismatch`, and
  no sender check against the Invite's device key. A first admission with no
  imported Invite now commits (T2); for that case only, the dispatcher fills
  the field from the chain's Organisation public key (master's source), since
  the ruling names no other value. So LLR-rys5nx is **not deleted**: it is
  reworded in place (this branch's own, unmerged) to state both sources, and
  LLR-xq9nrq's in-place text and LLR-e5c9ud's exclusion follow. Test
  `a_first_admission_records_the_invites_administrator_…` keeps its name and
  its Invite assertion; a second test covers the no-Invite case.
- **Q2 — merge, record the gap.** The trim merges on its own. The risk file
  records that any peer relaying a genuine update can substitute the
  Organisation secret until chain-authority's change 2 adds the receipt check;
  PR-ve9zw8 stays open, retargeted to "any peer", naming that check as its
  fix. T9's proposed RC-pm9kmx and RC-b6mydy texts stand, for chain-authority
  to copy. Whether change 2's receipt check supersedes the earlier CGKA
  wording was not answered; the risk file names both, unranked.
- **Q3 — resolved by ruling** (the default).
- **Q4 — fold in place.** LLR-mmdu38, LLR-56hc77, LLR-bwb9pu and org-members'
  LLR-k6dhz7 are amended in place to the current behaviour with a dated note;
  their replacements LLR-en9p5c, LLR-x35s6p, LLR-zyw5r2 and LLR-4sm3b5 are
  deleted, their content folded into the old items, and their tests carry the
  old ID only. T7 (org-node) and the org-members ledger take this on; the
  org-members part is a new task **T7b** (files:
  `org-members/docs/architecture/2026-10-04-key-parse.md`,
  `org-members/docs/architecture/2026-10-05-device-key-validity.md`,
  `org-members/docs/risk/2026-10-04-key-parse.md`,
  `org-members/docs/risk/2026-10-05-device-and-member-key-validity.md`,
  `org-members/tests/newtypes.rs`; parallel: yes).
- **Q5, Q6 — not put to the owner; the dispatcher takes the defaults.** The
  stale `satisfies:` lines are left for chain-authority's change 1, and the
  in-place texts of master's signature LLRs are this plan's wording, which
  chain-authority may review when it builds on this branch.
