//! Whether one Person definition may follow another. SDD-9hej83.

use crate::definition::Person;
use crate::error::IdentityError;

/// Refuses, with `PersonKeyNotRotated`, a `next` whose device keys differ
/// from `previous`'s while it holds the same PersonPublicKey (both present
/// and equal). Every other pair is accepted, a key change with unchanged
/// device keys included. Only the immediate predecessor is compared (owner,
/// 2026-10-04). LLR-wqha8d.
pub fn check_successor(previous: &Person, next: &Person) -> Result<(), IdentityError> {
    let devices_changed = previous.devices() != next.devices();
    let key_kept = previous.person_key().is_some() && previous.person_key() == next.person_key();
    if devices_changed && key_kept {
        Err(IdentityError::PersonKeyNotRotated)
    } else {
        Ok(())
    }
}
