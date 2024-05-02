use soroban_sdk::{xdr::ToXdr, Address, Env, String, Vec};

use crate::{
    contract::Xcall,
    errors::ContractError,
    event,
    types::{
        message::{CSMessage, CSMessageType},
        message_types::Rollback,
        network_address::NetId,
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
    },
};

impl Xcall {
    pub fn handle(env: &Env, from_nid: NetId, _msg: CSMessage) -> Result<(), ContractError> {
        let config = Self::get_config(&env)?;
        if config.network_id != from_nid.0 {
            return Err(ContractError::ProtocolsMismatch);
        }

        // TODO: _msg will be Bytes -> decode and then use here
        match _msg.message_type() {
            CSMessageType::CSMessageRequest => {
                // Self::handle_request(&env, sender, from_nid, req);
            }
            CSMessageType::CSMessageResult => {
                // Self::handle_result(&env, sender, data);s
            }
        }

        Ok(())
    }

    pub fn handle_request(
        env: &Env,
        sender: &Address,
        from_net: NetId,
        req: &CSMessageRequest,
    ) -> Result<(), ContractError> {
        // TODO: rlp decoding of data
        let req = req;

        let (src_net, _) = req.from().parse_network_address(&env);
        if src_net != from_net {
            return Err(ContractError::ProtocolsMismatch);
        }
        let source = sender.to_string();
        let source_valid = Self::is_valid_source(&env, &source, src_net, &req.protocols())?;
        if !source_valid {
            return Err(ContractError::ProtocolsMismatch);
        }

        if req.protocols().len() > 1 {
            let hash = env.crypto().keccak256(&req.clone().to_xdr(&env));
            let mut pending_request = Self::get_pending_request(&env, hash.clone());

            if !pending_request.contains(source.clone()) {
                pending_request.push_back(source);
                Self::store_pending_request(&env, hash.clone(), &pending_request);
            }
            if pending_request.len() != req.protocols().len() {
                return Ok(());
            }
            Self::remove_pending_request(&env, hash);
        }

        let req_id = Self::increment_last_request_id(&env);
        let request = CSMessageRequest::new(
            req.from().clone(),
            req.to().clone(),
            req.sequence_no(),
            req.protocols().clone(),
            req.msg_type(),
            env.crypto().keccak256(&req.data()).to_xdr(&env),
        );

        Self::store_proxy_request(&env, req_id, &request);

        event::call_message(
            &env,
            req.from().clone(),
            req.to().clone(),
            req.sequence_no(),
            req_id,
            req.data().clone(),
        );

        Ok(())
    }

    pub fn handle_result(
        env: &Env,
        sender: &Address,
        data: CSMessageResult,
    ) -> Result<(), ContractError> {
        // TODO: rlp decode data
        let result = data;

        let source = sender.to_string();
        let sequence_no = result.sequence_no();
        let mut rollback = Self::get_rollback(&env, sequence_no)?;

        let source_valid =
            Self::is_valid_source(&env, &source, rollback.to().nid(&env), &rollback.protocols)?;
        if !source_valid {
            return Err(ContractError::ProtocolsMismatch);
        }

        if rollback.protocols().len() > 1 {
            let hash = env.crypto().keccak256(&result.clone().to_xdr(&env));
            let mut pending_response = Self::get_pending_response(&env, hash.clone());

            if !pending_response.contains(source.clone()) {
                pending_response.push_back(source);
                Self::store_pending_response(&env, hash.clone(), &pending_response);
            }
            if pending_response.len() != rollback.protocols().len() {
                return Ok(());
            }
            Self::remove_pending_response(&env, hash);
        }

        event::response_message(&env, result.response_code().clone(), sequence_no);

        match result.response_code() {
            CSResponseType::CSResponseSuccess => {
                Self::remove_rollback(&env, sequence_no);
                Self::save_success_response(&env, sequence_no);

                if result.message().is_some() {
                    let _reply_msg = result.message().unwrap();
                    // Self::handle_reply(&env, &rollback, reply_msg);
                }

                ()
            }
            _ => {
                Self::ensure_rollback_size(&rollback.rollback())?;
                rollback.enable();

                Self::store_rollback(&env, sequence_no, &rollback);

                event::rollback_message(&env, sequence_no);

                ()
            }
        }

        Ok(())
    }

    pub fn handle_reply(
        env: &Env,
        rollback: &Rollback,
        reply: CSMessageRequest,
    ) -> Result<(), ContractError> {
        if rollback.to().nid(&env) != reply.from().nid(&env) {
            return Err(ContractError::InvalidReplyReceived);
        }
        let req_id = Self::increment_last_request_id(&env);

        let req = CSMessageRequest::new(
            reply.from().clone(),
            reply.to().clone(),
            reply.sequence_no(),
            reply.protocols().clone(),
            reply.msg_type(),
            env.crypto().keccak256(&reply.data()).to_xdr(&env),
        );

        Self::store_proxy_request(&env, req_id, &req);
        event::call_message(
            &env,
            req.from().clone(),
            req.to().clone(),
            req.sequence_no(),
            req_id,
            req.data().clone(),
        );

        Ok(())
    }

    pub fn handle_error_message(env: &Env, _sender: Address, sn: u128) {
        let _msg = CSMessageResult::new(&env, sn, CSResponseType::CSResponseFailure, None);
        // TODO: rlp encode msg
        // Self::handle_result(&env, &sender, _msg);
    }

    pub fn is_valid_source(
        e: &Env,
        sender: &String,
        src_net: NetId,
        protocols: &Vec<String>,
    ) -> Result<bool, ContractError> {
        if protocols.contains(sender) {
            return Ok(true);
        }

        let default_connection = Self::default_connection(e, src_net)?;
        Ok(sender.clone() == default_connection.to_string())
    }
}
