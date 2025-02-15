//! RPC endpoint is used for ledger state queries

use std::fmt::Display;
use std::str::FromStr;

use masp_primitives::asset_type::AssetType;
use namada::types::address::Address;
use namada::types::storage;
use namada::types::token::CONVERSION_KEY_PREFIX;
use thiserror::Error;

use crate::facade::tendermint::abci::Path as AbciPath;

/// RPC query path
#[derive(Debug, Clone)]
pub enum Path {
    /// Dry run a transaction
    DryRunTx,
    /// Epoch of the last committed block
    Epoch,
    /// Results of all committed blocks
    Results,
    /// Read a storage value with exact storage key
    Value(storage::Key),
    /// Read a range of storage values with a matching key prefix
    Prefix(storage::Key),
    /// Check if the given storage key exists
    HasKey(storage::Key),
    /// Conversion associated with given asset type
    Conversion(AssetType),
}

#[derive(Debug, Clone)]
pub struct BalanceQuery {
    #[allow(dead_code)]
    owner: Option<Address>,
    #[allow(dead_code)]
    token: Option<Address>,
}

const DRY_RUN_TX_PATH: &str = "dry_run_tx";
const EPOCH_PATH: &str = "epoch";
const RESULTS_PATH: &str = "results";
const VALUE_PREFIX: &str = "value";
const PREFIX_PREFIX: &str = "prefix";
const HAS_KEY_PREFIX: &str = "has_key";

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::DryRunTx => write!(f, "{}", DRY_RUN_TX_PATH),
            Path::Epoch => write!(f, "{}", EPOCH_PATH),
            Path::Results => write!(f, "{}", RESULTS_PATH),
            Path::Value(storage_key) => {
                write!(f, "{}/{}", VALUE_PREFIX, storage_key)
            }
            Path::Prefix(storage_key) => {
                write!(f, "{}/{}", PREFIX_PREFIX, storage_key)
            }
            Path::HasKey(storage_key) => {
                write!(f, "{}/{}", HAS_KEY_PREFIX, storage_key)
            }
            Path::Conversion(asset_type) => {
                write!(f, "{}/{}", CONVERSION_KEY_PREFIX, asset_type)
            }
        }
    }
}

impl FromStr for Path {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            DRY_RUN_TX_PATH => Ok(Self::DryRunTx),
            EPOCH_PATH => Ok(Self::Epoch),
            RESULTS_PATH => Ok(Self::Results),
            _ => match s.split_once('/') {
                Some((VALUE_PREFIX, storage_key)) => {
                    let key = storage::Key::parse(storage_key)
                        .map_err(PathParseError::InvalidStorageKey)?;
                    Ok(Self::Value(key))
                }
                Some((PREFIX_PREFIX, storage_key)) => {
                    let key = storage::Key::parse(storage_key)
                        .map_err(PathParseError::InvalidStorageKey)?;
                    Ok(Self::Prefix(key))
                }
                Some((HAS_KEY_PREFIX, storage_key)) => {
                    let key = storage::Key::parse(storage_key)
                        .map_err(PathParseError::InvalidStorageKey)?;
                    Ok(Self::HasKey(key))
                }
                Some((CONVERSION_KEY_PREFIX, asset_type)) => {
                    let key = AssetType::from_str(asset_type)
                        .map_err(PathParseError::InvalidAssetType)?;
                    Ok(Self::Conversion(key))
                }
                _ => Err(PathParseError::InvalidPath(s.to_string())),
            },
        }
    }
}

impl From<Path> for AbciPath {
    fn from(path: Path) -> Self {
        let path = path.to_string();
        // TODO: update in tendermint-rs to allow to construct this from owned
        // string. It's what `from_str` does anyway
        AbciPath::from_str(&path).unwrap()
    }
}

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum PathParseError {
    #[error("Unrecognized query path: {0}")]
    InvalidPath(String),
    #[error("Invalid storage key: {0}")]
    InvalidStorageKey(storage::Error),
    #[error("Unrecognized asset type: {0}")]
    InvalidAssetType(std::io::Error),
}
