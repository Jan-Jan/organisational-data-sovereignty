//! The Person store (SDD-27p679): the user's own Person, kept apart from
//! every Persona — a `person::Person` and the Person's own DevicePublicKey
//! and PersonPublicKey — sealed with the Person's own Device seed exactly as
//! a Persona store is (LLR-bg538c's AEAD and nonce layout), under its own
//! HKDF label, so a Persona store's bytes never open as a Person store or
//! the reverse (LLR-73sgz2). org-node names no path and touches no file.
use person::{DeviceSlots, Person};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use org_members::{DevicePublicKey, Name, PersonPublicKey, Surname};

use crate::error::{OrgNodeError, StoreRefusal};
use crate::keys::DeviceSecret;
use crate::store::{labelled_cipher, open_sealed, parse_field, seal_bytes, SealedStore, StoreCipher};
use crate::types::MemberSeed;

/// The HKDF info string of the Person store key (LLR-73sgz2): not the
/// Persona store's `ods/persona-store/v1`.
const PERSON_STORE_KEY_INFO: &[u8] = b"ods/person-store/v1";

/// The Person, its two public keys and the Person key pair's secret seed
/// (LLR-wqu26s): no Handle. The seed is crate-private and redacted under
/// `Debug`, as a Persona record's member seed is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonStoreContent {
    pub person: Person,
    pub device_key: DevicePublicKey,
    pub person_key: PersonPublicKey,
    pub(crate) person_seed: MemberSeed,
}

/// The Person store's public values, as `PersonStore::view` copies them
/// (LLR-8ru6mv): `PersonStoreContent` without the Person seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonView {
    pub person: Person,
    pub device_key: DevicePublicKey,
    pub person_key: PersonPublicKey,
}

#[cfg(feature = "test-support")]
impl PersonStoreContent {
    /// The kept Person seed (test builds only).
    pub fn person_seed_for_test(&self) -> &MemberSeed {
        &self.person_seed
    }
}

/// `PersonStoreContent` as the store encodes it: the Person's name and surname, its
/// two keys, then the Person seed. The Person value is rebuilt on unseal.
#[derive(Serialize)]
struct PersonStoreContentEncoding<'a> {
    name: &'a Name,
    surname: &'a Surname,
    device_key: &'a DevicePublicKey,
    person_key: &'a PersonPublicKey,
    person_seed: &'a MemberSeed,
}

/// `PersonStoreContent` as decoded, before parsing (`person.*`, LLR-73sgz2).
#[derive(Deserialize)]
struct RawPersonStoreContent {
    name: String,
    surname: String,
    device_key: [u8; 32],
    person_key: [u8; 32],
    person_seed: MemberSeed,
}

/// The Person built from parsed values: one device, its DevicePublicKey,
/// and the PersonPublicKey (`person`'s rules for it).
fn person_of(name: Name, surname: Surname, device_key: DevicePublicKey, person_key: PersonPublicKey) -> Result<Person, OrgNodeError> {
    let devices = parse_field("person", DeviceSlots::parse(vec![device_key]))?;
    parse_field("person", Person::new(name, surname, Some(person_key), devices))
}

impl TryFrom<RawPersonStoreContent> for PersonStoreContent {
    type Error = OrgNodeError;

    fn try_from(raw: RawPersonStoreContent) -> Result<Self, Self::Error> {
        let name = parse_field("person.name", Name::parse(&raw.name))?;
        let surname = parse_field("person.surname", Surname::parse(&raw.surname))?;
        let device_key = parse_field("person.device_key", DevicePublicKey::parse(&raw.device_key))?;
        let person_key = parse_field("person.person_key", PersonPublicKey::parse(&raw.person_key))?;
        Ok(Self { person: person_of(name, surname, device_key, person_key)?, device_key, person_key, person_seed: raw.person_seed })
    }
}

/// The user's open Person store: the store key, held only as the cipher,
/// and the Person store's content. No path: org-node touches no file.
pub struct PersonStore {
    key: StoreCipher,
    content: PersonStoreContent,
}

