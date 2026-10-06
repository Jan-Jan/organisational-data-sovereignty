# How a `person::IdentityError` leaves org-members — SDD-m9gs5g

Added by the Person definition change (2026-10-06), which gave
`person::IdentityError` five variants that only `person`'s Person operations
produce. SDD-m9gs5g (the typed error, `src/error.rs`) is defined in
`2026-09-17-decomposition.md`. Before this item the `From` mapping was stated
only one variant at a time, in the LLRs of the constructor that reports each
(LLR-w5nkbu, LLR-z954wj, LLR-a645bx, LLR-st6j2r); none stated it whole.

**LLR-28ekrv**: `From<person::IdentityError> for OrgMembersError` maps every
`person::IdentityError` variant to an `OrgMembersError` variant, by an
exhaustive match with no wildcard arm, so that a variant `person` adds stops
org-members compiling until its mapping is chosen. `FieldTooLong { field, max }`,
`InvalidDeviceKey`, `InvalidPersonKey`, `DeviceSlotsFull`, `DuplicateDevice`
and `DeviceNotFound` map to org-members' variants of the same names and
fields. `KeyWithoutDevice`, `MissingPersonKey`, `PersonKeyIsDeviceKey`,
`PersonKeyNotRotated` and `UnsupportedEncodingVersion(_)` map to
`InvariantViolated`: only `person`'s Person operations (`Person::new`,
`check_group_key`, `check_successor`, `person_hash`, `verify`) produce them,
org-members calls none of those, so one reaching org-members is an internal
invariant violation.
satisfies: REQ-ds8ryr

The five `InvariantViolated` arms are provisional. The change that aligns the
Member key rules with the Person's (`docs/plans/2026-10-04-person-sequencing.md`,
change 3) makes org-members call `check_group_key`, and then
`KeyWithoutDevice`, `MissingPersonKey` and `PersonKeyIsDeviceKey` become
rejections a caller can cause; that change revisits their mapping and amends
this item.
