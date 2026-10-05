//! An individual's identity types: names, device keys, the PersonPublicKey a
//! grant is encoded against, device slots and the device sub-trie.
//! org-members uses them; the Person definition is built on them.

#![no_std]
#![deny(clippy::indexing_slicing)]

extern crate alloc;

pub mod device_key;
pub mod device_trie;
pub mod error;
pub mod name;
pub mod person_key;
pub mod slots;
pub mod x25519;

pub use device_key::DevicePublicKey;
pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};
pub use error::IdentityError;
pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
pub use person_key::PersonPublicKey;
pub use slots::{DeviceSlots, MAX_DEVICES};

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
