//! The store files (SDD-egx8rv): each Persona's sealed store is the file
//! `<data_dir>/persona-<64 lower-case hex of its Device key>.store`
//! (LLR-nt8dja), and the Person's sealed store the file
//! `<data_dir>/person-<64 lower-case hex of its DevicePublicKey>.store`
//! (LLR-3cysjk, LLR-z7kbf9), each replaced atomically through a temporary
//! file in the same directory (LLR-b9bm36). A Persona's file is renamed to
//! `<data_dir>/removed-<hex>.store` and removed when its removed Persona is
//! deleted (LLR-vj2nkb). The files hold only what org-node sealed; nothing here
//! reads a key or a seed. No mode or ACL is set on any file: the platform's
//! defaults apply (owner ruling, 2026-10-07).

use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use org_node::{DevicePublicKey, SealedStore, StoreWrite};
use rand_core::RngCore;

use crate::rng::HandleRng;

const PREFIX: &str = "persona-";
const PERSON_PREFIX: &str = "person-";
const REMOVED_PREFIX: &str = "removed-";
const SUFFIX: &str = ".store";

/// The step of a store file operation that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreFileStep {
    /// Listing the data directory.
    List,
    /// Reading a store file.
    Read,
    /// Creating the temporary file.
    CreateTemporary,
    /// Writing the sealed bytes to the temporary file.
    Write,
    /// Syncing the temporary file.
    Sync,
    /// Renaming the temporary file over the store file.
    Rename,
    /// Syncing the directory after the rename or the removal (Unix).
    SyncDirectory,
    /// Removing a deleted Persona's removed file.
    Remove,
}

impl fmt::Display for StoreFileStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::List => "list",
            Self::Read => "read",
            Self::CreateTemporary => "create temporary",
            Self::Write => "write",
            Self::Sync => "sync",
            Self::Rename => "rename",
            Self::SyncDirectory => "sync directory",
            Self::Remove => "remove",
        })
    }
}

/// A store file operation that failed: the step, the path it acted on, and
/// the operating system's reason (LLR-b9bm36).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreFileError {
    pub step: StoreFileStep,
    pub path: PathBuf,
    pub reason: String,
}

impl StoreFileError {
    fn new(step: StoreFileStep, path: &Path, reason: impl fmt::Display) -> Self {
        Self { step, path: path.to_path_buf(), reason: reason.to_string() }
    }
}

impl fmt::Display for StoreFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: {}", self.step, self.path.display(), self.reason)
    }
}

impl std::error::Error for StoreFileError {}

/// The store file of the Persona whose Device key is `device_key`
/// (LLR-nt8dja).
pub fn store_path(data_dir: &Path, device_key: &DevicePublicKey) -> PathBuf {
    data_dir.join(format!("{PREFIX}{}{SUFFIX}", hex::encode(device_key.as_bytes())))
}

/// The Person file of the Person whose DevicePublicKey is `device_key`
/// (LLR-3cysjk, LLR-z7kbf9).
pub fn person_path(data_dir: &Path, device_key: &DevicePublicKey) -> PathBuf {
    data_dir.join(format!("{PERSON_PREFIX}{}{SUFFIX}", hex::encode(device_key.as_bytes())))
}

/// The name a Persona's store file has once its deletion began
/// (LLR-vj2nkb).
pub(crate) fn removed_path(data_dir: &Path, device_key: &DevicePublicKey) -> PathBuf {
    data_dir.join(format!("{REMOVED_PREFIX}{}{SUFFIX}", hex::encode(device_key.as_bytes())))
}

/// The DevicePublicKey a file name with `prefix` carries, or `None` for any
/// other name (LLR-nt8dja; the disk is an edge, so the name is parsed).
fn key_named(name: &str, prefix: &str) -> Option<DevicePublicKey> {
    let hex_key = name.strip_prefix(prefix)?.strip_suffix(SUFFIX)?;
    let lower_case_hex = |character: char| character.is_ascii_digit() || ('a'..='f').contains(&character);
    if hex_key.len() != 64 || !hex_key.chars().all(lower_case_hex) {
        return None;
    }
    let bytes: [u8; 32] = hex::decode(hex_key).ok()?.try_into().ok()?;
    DevicePublicKey::parse(&bytes).ok()
}