/// The cipher of the Person store key: LLR-2qqqx6's derivation under
/// `PERSON_STORE_KEY_INFO` (LLR-73sgz2).
pub(crate) fn cipher(secret: &DeviceSecret, device_key: &DevicePublicKey) -> Result<StoreCipher, OrgNodeError> {
    labelled_cipher(secret, device_key, PERSON_STORE_KEY_INFO)
}

impl PersonStore {
    /// A new Person store (LLR-wqu26s): the name and surname parsed
    /// (`InvalidField` `person.name` / `person.surname`, before any draw,
    /// creating nothing), then a fresh Device seed and a Person key pair drawn
    /// from `rng`; returns the store, the Device seed by value for org-io's
    /// keychain (org-node keeps no copy) and the store sealed.
    pub fn create<R: RngCore + CryptoRng>(
        rng: &mut R,
        name: &str,
        surname: &str,
    ) -> Result<(PersonStore, DeviceSecret, SealedStore), OrgNodeError> {
        let name = parse_field("person.name", Name::parse(name))?;
        let surname = parse_field("person.surname", Surname::parse(surname))?;
        let secret = DeviceSecret::generate(rng);
        let mut seed = Zeroizing::new([0u8; 32]);
        rng.fill_bytes(seed.as_mut());
        let person_seed = MemberSeed::from(*seed);
        let person_key = person_seed.x25519_keypair().member_key()?;
        let device_key = secret.device_key();
        let content =
            PersonStoreContent { person: person_of(name, surname, device_key, person_key)?, device_key, person_key, person_seed };
        let store = PersonStore { key: cipher(&secret, &device_key)?, content };
        let sealed = store.seal(rng)?;
        Ok((store, secret, sealed))
    }

    /// This store, sealed with a fresh 24-byte nonce (LLR-73sgz2).
    pub fn seal<R: RngCore + CryptoRng>(&self, rng: &mut R) -> Result<SealedStore, OrgNodeError> {
        let plaintext = Zeroizing::new(
            postcard::to_allocvec(&PersonStoreContentEncoding {
                name: self.content.person.name(),
                surname: self.content.person.surname(),
                device_key: &self.content.device_key,
                person_key: &self.content.person_key,
                person_seed: &self.content.person_seed,
            })
            .map_err(|error| OrgNodeError::Chain(format!("person store encode: {error}")))?,
        );
        seal_bytes(&self.key, &plaintext, rng)
    }

    /// Open the sealed Person store of `device_key` with its secret, which is
    /// consumed, refusing as `PersonaStore::unseal` refuses (LLR-73sgz2): a
    /// secret of another Device (`DeviceSecretMismatch`), too few bytes
    /// (`TooShort`), a ciphertext that does not authenticate under the Person
    /// store key (`NotAuthentic`: a Persona store's bytes among them), a
    /// plaintext that does not decode (`Undecodable`) or holds a value its
    /// type refuses (`InvalidField`, `person.*`), and a store naming another
    /// Device (`DeviceSecretMismatch`).
    pub fn unseal(sealed: &SealedStore, device_key: DevicePublicKey, secret: DeviceSecret) -> Result<Self, OrgNodeError> {
        let (key, plaintext) = open_sealed(sealed, &device_key, secret, PERSON_STORE_KEY_INFO)?;
        let raw: RawPersonStoreContent =
            postcard::from_bytes(&plaintext).map_err(|_| OrgNodeError::StoreRefused(StoreRefusal::Undecodable))?;
        let content = PersonStoreContent::try_from(raw)?;
        if content.device_key != device_key {
            return Err(OrgNodeError::DeviceSecretMismatch);
        }
        Ok(Self { key, content })
    }

    /// The Person's DevicePublicKey.
    pub fn device_key(&self) -> DevicePublicKey {
        self.content.device_key
    }

    /// The Person store's content: the Person, its Device key, its Person
    /// key and its Person seed.
    pub fn content(&self) -> &PersonStoreContent {
        &self.content
    }

    /// A copy of the Person store's public values (LLR-8ru6mv): the Person
    /// and its two public keys, never the Person seed.
    pub fn view(&self) -> PersonView {
        PersonView {
            person: self.content.person.clone(),
            device_key: self.content.device_key,
            person_key: self.content.person_key,
        }
    }
}
