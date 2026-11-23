# Design — a kept address that names no host is not dialled (org-io)

Low-level requirement for change `worktree-networked-send-fanout`. It
replaces the send path's address rule (SDD-2bzgtp) of
`2026-10-16-domain-api.md`, which dialled every parsed kept address. No new software
item, dependency edge or SOUP. Class C, no per-item override.

## The send path's address rule (SDD-2bzgtp)

**LLR-2tp7bk**: in Networked mode each send dials the recipient by the
endpoint id equal to its Device key, with the addresses kept for that key
(or the Invite's, LLR-qa4nrt) as a hint ahead of discovery, and with none
when none is kept; in Loopback mode a send to a recipient refuses, sending
nothing to it, with `SendError::NoLoopbackAddress` when no address is kept
for that recipient and `SendError::AddressNotARecipient` when no kept
address's endpoint identity is the recipient's Device key, and otherwise
dials only the kept addresses whose identity is that key; each recipient is
judged on its own addresses, so one recipient's refusal leaves the others
attempted. A kept address is read in the text form of LLR-v9dkrv, its
endpoint identity the 64 hex characters before `@` and its socket the rest;
an entry of any other form, or whose socket's IP, read in its canonical
form (`::ffff:a.b.c.d` as `a.b.c.d`), is unspecified, broadcast or
multicast, names no identity and is not dialled.
satisfies: REQ-8trkps
