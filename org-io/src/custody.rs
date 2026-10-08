//! Custody of the user's own sr25519 signatory key (SDD-789u6d): the
//! development configuration's parse, the key holder and the development-only
//! seed read. Errors name the variable and the rule and carry no part of the
//! value (LLR-gc6kwy).

use core::fmt;

use on_chain_client::write::AccountId;
use subxt_signer::sr25519::Keypair;
use zeroize::Zeroize;
#[cfg(feature = "dev-seed")]
use zeroize::Zeroizing;

/// The environment variable a configuration value came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigVariable {
    AdminSeed,
    CoSigner,
}

impl ConfigVariable {
    fn name(self) -> &'static str {
        match self {
            ConfigVariable::AdminSeed => "ODS_ADMIN_SEED",
            ConfigVariable::CoSigner => "ODS_COSIGNER_PUB",
        }
    }
}

/// The rule a configuration value broke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigRule {
    Empty,
    NotHex,
    WrongLength,
    NotAKey,
}

/// A refused configuration value: which variable, which rule. Holds nothing
/// taken from the value (LLR-gc6kwy).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub variable: ConfigVariable,
    pub rule: ConfigRule,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rule = match self.rule {
            ConfigRule::Empty => "is empty",
            ConfigRule::NotHex | ConfigRule::WrongLength => {
                "must be 64 hexadecimal characters after at most one leading 0x"
            }
            ConfigRule::NotAKey => "is not a valid sr25519 secret seed",
        };
        write!(formatter, "{} {rule}", self.variable.name())
    }
}

impl std::error::Error for ConfigError {}

/// The 32 seed bytes, wiped on drop; no public accessor (LLR-c4bktx). Held
/// on the heap and written there in place by the parse, so moving a
/// `SeedBytes` copies a pointer, never the seed.
pub struct SeedBytes(Box<[u8; 32]>);

impl SeedBytes {
    /// Whether two seeds hold the same bytes (test builds only: a
    /// non-constant-time comparison of the secret).
    #[cfg(feature = "test-support")]
    pub fn same_bytes_as(&self, other: &SeedBytes) -> bool {
        self.0 == other.0
    }

    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for SeedBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for SeedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SeedBytes(..)")
    }
}

/// 32 bytes from `raw`, written into `destination` in place: at most one
/// leading `0x`, then exactly 64 hex characters. Rules, in order: empty
/// body → `Empty`; a body that is not 64 bytes long → `WrongLength` (a
/// doubled `0x` leaves 66); a 64-byte body with any byte outside
/// `0-9a-fA-F` (a non-ASCII character included) → `NotHex`. On a refusal
/// `destination` may hold some parsed bytes; the seed's is wiped on drop.
fn parse_32_hex_into(raw: &str, variable: ConfigVariable, destination: &mut [u8; 32]) -> Result<(), ConfigError> {
    let refuse = |rule| ConfigError { variable, rule };
    let body = raw.strip_prefix("0x").unwrap_or(raw).as_bytes();
    if body.is_empty() {
        return Err(refuse(ConfigRule::Empty));
    }
    if body.len() != 64 {
        return Err(refuse(ConfigRule::WrongLength));
    }
    for (slot, pair) in destination.iter_mut().zip(body.chunks_exact(2)) {
        let (Some(&high), Some(&low)) = (pair.first(), pair.get(1)) else {
            return Err(refuse(ConfigRule::WrongLength));
        };
        let high = hex_value(high).ok_or(refuse(ConfigRule::NotHex))?;
        let low = hex_value(low).ok_or(refuse(ConfigRule::NotHex))?;
        *slot = high << 4 | low;
    }
    Ok(())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// The development signing seed (LLR-4ax2m6).
pub fn parse_seed(raw: &str) -> Result<SeedBytes, ConfigError> {
    let mut seed = SeedBytes(Box::default());
    parse_32_hex_into(raw, ConfigVariable::AdminSeed, &mut seed.0)?;
    Ok(seed)
}

/// One co-signer account (LLR-9fy622).
pub fn parse_co_signer(raw: &str) -> Result<AccountId, ConfigError> {
    let mut account = AccountId(Default::default());
    parse_32_hex_into(raw, ConfigVariable::CoSigner, &mut account.0)?;
    Ok(account)
}

/// The user's own sr25519 signatory key (LLR-c4bktx). The key pair is
/// private; the writer reaches it through `keypair`, crate-private.
pub struct SignatoryKey {
    keypair: Keypair,
}

impl SignatoryKey {
    /// Build the key from its seed; the seed is consumed and wiped. The
    /// bytes go straight from `SeedBytes` into subxt-signer's
    /// `from_secret_key`, which takes them by value: that argument is the one
    /// copy org-io cannot wipe (SOUP, org-io/docs/architecture/soup.md).
    pub fn from_seed(seed: SeedBytes) -> Result<Self, ConfigError> {
        Keypair::from_secret_key(*seed.bytes())
            .map(|keypair| Self { keypair })
            .map_err(|_| ConfigError {
                variable: ConfigVariable::AdminSeed,
                rule: ConfigRule::NotAKey,
            })
    }

    /// The public account: the key's sr25519 public key.
    pub fn account_id(&self) -> AccountId {
        AccountId(self.keypair.public_key().0)
    }

    pub(crate) fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl fmt::Debug for SignatoryKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let public = self.keypair.public_key().0;
        write!(
            formatter,
            "SignatoryKey(account {:02x}{:02x}{:02x}{:02x}..)",
            public[0], public[1], public[2], public[3]
        )
    }
}

/// The one read of `ODS_ADMIN_SEED`, development builds only (LLR-rgdx22,
/// REQ-8zuka3). `Ok(None)` when the variable is unset; a value that is set
/// but not UTF-8 is a malformed seed (`NotHex`). The value read is wiped
/// when dropped.
#[cfg(feature = "dev-seed")]
pub fn signatory_from_environment() -> Result<Option<SignatoryKey>, ConfigError> {
    match std::env::var("ODS_ADMIN_SEED") {
        Ok(raw) => {
            let raw = Zeroizing::new(raw);
            parse_seed(&raw).and_then(SignatoryKey::from_seed).map(Some)
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(value)) => {
            wipe_os_string(value);
            Err(ConfigError { variable: ConfigVariable::AdminSeed, rule: ConfigRule::NotHex })
        }
    }
}

/// Wipe a value read from the environment that was not UTF-8 (on Unix,
/// where its bytes are reachable; elsewhere it is dropped).
#[cfg(feature = "dev-seed")]
fn wipe_os_string(value: std::ffi::OsString) {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        drop(Zeroizing::new(value.into_vec()));
    }
    #[cfg(not(unix))]
    drop(value);
}
