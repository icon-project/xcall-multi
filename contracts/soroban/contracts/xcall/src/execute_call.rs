use soroban_sdk::{vec, Address, Bytes, Env, String};
use soroban_xcall_lib::messages::msg_type::MessageType;

use crate::{
    connection, dapp,
    errors::ContractError,
    event, helpers, storage,
    types::{
        message::CSMessage,
        result::{CSMessageResult, CSResponseType},
    },
};

pub fn execute_message(
    env: &Env,
    sender: Address,
    req_id: u128,
    data: Bytes,
) -> Result<(), ContractError> {
    let req = storage::get_proxy_request(&env, req_id)?;

    let hash_data = helpers::hash_data(&env, &data);
    if &hash_data != req.data() {
        return Err(ContractError::DataMismatch);
    }
    storage::remove_proxy_request(&env, req_id);

    let to = Address::from_string(&req.to());

    match req.msg_type() {
        MessageType::CallMessage => {
            dapp::try_handle_call_message(
                &env,
                req_id,
                to,
                &req.from(),
                &data,
                req.protocols().clone(),
            );
        }
        MessageType::CallMessagePersisted => {
            dapp::handle_call_message(
                &env,
                to.clone(),
                &req.from(),
                &data,
                req.protocols().clone(),
            );

            let code: u8 = CSResponseType::CSResponseSuccess.into();
            event::call_executed(&env, req_id, code, String::from_str(&env, "success"));
        }
        MessageType::CallMessageWithRollback => {
            let code = dapp::try_handle_call_message(
                &env,
                req_id,
                to,
                &req.from(),
                &data,
                req.protocols().clone(),
            );

            let response_code = code.into();
            let result = CSMessageResult::new(req.sequence_no(), response_code, Bytes::new(&env));
            let cs_message = CSMessage::from_result(&env, &result).encode(&env);

            let nid = req.from().nid(&env);
            let mut destinations = req.protocols().clone();
            if destinations.is_empty() {
                let deafult_connection = storage::default_connection(&env, nid.clone())?;
                destinations = vec![&env, deafult_connection.to_string()];
            }

            for to in destinations {
                connection::call_connection_send_message(
                    &env,
                    &sender,
                    &to,
                    &nid,
                    -(req.sequence_no() as i64),
                    &cs_message,
                )?;
            }
        }
    };

    Ok(())
}

pub fn execute_rollback_message(env: &Env, sequence_no: u128) -> Result<(), ContractError> {
    let rollback = storage::get_rollback(&env, sequence_no)?;
    helpers::ensure_rollback_enabled(&rollback)?;
    storage::remove_rollback(&env, sequence_no);

    dapp::handle_call_message(
        &env,
        rollback.from().clone(),
        &storage::get_own_network_address(&env)?,
        rollback.rollback(),
        rollback.protocols().clone(),
    );
    event::rollback_executed(&env, sequence_no);

    Ok(())
}
