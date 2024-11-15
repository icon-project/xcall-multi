use soroban_sdk::{token, Address, Env};

use crate::{error::ContractError, storage};

pub fn ensure_admin(e: &Env) -> Result<Address, ContractError> {
    let admin = storage::admin(&e)?;
    admin.require_auth();

    Ok(admin)
}

pub fn ensure_upgrade_authority(e: &Env) -> Result<Address, ContractError> {
    let authority = storage::get_upgrade_authority(&e)?;
    authority.require_auth();

    Ok(authority)
}

pub fn transfer_token(env: &Env, token: &Address, from: &Address, to: &Address, amount: u128) {
    let token_client = token::Client::new(&env, &token);
    token_client.transfer(&from, &to, &(amount as i128));
}