/// The Device keys of the files in `data_dir` named `<prefix><hex>.store`,
/// in byte order.
fn list_named(data_dir: &Path, prefix: &str) -> Result<Vec<DevicePublicKey>, StoreFileError> {
    let entries = std::fs::read_dir(data_dir).map_err(|error| StoreFileError::new(StoreFileStep::List, data_dir, error))?;
    let mut keys = vec![];
    for entry in entries {
        let entry = entry.map_err(|error| StoreFileError::new(StoreFileStep::List, data_dir, error))?;
        if let Some(device_key) = entry.file_name().to_str().and_then(|name| key_named(name, prefix)) {
            keys.push(device_key);
        }
    }
    keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(keys)
}

/// The Device keys of the Persona store files in `data_dir`, in byte
/// order; every other name, Person files included, is ignored
/// (LLR-nt8dja, LLR-z7kbf9).
pub fn list_stores(data_dir: &Path) -> Result<Vec<DevicePublicKey>, StoreFileError> {
    list_named(data_dir, PREFIX)
}

/// The Device keys of the Person files in `data_dir`, in byte order
/// (LLR-z7kbf9).
pub fn list_person_stores(data_dir: &Path) -> Result<Vec<DevicePublicKey>, StoreFileError> {
    list_named(data_dir, PERSON_PREFIX)
}

/// The Device keys of the `removed-<hex>.store` files in `data_dir`, in
/// byte order (LLR-upqj5g).
pub(crate) fn list_removed(data_dir: &Path) -> Result<Vec<DevicePublicKey>, StoreFileError> {
    list_named(data_dir, REMOVED_PREFIX)
}

/// Rename `device_key`'s store file to its removed name, and on Unix sync
/// the directory (LLR-vj2nkb): the first step of the Persona's deletion,
/// called only from `retention.rs`.
pub(crate) fn mark_removed(data_dir: &Path, device_key: &DevicePublicKey) -> Result<(), StoreFileError> {
    let (from, to) = (store_path(data_dir, device_key), removed_path(data_dir, device_key));
    std::fs::rename(&from, &to).map_err(|error| StoreFileError::new(StoreFileStep::Rename, &from, error))?;
    sync_directory(data_dir)
}

/// Remove `device_key`'s removed file, and on Unix sync the directory
/// (LLR-vj2nkb): the last file step of the deletion, called only from
/// `retention.rs`. A file already gone counts as removed.
pub(crate) fn remove_removed_file(data_dir: &Path, device_key: &DevicePublicKey) -> Result<(), StoreFileError> {
    let path = removed_path(data_dir, device_key);
    match std::fs::remove_file(&path) {
        Ok(()) => sync_directory(data_dir),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(StoreFileError::new(StoreFileStep::Remove, &path, error)),
    }
}

/// The sealed bytes of `device_key`'s store file.
pub(crate) fn read_store(data_dir: &Path, device_key: &DevicePublicKey) -> Result<SealedStore, StoreFileError> {
    let path = store_path(data_dir, device_key);
    std::fs::read(&path).map(SealedStore::from_bytes).map_err(|error| StoreFileError::new(StoreFileStep::Read, &path, error))
}

/// The sealed bytes of `device_key`'s Person file.
pub(crate) fn read_person_store(data_dir: &Path, device_key: &DevicePublicKey) -> Result<SealedStore, StoreFileError> {
    let path = person_path(data_dir, device_key);
    std::fs::read(&path).map(SealedStore::from_bytes).map_err(|error| StoreFileError::new(StoreFileStep::Read, &path, error))
}

/// `write_store`'s steps (LLR-b9bm36) for the Person file of `device_key`
/// (LLR-3cysjk).
pub(crate) fn write_person_store(
    data_dir: &Path,
    device_key: &DevicePublicKey,
    sealed: &SealedStore,
    rng: &HandleRng,
) -> Result<(), StoreFileError> {
    replace_file(data_dir, &person_path(data_dir, device_key), sealed.as_bytes(), rng)
}

