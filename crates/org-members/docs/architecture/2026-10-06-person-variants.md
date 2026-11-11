# How a `person::IdentityError` leaves org-members — SDD-m9gs5g

Added by the Person definition change (2026-10-06), which gave
`person::IdentityError` five variants that only `person`'s Person operations
produce. SDD-m9gs5g (the typed error, `src/error.rs`) is defined in
`2026-09-17-decomposition.md`. Before this change the `From` mapping was stated
only one variant at a time, in the LLRs of the constructor that reports each
(LLR-w5nkbu, LLR-z954wj, LLR-a645bx, LLR-st6j2r); none stated it whole. The
item that states it whole is LLR-hbb4je, in
`2026-10-24-on-sparse-merkle-tree.md`.
