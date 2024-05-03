use soroban_sdk::{vec, xdr::ToXdr, Address, Bytes, Env};

use crate::{
    contract::Xcall,
    errors::ContractError,
    event,
    messages::cs_message::CSMessage,
    types::{message::MessageType, result::CSMessageResult},
};

impl Xcall {
    pub fn execute_message(env: &Env, req_id: u128, data: Bytes) -> Result<(), ContractError> {
        let req = Self::get_proxy_request(&env, req_id)?;

        let data_xdr = data.clone().to_xdr(&env);
        if data_xdr != req.data().clone() {
            return Err(ContractError::DataMismatch);
        }

        let to = Address::from_string(&req.to());
        Self::remove_proxy_request(&env, req_id);

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

                // TODO: rlp encode call reply
                let _call_reply = Self::remove_call_reply(&env);
                let call_reply = Some(Bytes::new(&env));

                let response_code = code.try_into().unwrap();
                let response =
                    CSMessageResult::new(&env, req.sequence_no(), response_code, call_reply);

                // TODO: rlp encode
                let _message: CSMessage = response.into();
                let message = Bytes::new(&env);

                let sn = -(req.sequence_no() as i64);
                let nid = req.from().nid(&env).clone();
                let mut destinations = req.protocols().clone();
                if destinations.is_empty() {
                    let deafult_connection = Self::default_connection(&env, nid.clone())?;
                    destinations = vec![&env, deafult_connection.to_string()];
                }

                for to in destinations {
                    Self::call_connection_send_message(&env, &to, 0_u128, &nid, sn, &message)?;
                }
            }
        };

        Ok(())
    }

    pub fn execute_rollback(env: &Env, sequence_no: u128) -> Result<(), ContractError> {
        let rollback = Self::get_rollback(&env, sequence_no)?;
        Self::remove_rollback(&env, sequence_no);
        Self::ensure_rollback_enabled(&rollback)?;

        let from = Self::get_own_network_address(&env)?;

        Self::handle_call_message(
            &env,
            rollback.from().clone(),
            &from,
            rollback.rollback(),
            rollback.protocols().clone(),
        );

        event::rollback_executed(&env, sequence_no);

        Ok(())
    }
}
