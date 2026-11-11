//! The handle's clock (SDD-3my66m, LLR-fzf2n3): one reading per verification, whole seconds since
//! the Unix epoch; org-io keeps no time of its own. Replaced under test-support only (LLR-6vtbn6).

/// A clock: `None` for a reading before the Unix epoch.
pub trait Clock: Send + Sync {
    fn now(&self) -> Option<u64>;
}

/// The production clock: `std::time::SystemTime::now()`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Option<u64> {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|elapsed| elapsed.as_secs())
    }
}
