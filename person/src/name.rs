use alloc::string::String;
use core::fmt;

use unicode_normalization::UnicodeNormalization;

use crate::error::IdentityError;

/// Maximum byte length of a name after NFC normalization.
pub const MAX_NAME_LEN: usize = 128;

/// Maximum byte length of a surname after NFC normalization.
pub const MAX_SURNAME_LEN: usize = 128;

/// The NFC form of `value`, rejected when longer than `max` bytes. LLR-ayu93n.
fn nfc_within_bound(value: &str, field: &'static str, max: usize) -> Result<String, IdentityError> {
    let nfc_form: String = value.nfc().collect();
    if nfc_form.len() > max {
        return Err(IdentityError::FieldTooLong { field, max });
    }
    Ok(nfc_form)
}

macro_rules! string_newtype_impls {
    ($ty:ident) => {
        impl $ty {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $ty {
            type Error = IdentityError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<String> for $ty {
            type Error = IdentityError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(&value)
            }
        }

        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.0
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($ty), "([REDACTED])"))
            }
        }
    };
}

/// A given name: NFC, at most `MAX_NAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Name(String);

impl Name {
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        nfc_within_bound(value, "name", MAX_NAME_LEN).map(Self)
    }
}

string_newtype_impls!(Name);

/// A surname: NFC, at most `MAX_SURNAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Surname(String);

impl Surname {
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        nfc_within_bound(value, "surname", MAX_SURNAME_LEN).map(Self)
    }
}

string_newtype_impls!(Surname);
