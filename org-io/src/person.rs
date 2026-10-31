//! The Person (SDD-j7k844): the user's Person, proposed apart from any Organisation, kept as a
//! small sealed Person file and its own keychain item, reported provisional. Its types are
//! org-node's (`PersonStore`); org-io names no `person` crate (LLR-b97usg).

use crate::view::PersonSummary;
use crate::{store_file, CreateError, OrgIo, StoreFailure};

impl OrgIo {
    /// Propose the Person (LLR-3cysjk): org-node builds the Person store from `name`, `surname`
    /// and keys drawn from the handle's source; its Device seed is written to its keychain item
    /// first, then the sealed bytes to `person-<hex>.store`; only then is it held and returned.
    /// One Person per device: while one is held a proposal is refused with `PersonAlreadyHeld`
    /// naming it, before any draw or write (owner ruling 2026-10-22). The Persons' lock is held
    /// for the whole (synchronous) body, so two calls cannot both pass the check.
    #[allow(clippy::result_large_err, reason = "a refusal names its Device public key, a parsed point; proposed once per device")]
    pub async fn propose_person(&self, name: &str, surname: &str) -> Result<PersonSummary, CreateError> {
        let mut persons = self.local.persons.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(held) = persons.first() {
            return Err(CreateError::PersonAlreadyHeld { device_key: held.device_key() });
        }
        let created = {
            let mut rng = self.local.rng.lock();
            org_node::PersonStore::create(&mut *rng, name, surname)
        };
        let (person, secret, sealed) = created.map_err(CreateError::Node)?;
        let summary = PersonSummary::of(person.view());
        self.local.keychain.keep(secret).map_err(CreateError::Keychain)?;
        if let Some(data_dir) = &self.local.data_dir {
            store_file::write_person_store(data_dir, &summary.device_key, &sealed, &self.local.rng)
                .map_err(|failed| CreateError::Store(StoreFailure::NotKept { failed }))?;
        }
        persons.push(person);
        Ok(summary)
    }

    /// The one Person this node holds, or `None` (LLR-mas8vc, LLR-z7kbf9).
    pub async fn person(&self) -> Option<PersonSummary> {
        self.local
            .persons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .map(|person| PersonSummary::of(person.view()))
    }
}
