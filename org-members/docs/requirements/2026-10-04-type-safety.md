# Type-safety requirements

Decided 2026-10-04 in the `grill-requirements` interview that introduced the
parse-at-the-system-edge rule (plain types only where data enters the system,
parsed once into a validated or tag newtype that every signature then takes;
see `org-members/AGENTS.md`). Handle lookups previously answered "not found"
for a string that is not a valid handle; the owner chose that a malformed
query be reported as invalid, distinct from a valid handle nobody holds. The
invalid outcome is now produced when the string is parsed into a handle, before
any lookup runs.

REQ-h5ret5, whose rules REQ-t46uad cites, was amended in this change by owner
ruling (2026-10-04), in place in `2026-08-31-org-membership.md`: it now states
the identifier-character rule (UTS#39 General Security Profile, `-` excepted)
that `Handle::parse` has applied since 2026-05-12.

## Handle lookup

**REQ-t46uad**: The software shall answer a query for a member by handle with
one of three distinct outcomes: the handle is invalid under the rules of
REQ-h5ret5 (reported as an invalid-handle error), the handle is valid and held
by no member, or the handle is valid and held by a member (that member's
record, for a lookup).
satisfies: derived
exported: yes
