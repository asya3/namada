//! Proof of Stake data types

use core::fmt::Debug;
use std::collections::{BTreeSet, HashMap};
use std::convert::TryFrom;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Sub};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use rust_decimal::prelude::{Decimal, ToPrimitive};

use crate::epoched::{
    Epoched, EpochedDelta, OffsetPipelineLen, OffsetUnbondingLen,
};
use crate::parameters::PosParams;

/// Epoched validator's consensus key.
pub type ValidatorConsensusKeys<PublicKey> =
    Epoched<PublicKey, OffsetPipelineLen>;
/// Epoched validator's state.
pub type ValidatorStates = Epoched<ValidatorState, OffsetPipelineLen>;
/// Epoched validator's total deltas.
pub type ValidatorDeltas<TokenChange> =
    EpochedDelta<TokenChange, OffsetUnbondingLen>;

/// Epoched bond.
pub type Bonds<TokenAmount> =
    EpochedDelta<Bond<TokenAmount>, OffsetUnbondingLen>;
/// Epoched unbond.
pub type Unbonds<TokenAmount> =
    EpochedDelta<Unbond<TokenAmount>, OffsetUnbondingLen>;
/// Epoched validator set.
pub type ValidatorSets<Address> =
    Epoched<ValidatorSet<Address>, OffsetUnbondingLen>;
/// Epoched total deltas.
pub type TotalDeltas<TokenChange> =
    EpochedDelta<TokenChange, OffsetUnbondingLen>;
/// Epoched validator commission rate
pub type CommissionRates = Epoched<Decimal, OffsetPipelineLen>;

/// Epoch identifier. Epochs are identified by consecutive natural numbers.
///
/// In the API functions, this type is wrapped in [`Into`]. When using this
/// library, to replace [`Epoch`] with a custom type, simply implement [`From`]
/// to and from the types here.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
)]
pub struct Epoch(u64);

/// A genesis validator definition.
#[derive(
    Debug,
    Clone,
    BorshSerialize,
    BorshSchema,
    BorshDeserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct GenesisValidator<Address, Token, PK> {
    /// Validator's address
    pub address: Address,
    /// Staked tokens are put into a self-bond
    pub tokens: Token,
    /// A public key used for signing validator's consensus actions
    pub consensus_key: PK,
    /// Commission rate charged on rewards for delegators (bounded inside 0-1)
    pub commission_rate: Decimal,
    /// Maximum change in commission rate permitted per epoch
    pub max_commission_rate_change: Decimal,
}

/// An update of the active and inactive validator set.
#[derive(Debug, Clone)]
pub enum ValidatorSetUpdate<PK> {
    /// A validator is active
    Active(ActiveValidator<PK>),
    /// A validator who was active in the last update and is now inactive
    Deactivated(PK),
}

/// Active validator's consensus key and its bonded stake.
#[derive(Debug, Clone)]
pub struct ActiveValidator<PK> {
    /// A public key used for signing validator's consensus actions
    pub consensus_key: PK,
    /// Total bonded stake of the validator
    pub bonded_stake: u64,
}

/// ID of a bond and/or an unbond.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
)]
pub struct BondId<Address>
where
    Address: Display
        + Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + BorshSerialize
        + BorshSchema
        + BorshDeserialize,
{
    /// (Un)bond's source address is the owner of the bonded tokens.
    pub source: Address,
    /// (Un)bond's validator address.
    pub validator: Address,
}

/// Validator's address with its voting power.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
)]
pub struct WeightedValidator<Address>
where
    Address: Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + BorshDeserialize
        + BorshSchema
        + BorshSerialize,
{
    /// The `total_stake` field must be on top, because lexicographic ordering
    /// is based on the top-to-bottom declaration order and in the
    /// `ValidatorSet` the `WeightedValidator`s these need to be sorted by
    /// the `total_stake`.
    pub bonded_stake: u64,
    /// Validator's address
    pub address: Address,
}

impl<Address> Display for WeightedValidator<Address>
where
    Address: Display
        + Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + BorshDeserialize
        + BorshSchema
        + BorshSerialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} with bonded stake {}",
            self.address, self.bonded_stake
        )
    }
}

/// Active and inactive validator sets.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
)]
pub struct ValidatorSet<Address>
where
    Address: Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + BorshDeserialize
        + BorshSchema
        + BorshSerialize,
{
    /// Active validator set with maximum size equal to `max_validator_slots`
    /// in [`PosParams`].
    pub active: BTreeSet<WeightedValidator<Address>>,
    /// All the other validators that are not active
    pub inactive: BTreeSet<WeightedValidator<Address>>,
}

