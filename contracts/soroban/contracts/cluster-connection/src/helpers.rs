use soroban_sdk::{token, xdr::ToXdr, Address, Bytes, BytesN, Env, Map, String, Vec};
use crate::{errors::ContractError, interfaces::interface_xcall::XcallClient, storage};
use soroban_xcall_lib::network_address::NetworkAddress;

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
    let dst_network = get_network_id(e);
    let validators = storage::get_validators(e).unwrap();
    let threshold = storage::get_validators_threshold(e).unwrap();

    if signatures.len() < threshold {
        return false
    }
    let message_hash = e.crypto().keccak256(&get_encoded_message(e, src_network, conn_sn, message, &dst_network));
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
 

pub fn string_to_bytes(env: &Env, value: String) -> Bytes {
    let string_xdr = value.clone().to_xdr(&env);
    let mut bytes = Bytes::new(&env);
    for i in 8..(8 + value.len()) {
        if let Some(byte) = string_xdr.get(i) {
            bytes.push_back(byte);
        }
    }
    bytes
}

pub fn get_encoded_message(e: &Env, src_network: &String, conn_sn: &u128, message: &Bytes, dst_network: &String) -> Bytes {
    let mut encoded = Bytes::new(e);
    encoded.append(&string_to_bytes(e, src_network.clone()));
    encoded.append(&u128_to_string(e, *conn_sn));
    encoded.append(message);
    encoded.append(&string_to_bytes(e, dst_network.clone()));
    encoded
}

pub fn u128_to_string(env: &Env, value: u128) -> Bytes {
    let mut num = value;    
    let mut temp_bytes = Bytes::new(&env);
    let mut bytes = Bytes::new(&env);

    if value == 0 {
        temp_bytes.push_back(b'0');
        return temp_bytes;
    }
    while num > 0 {
        let digit = (num % 10) as u8 + b'0';
        temp_bytes.push_back(digit);
        num /= 10;
    }
    for byte in temp_bytes.iter().rev() {
        bytes.push_back(byte); 
    }
    bytes
}

#[cfg(not(test))]
pub fn get_network_id(e: &Env) -> String {
    let xcall_addr = storage::get_xcall(&e).unwrap();
    let client = XcallClient::new(&e, &xcall_addr);
    let network_address = NetworkAddress::from_string(client.get_network_address());
    network_address.nid(&e)
}

#[cfg(test)]
pub fn get_network_id(e: &Env) -> String {
    let network_address =  NetworkAddress::from_string(String::from_str(e, "archway/testnet"));
    network_address.nid(&e)
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

#[test]
fn verify_encoded_message() {
    use soroban_sdk::bytes;
    let env = Env::default();
    let src_network = String::from_str(&env, "0x2.icon");
    let conn_sn = 128;
    let message = bytes!(&env, 0x68656c6c6f);
    let dst_network = String::from_str(&env, "archway");
    let encoded = get_encoded_message(&env, &src_network, &conn_sn, &message, &dst_network);
    assert_eq!(encoded, bytes!(&env,0x3078322e69636f6e31323868656c6c6f61726368776179));
}