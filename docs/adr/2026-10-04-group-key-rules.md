# ADR: One set of group-key rules for Members and Persons, compatible with TreeKEM

- **Date:** 2026-10-04
- **Status:** accepted (2026-10-04)
- **Relates to:** REQ-ht3x78, REQ-7qgx2q, REQ-bhez2u; to supersede REQ-ewdg2q
  and REQ-r784fu (docs/plans/2026-10-04-person-sequencing.md, change 3)
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

The Member-as-a-group key and the PersonPublicKey follow one rule set: no such
key with zero DevicePublicKeys; with one or more, such a key distinct from every
DevicePublicKey; a change whenever DevicePublicKeys are added, removed or replaced; and
free rotation otherwise. The first proposal made the key equal to the sole
DevicePublicKey and forbade rotation without a device change; both were dropped
because MLS's TreeKEM and Keyhive's BeeKEM give even a one-device group its own
key and recover from a leaked secret by rotating it in place, and this layer
must stay drivable by them. Removing the key at zero devices, rather than
installing a fresh one as isolation does today, means revoking every device
leaves nothing any removed device could derive access from; requiring a change
on an addition is stricter than MLS's minimum, so the key-agreement layer must
always commit with an update path.