/// Replace `write`'s Persona's store file with exactly `write.sealed`'s
/// bytes (LLR-b9bm36): a new temporary file `persona-<hex>.store.tmp-<16
/// hex>` in the same directory, its suffix drawn from the handle's source
/// (LLR-4qd5fx; a failed draw refuses as `CreateTemporary` naming the random
/// source and writes nothing, LLR-94nn6q), created with `create_new`,
/// written, synced, renamed over the store file, and on Unix the directory
/// synced. A failure at any later step removes the temporary file where it
/// can and names the step. Crate-visible: the app writes a store only
/// through the handle's operations.
pub(crate) fn write_store(data_dir: &Path, write: &StoreWrite, rng: &HandleRng) -> Result<(), StoreFileError> {
    replace_file(data_dir, &store_path(data_dir, &write.device_key), write.sealed.as_bytes(), rng)
}

/// Replace the file at `path` in `data_dir` with exactly `bytes`, by
/// `write_store`'s steps (LLR-b9bm36).
fn replace_file(data_dir: &Path, path: &Path, bytes: &[u8], rng: &HandleRng) -> Result<(), StoreFileError> {
    let mut suffix = [0u8; 8];
    rng.lock().try_fill_bytes(&mut suffix).map_err(|error| {
        StoreFileError::new(StoreFileStep::CreateTemporary, path, format!("the random source failed: {error}"))
    })?;
    let mut temporary_name = path.as_os_str().to_owned();
    temporary_name.push(format!(".tmp-{}", hex::encode(suffix)));
    let temporary = PathBuf::from(temporary_name);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| StoreFileError::new(StoreFileStep::CreateTemporary, &temporary, error))?;
    let discard = |step: StoreFileStep, error: &dyn fmt::Display| {
        // A failed removal is safe to drop: the step's own error is the one
        // reported, a leftover `.tmp-` file is never read (the store listing
        // takes only names of the form `persona-<hex>.store`), and the next
        // write of this store uses a fresh temporary and renames over the
        // store file.
        let _ = std::fs::remove_file(&temporary);
        StoreFileError::new(step, &temporary, error)
    };
    file.write_all(bytes).map_err(|error| discard(StoreFileStep::Write, &error))?;
    file.sync_all().map_err(|error| discard(StoreFileStep::Sync, &error))?;
    drop(file);
    #[cfg(feature = "test-support")]
    if test_hook::take_fail_after_temporary() {
        return Err(discard(StoreFileStep::Rename, &"failure injected by the test after the temporary file was written"));
    }
    std::fs::rename(&temporary, path).map_err(|error| discard(StoreFileStep::Rename, &error))?;
    #[cfg(feature = "test-support")]
    test_hook::record_write(path, bytes);
    sync_directory(data_dir)
}

/// Make a rename or removal in `data_dir` durable (Unix; elsewhere the
/// platform offers no directory sync through `std`).
fn sync_directory(data_dir: &Path) -> Result<(), StoreFileError> {
    #[cfg(unix)]
    std::fs::File::open(data_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| StoreFileError::new(StoreFileStep::SyncDirectory, data_dir, error))?;
    #[cfg(not(unix))]
    let _ = data_dir;
    Ok(())
}

/// The test-only interruption of `write_store` (`test_support`).
#[cfg(feature = "test-support")]
pub(crate) mod test_hook {
    use std::cell::Cell;

    thread_local! {
        static FAIL_AFTER_TEMPORARY: Cell<bool> = const { Cell::new(false) };
    }

    /// Make this thread's next `write_store` fail after its temporary file
    /// is written and synced, before the rename.
    pub(crate) fn fail_next_write_after_temporary() {
        FAIL_AFTER_TEMPORARY.with(|flag| flag.set(true));
    }

    pub(super) fn take_fail_after_temporary() -> bool {
        FAIL_AFTER_TEMPORARY.with(|flag| flag.replace(false))
    }

    /// Every store file write that landed in this process, by file, in
    /// order.
    static WRITES: std::sync::Mutex<Vec<(std::path::PathBuf, Vec<u8>)>> = std::sync::Mutex::new(Vec::new());

    fn writes() -> std::sync::MutexGuard<'static, Vec<(std::path::PathBuf, Vec<u8>)>> {
        WRITES.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn record_write(path: &std::path::Path, sealed: &[u8]) {
        writes().push((path.to_path_buf(), sealed.to_vec()));
    }

    /// The bytes of every write that landed at `path`, in order.
    pub(crate) fn writes_to(path: &std::path::Path) -> Vec<Vec<u8>> {
        writes().iter().filter(|(written, _)| written == path).map(|(_, sealed)| sealed.clone()).collect()
    }
}
