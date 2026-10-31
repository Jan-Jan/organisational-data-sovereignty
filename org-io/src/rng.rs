//! The handle's random source (SDD-3qwjs2): one cryptographic generator `OrgIo` owns, through its
//! `LocalStore`, and draws every random value of its operations from — the keys org-node draws for a
//! Person, a Persona, a provisional organisation and an Organisation key pair, the invite identifier,
//! the store nonces, and the store file's temporary suffix — so that no caller supplies or sees one
//! (LLR-4qd5fx, LLR-ctm86b). Production builds hold `rand_core::OsRng` (rand_core 0.6.4 over
//! `getrandom`); a test build may replace it (LLR-5acgfg). A draw org-node makes through
//! `fill_bytes` panics on an entropy failure, inside the org-node call and before any store change
//! is returned (LLR-94nn6q); org-io's own draw uses `try_fill_bytes` and refuses.

use std::sync::{Mutex, MutexGuard, PoisonError};

use rand_core::{CryptoRngCore, OsRng};

/// The source, behind its own lock. Not `Clone`, no `Debug`, and no public path reaches it.
pub(crate) struct HandleRng {
    source: Mutex<Box<dyn CryptoRngCore + Send>>,
}

impl HandleRng {
    /// The operating system's source: the only one a production build holds.
    pub(crate) fn os() -> Self {
        Self { source: Mutex::new(Box::new(OsRng)) }
    }

    /// The source replaced by `source` (test builds only; `OrgIo::with_rng_for_test`).
    #[cfg(feature = "test-support")]
    pub(crate) fn replaced_by(source: Box<dyn CryptoRngCore + Send>) -> Self {
        Self { source: Mutex::new(source) }
    }

    /// The source, for one synchronous draw. A lock poisoned by a draw that panicked (a failed
    /// entropy source, LLR-94nn6q) is taken as it is: the panic already refused that call.
    pub(crate) fn lock(&self) -> MutexGuard<'_, Box<dyn CryptoRngCore + Send>> {
        self.source.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
