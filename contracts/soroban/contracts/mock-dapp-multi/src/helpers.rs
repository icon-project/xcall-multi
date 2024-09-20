use soroban_sdk::{Address, Env};

use crate::{errors::ContractError, storage};

pub fn ensure_admin(e: &Env) -> Result<Address, ContractError> {
    let admin = storage::admin(&e)?;
    admin.require_auth();

    Ok(admin)
}

pub fn ensure_xcall(e: &Env) -> Result<Address, ContractError> {
    let xcall = storage::get_xcall_address(&e)?;
    xcall.require_auth();

    Ok(xcall)
}