/// Validator's state.
#[derive(
    Debug,
    Clone,
    Copy,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
    PartialEq,
    Eq,
)]
pub enum ValidatorState {
    /// Inactive validator may not participate in the consensus.
    Inactive,
    /// A `Pending` validator will become `Candidate` in a future epoch.
    Pending,
    /// A `Candidate` validator may participate in the consensus. It is either
    /// in the active or inactive validator set.
    Candidate,
    // TODO consider adding `Jailed`
}

/// A bond is either a validator's self-bond or a delegation from a regular
/// account to a validator.
#[derive(
    Debug, Clone, Default, BorshDeserialize, BorshSerialize, BorshSchema,
)]
pub struct Bond<Token: Default> {
    /// Bonded positive deltas. A key is the epoch set for the bond. This is
    /// used in unbonding, where it's needed for slash epoch range check.
    ///
    /// TODO: For Bonds, there's unnecessary redundancy with this hash map.
    /// We only need to keep the start `Epoch` for the Epoched head element
    /// (i.e. the current epoch data), the rest of the array can be calculated
    /// from the offset from the head
    pub pos_deltas: HashMap<Epoch, Token>,
    /// Unbonded negative deltas. The values are recorded as positive, but
    /// should be subtracted when we're finding the total for some given
    /// epoch.
    pub neg_deltas: Token,
}

/// An unbond contains unbonded tokens from a validator's self-bond or a
/// delegation from a regular account to a validator.
#[derive(
    Debug, Clone, Default, BorshDeserialize, BorshSerialize, BorshSchema,
)]
pub struct Unbond<Token: Default> {
    /// A key is a pair of the epoch of the bond from which a unbond was
    /// created the epoch of unbonding. This is needed for slash epoch range
    /// check.
    pub deltas: HashMap<(Epoch, Epoch), Token>,
}

/// A slash applied to validator, to punish byzantine behavior by removing
/// their staked tokens at and before the epoch of the slash.
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct Slash {
    /// Epoch at which the slashable event occurred.
    pub epoch: Epoch,
    /// Block height at which the slashable event occurred.
    pub block_height: u64,
    /// A type of slashsable event.
    pub r#type: SlashType,
    /// A rate is the portion of staked tokens that are slashed.
    pub rate: Decimal,
}

/// Slashes applied to validator, to punish byzantine behavior by removing
/// their staked tokens at and before the epoch of the slash.
pub type Slashes = Vec<Slash>;

/// A type of slashsable event.
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum SlashType {
    /// Duplicate block vote.
    DuplicateVote,
    /// Light client attack.
    LightClientAttack,
}

/// Derive Tendermint raw hash from the public key
pub trait PublicKeyTmRawHash {
    /// Derive Tendermint raw hash from the public key
    fn tm_raw_hash(&self) -> String;
}

impl Epoch {
    /// Iterate a range of consecutive epochs starting from `self` of a given
    /// length. Work-around for `Step` implementation pending on stabilization of <https://github.com/rust-lang/rust/issues/42168>.
    pub fn iter_range(self, len: u64) -> impl Iterator<Item = Epoch> + Clone {
        let start_ix: u64 = self.into();
        let end_ix: u64 = start_ix + len;
        (start_ix..end_ix).map(Epoch::from)
    }

    /// Checked epoch subtraction. Computes self - rhs, returning None if
    /// overflow occurred.
    #[must_use = "this returns the result of the operation, without modifying \
                  the original"]
    pub fn checked_sub(self, rhs: Epoch) -> Option<Self> {
        if rhs.0 > self.0 {
            None
        } else {
            Some(Self(self.0 - rhs.0))
        }
    }

    /// Checked epoch subtraction. Computes self - rhs, returning default
    /// `Epoch(0)` if overflow occurred.
    #[must_use = "this returns the result of the operation, without modifying \
                  the original"]
    pub fn sub_or_default(self, rhs: Epoch) -> Self {
        self.checked_sub(rhs).unwrap_or_default()
    }
}

impl From<u64> for Epoch {
    fn from(epoch: u64) -> Self {
        Epoch(epoch)
    }
}

impl From<Epoch> for u64 {
    fn from(epoch: Epoch) -> Self {
        epoch.0
    }
}

impl From<Epoch> for usize {
    fn from(epoch: Epoch) -> Self {
        epoch.0 as usize
    }
}

impl Add<u64> for Epoch {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Epoch(self.0 + rhs)
    }
}

impl Add<usize> for Epoch {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Epoch(self.0 + rhs as u64)
    }
}

