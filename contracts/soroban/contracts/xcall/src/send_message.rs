use soroban_sdk::{vec, Address, Bytes, Env, String, Vec};
use soroban_xcall_lib::{
    messages::{envelope::Envelope, msg_trait::IMessage, AnyMessage},
    network_address::NetworkAddress,
};

use crate::{
    connection,
    errors::ContractError,
    event, helpers,
    storage::{self, protocol_fee},
    types::{message::CSMessage, request::CSMessageRequest, rollback::Rollback},
};

pub fn send_call(
    env: &Env,
    tx_origin: Address,
    sender: Address,
    envelope: Envelope,
    to: String,
) -> Result<u128, ContractError> {
    sender.require_auth();
    tx_origin.require_auth();

    let config = storage::get_config(&env)?;
    let sequence_no = storage::get_next_sn(&env);

    let to = NetworkAddress::from_string(to.clone());
    let (nid_to, dst_account) = to.parse_network_address(&env);
    let from = NetworkAddress::new(&env, config.network_id, sender.to_string());

    process_message(&env, &to, sequence_no, &sender, &envelope)?;

    let request = CSMessageRequest::new(
        from,
        dst_account,
        sequence_no,
        envelope.destinations,
        envelope.message.msg_type(),
        envelope.message.data(),
    );

    let need_response = request.need_response();

    let cs_message = CSMessage::from_request(&env, &request);
    let encode_msg = cs_message.encode(&env);
    helpers::ensure_data_size(encode_msg.len() as usize)?;

    call_connection(
        &env,
        &tx_origin,
        &nid_to,
        sequence_no,
        envelope.sources,
        need_response,
        encode_msg.clone(),
    )?;
    claim_protocol_fee(&env, &tx_origin)?;

    event::message_sent(&env, sender, to.to_string(), sequence_no);

    Ok(sequence_no)
}

pub fn claim_protocol_fee(e: &Env, tx_origin: &Address) -> Result<(), ContractError> {
    let protocol_fee = protocol_fee(&e);
    if protocol_fee > 0 {
        let fee_handler = storage::get_fee_handler(&e)?;
        helpers::transfer_token(&e, &tx_origin, &fee_handler, &protocol_fee)?;
    }

    Ok(())
}

pub fn call_connection(
    e: &Env,
    tx_origin: &Address,
    nid: &String,
    sequence_no: u128,
    sources: Vec<String>,
    rollback: bool,
    msg: Bytes,
) -> Result<(), ContractError> {
    let mut sources = sources;
    let sn = if rollback { sequence_no as i64 } else { 0 };
    if sources.is_empty() {
        let default_conn = storage::default_connection(&e, nid.clone())?;
        sources = vec![&e, default_conn.to_string()];
    }

    for source in sources.iter() {
        connection::call_connection_send_message(&e, tx_origin, &source, &nid, sn, &msg)?;
    }

    Ok(())
}

pub fn process_message(
    e: &Env,
    to: &NetworkAddress,
    sequence_no: u128,
    sender: &Address,
    envelope: &Envelope,
) -> Result<(), ContractError> {
    match &envelope.message {
        AnyMessage::CallMessage(_) => Ok(()),
        AnyMessage::CallMessagePersisted(_) => Ok(()),
        AnyMessage::CallMessageWithRollback(msg) => {
            if !helpers::is_contract(&e, sender) {
                return Err(ContractError::RollbackNotPossible);
            }
            helpers::ensure_rollback_size(&msg.rollback().unwrap())?;

            let rollback = Rollback::new(
                sender.clone(),
                to.clone(),
                envelope.sources.clone(),
                msg.rollback.clone(),
                false,
            );
            storage::store_rollback(&e, sequence_no, &rollback);

            Ok(())
        }
    }
}

pub fn get_total_fee(
    env: &Env,
    nid: &String,
    sources: Vec<String>,
    rollback: bool,
) -> Result<u128, ContractError> {
    let mut sources = sources;
    if sources.is_empty() {
        let default_conn = storage::default_connection(&env, nid.clone())?;
        sources = vec![&env, default_conn.to_string()];
    }

    let protocol_fee = storage::protocol_fee(&env);
    let mut connections_fee = 0_u128;
    for source in sources.iter() {
        let fee = connection::query_connection_fee(&env, &nid, rollback, &source)?;
        if fee > 0 {
            connections_fee = connections_fee.checked_add(fee).expect("no overflow");
        }
    }

    Ok(connections_fee + protocol_fee)
}
