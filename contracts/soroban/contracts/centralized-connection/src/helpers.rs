use soroban_sdk::{token, Address, Bytes, Env, String};

use crate::{errors::ContractError, interfaces::interface_xcall::XcallClient, storage};

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

pub fn ensure_xcall(e: &Env) -> Result<Address, ContractError> {
    let xcall = storage::get_xcall(&e)?;
    xcall.require_auth();

    Ok(xcall)
}

pub fn get_network_fee(
    env: &Env,
    network_id: String,
    response: bool,
) -> Result<u128, ContractError> {
    let mut fee = storage::get_msg_fee(&env, network_id.clone())?;
    if response {
        fee += storage::get_res_fee(&env, network_id)?;
    }

    Ok(fee)
}

pub fn transfer_token(
    e: &Env,
    from: &Address,
    to: &Address,
    amount: &u128,
) -> Result<(), ContractError> {
    let native_token = storage::native_token(&e)?;
    let client = token::Client::new(&e, &native_token);

    client.transfer(&from, &to, &(*amount as i128));
    Ok(())
}

pub fn call_xcall_handle_message(e: &Env, nid: &String, msg: Bytes) -> Result<(), ContractError> {
    let xcall_addr = storage::get_xcall(&e)?;
    let client = XcallClient::new(&e, &xcall_addr);
    client.handle_message(&e.current_contract_address(), nid, &msg);

    Ok(())
}

pub fn call_xcall_handle_error(e: &Env, sn: u128) -> Result<(), ContractError> {
    let xcall_addr = storage::get_xcall(&e)?;
    let client = XcallClient::new(&e, &xcall_addr);
    client.handle_error(&e.current_contract_address(), &sn);

    Ok(())
}
