//! An individual's identity types: names, device keys, the PersonPublicKey a
//! grant is encoded against, device slots and the device sub-trie.
//! org-members uses them. On them this crate builds the Person definition, the
//! check of one definition against its predecessor, and the Person hash.

#![no_std]
#![deny(clippy::indexing_slicing)]

extern crate alloc;

pub mod definition;
#[cfg(feature = "serde")]
mod definition_serde;
pub mod device_hasher;
pub mod device_key;
pub mod device_trie;
pub mod encoding;
pub mod error;
pub mod group_key;
pub mod hash;
pub mod name;
pub mod person_key;
pub mod slots;
pub mod successor;
pub mod x25519;

pub use definition::Person;
pub use device_hasher::PersonDeviceHasher;
pub use device_key::DevicePublicKey;
pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};
pub use encoding::EncodingVersion;
pub use error::IdentityError;
pub use group_key::check_group_key;
pub use hash::{person_hash, verify, PersonHash};
pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
pub use person_key::PersonPublicKey;
pub use slots::{DeviceSlots, MAX_DEVICES};
pub use successor::check_successor;

/// Writes `type_name(aabbccdd..)`: the type name and the first four of the 32
/// bytes in hex. The Debug form of the crate's 32-byte key and hash types.
fn write_hex_prefix(
    formatter: &mut core::fmt::Formatter<'_>,
    type_name: &str,
    bytes: &[u8; 32],
) -> core::fmt::Result {
    let [byte0, byte1, byte2, byte3, ..] = *bytes;
    write!(
        formatter,
        "{type_name}({byte0:02x}{byte1:02x}{byte2:02x}{byte3:02x}..)"
    )
}
