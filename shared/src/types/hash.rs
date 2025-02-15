//! Types for working with 32 bytes hashes.

use std::fmt::{self, Display};
use std::ops::Deref;
use std::str::FromStr;

use arse_merkle_tree::traits::Value;
use arse_merkle_tree::Hash as TreeHash;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::tendermint::abci::transaction;
use crate::tendermint::Hash as TmHash;

/// The length of the raw transaction hash.
pub const HASH_LENGTH: usize = 32;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("TEMPORARY error: {error}")]
    Temporary { error: String },
    #[error("Failed trying to convert slice to a hash: {0}")]
    ConversionFailed(std::array::TryFromSliceError),
    #[error("Failed to convert string into a hash: {0}")]
    FromStringError(hex::FromHexError),
}

/// Result for functions that may fail
pub type HashResult<T> = std::result::Result<T, Error>;

#[derive(
    Clone,
    Debug,
    Default,
    Hash,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
    Serialize,
    Deserialize,
)]
/// A hash, typically a sha-2 hash of a tx
pub struct Hash(pub [u8; 32]);

impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = self::Error;

    fn try_from(value: &[u8]) -> HashResult<Self> {
        if value.len() != HASH_LENGTH {
            return Err(Error::Temporary {
                error: format!(
                    "Unexpected tx hash length {}, expected {}",
                    value.len(),
                    HASH_LENGTH
                ),
            });
        }
        let hash: [u8; 32] =
            TryFrom::try_from(value).map_err(Error::ConversionFailed)?;
        Ok(Hash(hash))
    }
}

impl TryFrom<String> for Hash {
    type Error = self::Error;

    fn try_from(string: String) -> HashResult<Self> {
        string.as_str().try_into()
    }
}

impl TryFrom<&str> for Hash {
    type Error = self::Error;

    fn try_from(string: &str) -> HashResult<Self> {
        Ok(Self(
            <[u8; HASH_LENGTH]>::from_hex(string)
                .map_err(Error::FromStringError)?,
        ))
    }
}

impl From<Hash> for transaction::Hash {
    fn from(hash: Hash) -> Self {
        Self::new(hash.0)
    }
}

impl FromStr for Hash {
    type Err = self::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Self::try_from(str)
    }
}

impl Hash {
    /// Compute sha256 of some bytes
    pub fn sha256(data: impl AsRef<[u8]>) -> Self {
        let digest = Sha256::digest(data.as_ref());
        Self(*digest.as_ref())
    }

    /// Check if the hash is all zeros
    pub fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl From<Hash> for TmHash {
    fn from(hash: Hash) -> Self {
        TmHash::Sha256(hash.0)
    }
}

impl From<Hash> for TreeHash {
    fn from(hash: Hash) -> Self {
        Self::from(hash.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::string::{string_regex, RegexGeneratorStrategy};

    use super::*;

    /// Returns a proptest strategy that yields hex encoded hashes.
    fn hex_encoded_hash_strat() -> RegexGeneratorStrategy<String> {
        string_regex(r"[a-fA-F0-9]{64}").unwrap()
    }

    proptest! {
        #[test]
        fn test_hash_string(hex_hash in hex_encoded_hash_strat()) {
            let _: Hash = hex_hash.try_into().unwrap();
        }
    }
}
