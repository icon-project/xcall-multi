use soroban_sdk::{token, Address, Bytes, Env, String};

use crate::contract::CentralizedConnection;
use crate::errors::ContractError;

mod xcall {
    soroban_sdk::contractimport!(file = "./wasm/xcall.wasm");
}

impl CentralizedConnection {
    pub fn ensure_admin(e: &Env) -> Result<Address, ContractError> {
        let admin = Self::admin(&e)?;
        admin.require_auth();

        Ok(admin)
    }

    pub fn ensure_xcall(e: &Env) -> Result<Address, ContractError> {
        let xcall = Self::get_xcall(&e)?;
        xcall.require_auth();

        Ok(xcall)
    }

    pub fn get_network_fee(env: &Env, network_id: String, response: bool) -> u128 {
        let mut fee = Self::get_msg_fee(&env, network_id.clone());
        if response {
            fee += Self::get_res_fee(&env, network_id);
        }

        fee
    }

    pub fn transfer_token(
        e: &Env,
        from: &Address,
        to: &Address,
        amount: &u128,
    ) -> Result<(), ContractError> {
        let native_token = Self::native_token(&e)?;
        let client = token::Client::new(&e, &native_token);

        client.transfer(&from, &to, &(*amount as i128));
        Ok(())
    }

    pub fn call_xcall_handle_message(
        e: &Env,
        nid: &String,
        msg: Bytes,
    ) -> Result<(), ContractError> {
        let xcall_addr = Self::get_xcall(&e)?;
        let client = xcall::Client::new(&e, &xcall_addr);
        client.handle_message(&nid, &msg);

        Ok(())
    }

    pub fn call_xcall_handle_error(e: &Env, sn: u128) -> Result<(), ContractError> {
        let xcall_addr = Self::get_xcall(&e)?;
        let client = xcall::Client::new(&e, &xcall_addr);
        client.handle_error(&sn);

        Ok(())
    }
}
