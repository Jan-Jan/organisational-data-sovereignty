//! Reserved for a verifier that closes the loop with
//! `org_members::CandidateTrie::verify_against`, comparing a candidate trie's
//! root against on-chain state read at a given block; the trie crate owns the
//! actual SMT verification. It holds no code: the crate does not depend on
//! `org-members`.