impl Sub<u64> for Epoch {
    type Output = Epoch;

    fn sub(self, rhs: u64) -> Self::Output {
        Epoch(self.0 - rhs)
    }
}

impl Sub<Epoch> for Epoch {
    type Output = Self;

    fn sub(self, rhs: Epoch) -> Self::Output {
        Epoch(self.0 - rhs.0)
    }
}

impl Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<Address> Display for BondId<Address>
where
    Address: Display
        + Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + BorshSerialize
        + BorshDeserialize
        + BorshSchema,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{source: {}, validator: {}}}",
            self.source, self.validator
        )
    }
}

impl<Token> Bond<Token>
where
    Token: Clone + Copy + Add<Output = Token> + Sub<Output = Token> + Default,
{
    /// Find the sum of all the bonds amounts.
    pub fn sum(&self) -> Token {
        let pos_deltas_sum: Token = self
            .pos_deltas
            .iter()
            .fold(Default::default(), |acc, (_epoch, amount)| acc + *amount);
        pos_deltas_sum - self.neg_deltas
    }
}

impl<Token> Add for Bond<Token>
where
    Token: Clone + AddAssign + Default,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        // This is almost the same as `self.pos_deltas.extend(rhs.pos_deltas);`,
        // except that we add values where a key is present on both
        // sides.
        let iter = rhs.pos_deltas.into_iter();
        let reserve = if self.pos_deltas.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.pos_deltas.reserve(reserve);
        iter.for_each(|(k, v)| {
            // Add or insert
            match self.pos_deltas.get_mut(&k) {
                Some(value) => *value += v,
                None => {
                    self.pos_deltas.insert(k, v);
                }
            }
        });
        self.neg_deltas += rhs.neg_deltas;
        self
    }
}

impl<Token> Unbond<Token>
where
    Token: Clone + Copy + Add<Output = Token> + Default,
{
    /// Find the sum of all the unbonds amounts.
    pub fn sum(&self) -> Token {
        self.deltas
            .iter()
            .fold(Default::default(), |acc, (_epoch, amount)| acc + *amount)
    }
}

impl<Token> Add for Unbond<Token>
where
    Token: Clone + AddAssign + Default,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        // This is almost the same as `self.deltas.extend(rhs.deltas);`, except
        // that we add values where a key is present on both sides.
        let iter = rhs.deltas.into_iter();
        let reserve = if self.deltas.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.deltas.reserve(reserve);
        iter.for_each(|(k, v)| {
            // Add or insert
            match self.deltas.get_mut(&k) {
                Some(value) => *value += v,
                None => {
                    self.deltas.insert(k, v);
                }
            }
        });
        self
    }
}

impl SlashType {
    /// Get the slash rate applicable to the given slash type from the PoS
    /// parameters.
    pub fn get_slash_rate(&self, params: &PosParams) -> Decimal {
        match self {
            SlashType::DuplicateVote => params.duplicate_vote_min_slash_rate,
            SlashType::LightClientAttack => {
                params.light_client_attack_min_slash_rate
            }
        }
    }
}

impl Display for SlashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlashType::DuplicateVote => write!(f, "Duplicate vote"),
            SlashType::LightClientAttack => write!(f, "Light client attack"),
        }
    }
}

/// Multiply a value of type Decimal with one of type u64 and then return the
/// truncated u64
pub fn decimal_mult_u64(dec: Decimal, int: u64) -> u64 {
    let prod = dec * Decimal::from(int);
    // truncate the number to the floor
    prod.to_u64().expect("Product is out of bounds")
}

/// Multiply a value of type Decimal with one of type i128 and then return the
/// truncated i128
pub fn decimal_mult_i128(dec: Decimal, int: i128) -> i128 {
    let prod = dec * Decimal::from(int);
    // truncate the number to the floor
    prod.to_i128().expect("Product is out of bounds")
}

/// Calculate voting power in the tendermint context (which is stored as i64)
/// from the number of tokens
pub fn into_tm_voting_power(
    votes_per_token: Decimal,
    tokens: impl Into<u64>,
) -> i64 {
    let prod = decimal_mult_u64(votes_per_token, tokens.into());
    i64::try_from(prod).expect("Invalid voting power")
}

#[cfg(test)]
pub mod tests {

    use std::ops::Range;

    use proptest::prelude::*;

    use super::*;

    /// Generate arbitrary epoch in given range
    pub fn arb_epoch(range: Range<u64>) -> impl Strategy<Value = Epoch> {
        range.prop_map(Epoch)
    }
}
