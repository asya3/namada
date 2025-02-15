//! Cryptographic signature keys

use namada::types::address::Address;
pub use namada::types::key::*;

use super::*;

/// Get the public key associated with the given address from the state prior to
/// tx execution. Returns `Ok(None)` if not found.
pub fn get(ctx: &Ctx, owner: &Address) -> EnvResult<Option<common::PublicKey>> {
    storage_api::key::get(&ctx.pre(), owner)
}
