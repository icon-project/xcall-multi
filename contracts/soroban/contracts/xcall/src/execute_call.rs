use soroban_sdk::{vec, Address, Bytes, Env};

use crate::{
    contract::Xcall,
    errors::ContractError,
    event,
    messages::cs_message::CSMessage,
    types::{
        message::MessageType,
        result::{CSMessageResult, CSResponseType},
    },
};

impl Xcall {
    pub fn execute_message(env: &Env, req_id: u128, data: Bytes) -> Result<(), ContractError> {
        let req = Self::get_proxy_request(&env, req_id)?;

        let hash_data = Self::hash_data(&env, &data);
        if &hash_data != req.data() {
            return Err(ContractError::DataMismatch);
        }
        Self::remove_proxy_request(&env, req_id);

        let to = Address::from_string(&req.to());

        match req.msg_type() {
            MessageType::CallMessage => {
                Self::try_handle_call_message(
                    &env,
                    req_id,
                    to,
                    req.from(),
                    &data,
                    req.protocols().clone(),
                );
            }
            MessageType::CallMessagePersisted => {
                Self::handle_call_message(
                    &env,
                    to.clone(),
                    req.from(),
                    &data,
                    req.protocols().clone(),
                );
            }
            MessageType::CallMessageWithRollback => {
                Self::store_reply_state(&env, &req);
                let code = Self::try_handle_call_message(
                    &env,
                    req_id,
                    to,
                    req.from(),
                    &data,
                    req.protocols().clone(),
                );
                Self::remove_reply_state(&env);

                let response_code = code.into();
                let mut message = Bytes::new(&env);
                let call_reply = Self::remove_call_reply(&env);
                if call_reply.is_some() && response_code == CSResponseType::CSResponseSuccess {
                    message = call_reply.unwrap().encode(&env);
                }

                let result = CSMessageResult::new(req.sequence_no(), response_code, message);
                let cs_message = CSMessage::from_result(&env, &result).encode(&env);

                let nid = req.from().nid(&env);
                let mut destinations = req.protocols().clone();
                if destinations.is_empty() {
                    let deafult_connection = Self::default_connection(&env, nid.clone())?;
                    destinations = vec![&env, deafult_connection.to_string()];
                }

                for to in destinations {
                    Self::call_connection_send_message(
                        &env,
                        &to,
                        0_u128,
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
        let rollback = Self::get_rollback(&env, sequence_no)?;
        Self::ensure_rollback_enabled(&rollback)?;
        Self::remove_rollback(&env, sequence_no);

        Self::handle_call_message(
            &env,
            rollback.from().clone(),
            &Self::get_own_network_address(&env)?,
            rollback.rollback(),
            rollback.protocols().clone(),
        );
        event::rollback_executed(&env, sequence_no);

        Ok(())
    }
}
