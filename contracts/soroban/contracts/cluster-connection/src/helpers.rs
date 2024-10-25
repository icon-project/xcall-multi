use core::panic;

use soroban_sdk::{token, Address, Bytes, BytesN, Env, FromVal, Map, String, Vec};
use crate::{errors::ContractError, interfaces::interface_xcall::XcallClient, storage};

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

pub fn compress_public_keys(e: &Env, uncompressed_public_key: BytesN<65>) -> Bytes {
    let uncompressed_pub_key_array = uncompressed_public_key.to_array();
    if uncompressed_pub_key_array[0] != 0x04 {
        //return empty bytessize(33);
        return Bytes::from_array(e, &[0u8; 33]);
    }

    let x = &uncompressed_pub_key_array[1..33];
    let y = &uncompressed_pub_key_array[33..65];

    let prefix = if y[31] % 2 == 0 { 0x02 } else { 0x03 };

    let mut compressed_pub_key_array = [0u8; 33];
    compressed_pub_key_array[0] = prefix;
    compressed_pub_key_array[1..].copy_from_slice(x);
    let compressed_pub_key = Bytes::from_array(e, &compressed_pub_key_array);

    compressed_pub_key
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
        let recovery_id = r_s_v[64] as u32; // recovery ID

        let signature = BytesN::<64>::from_array(e, &signature_array);

        let uncompressed_public_key = e.crypto().secp256k1_recover(&message_hash, &signature, recovery_id);

        let compressed_pub_key = compress_public_keys(e, uncompressed_public_key);

        let stellar_address = Address::from_string_bytes(&compressed_pub_key);

        if validators.contains(&stellar_address) {   
            if !unique_validators.contains_key(stellar_address.clone()) {
                unique_validators.set(stellar_address, count);
                count += 1;
            }    
        }
    }
    (unique_validators.len() as u32) >= threshold

}
 

pub fn get_encoded_message(e: &Env, src_network: &String, conn_sn: &u128, message: &Bytes) -> Bytes {
    let mut result = Bytes::from_val(e, &src_network.to_val());
    result.extend_from_slice(&conn_sn.to_be_bytes());
    result.append(message);
    result
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
