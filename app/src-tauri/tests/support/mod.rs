//! Fixtures shared by the app's test targets. Not a test target: each target
//! that uses it declares `mod support;`.

use org_io::node::{DeviceSeed, MemberSeed};

/// A member key and a device key that parse, as an Invite reply carries them.
pub fn reply_keys() -> ([u8; 32], [u8; 32]) {
    let member = MemberSeed::from([8; 32]).x25519_keypair().member_key().expect("member key");
    let device = DeviceSeed::from([7; 32]).signing_keypair().device_key().expect("device key");
    (*member.as_bytes(), *device.as_bytes())
}
