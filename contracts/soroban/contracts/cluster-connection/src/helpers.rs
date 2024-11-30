use soroban_sdk::{token, vec, Address, Bytes, BytesN, Env, Map, String, Vec};
use crate::{errors::ContractError, interfaces::interface_xcall::XcallClient, storage};
use soroban_rlp::encoder;

pub fn ensure_relayer(e: &Env) -> Result<Address, ContractError> {
    let relayer = storage::relayer(&e)?;
    relayer.require_auth();

    Ok(relayer)
}

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

pub fn verify_signatures(
    e: &Env,
    signatures: Vec<BytesN<65>>,
    src_network: &String,
    conn_sn: &u128,
    message: &Bytes,
) -> bool {
    let validators = storage::get_validators(e).unwrap();
    let threshold = storage::get_validators_threshold(e).unwrap();

    if signatures.len() < threshold {
        return false
    }
    let message_hash = e.crypto().keccak256(&get_encoded_message(e, src_network, conn_sn, message));
     let mut unique_validators = Map::new(e);
     let mut count = 0;
     

      for sig in signatures.iter() {
        let r_s_v = sig.to_array();
        // Separate signature (r + s) and recovery ID
        let signature_array: [u8; 64] = r_s_v[..64].try_into().unwrap(); // r + s part
        let recovery_code = match r_s_v[64] {
            rc if rc >= 27 => rc - 27,
            rc => rc,
        };
        let signature = BytesN::<64>::from_array(e, &signature_array);

        let public_key = e.crypto().secp256k1_recover(&message_hash, &signature, recovery_code as u32);

        if validators.contains(&public_key) {   
            if !unique_validators.contains_key(public_key.clone()) {
                unique_validators.set(public_key, count);
                count += 1;
            }    
        }
    }
    (unique_validators.len() as u32) >= threshold

}
 

pub fn get_encoded_message(e: &Env, src_network: &String, conn_sn: &u128, message: &Bytes) -> Bytes {
    let mut list = vec![&e];
    list.push_back(encoder::encode_string(&e, src_network.clone()));
    list.push_back(encoder::encode_u128(&e, conn_sn.clone()));
    list.push_back(encoder::encode(&e, message.clone()));

    encoder::encode_list(&e, list, false)
}

#[cfg(not(test))]
pub fn call_xcall_handle_message(e: &Env, nid: &String, msg: Bytes) -> Result<(), ContractError> {
    let xcall_addr = storage::get_xcall(&e)?;
    let client = XcallClient::new(&e, &xcall_addr);
    client.handle_message(&e.current_contract_address(), nid, &msg);

    Ok(())
}

#[cfg(test)]
pub fn call_xcall_handle_message(_e: &Env, _nid: &String, _msg: Bytes) -> Result<(), ContractError> {
    Ok(())
}



pub fn call_xcall_handle_error(e: &Env, sn: u128) -> Result<(), ContractError> {
    let xcall_addr = storage::get_xcall(&e)?;
    let client = XcallClient::new(&e, &xcall_addr);
    client.handle_error(&e.current_contract_address(), &sn);

    Ok(())
}
