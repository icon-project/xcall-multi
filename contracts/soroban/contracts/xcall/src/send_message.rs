use soroban_sdk::{vec, Address, Bytes, Env, String, Vec};

use crate::{
    contract::Xcall,
    errors::ContractError,
    event,
    messages::{cs_message::CSMessage, envelope::Envelope, AnyMessage},
    types::{
        message::IMessage, network_address::NetworkAddress, request::CSMessageRequest,
        rollback::Rollback,
    },
};

impl Xcall {
    pub fn send_message(
        env: &Env,
        tx_origin: Address,
        sender: Address,
        envelope: Envelope,
        to: NetworkAddress,
    ) -> Result<u128, ContractError> {
        sender.require_auth();
        tx_origin.require_auth();

        let sequence_no = Self::get_next_sn(&env);

        let config = Self::get_config(&env)?;
        let (nid_to, dst_account) = to.parse_network_address(&env);
        let from = NetworkAddress::new(
            &env,
            config.network_id,
            env.current_contract_address().to_string(),
        );

        Self::process_message(&env, &to, sequence_no, &sender, &envelope)?;

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
        Self::ensure_data_size(encode_msg.len() as usize)?;

        if Self::is_reply(&env, &nid_to, &envelope.sources) && !need_response {
            Self::store_call_reply(&env, &request);
        } else {
            Self::call_connection(
                &env,
                &tx_origin,
                &nid_to,
                sequence_no,
                envelope.sources,
                need_response,
                encode_msg.clone(),
            )?;
            Self::claim_protocol_fee(&env, &tx_origin)?;
        }
        event::message_sent(&env, sender, to, sequence_no);

        Ok(sequence_no)
    }

    pub fn claim_protocol_fee(e: &Env, tx_origin: &Address) -> Result<(), ContractError> {
        let protocol_fee = Self::get_protocol_fee(&e)?;
        if protocol_fee > 0 {
            let fee_handler = Self::get_fee_handler(&e)?;
            Self::transfer_token(&e, &tx_origin, &fee_handler, &protocol_fee)?;
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
            let default_conn = Self::default_connection(&e, nid.clone())?;
            sources = vec![&e, default_conn.to_string()];
        }

        for source in sources.iter() {
            Self::call_connection_send_message(&e, tx_origin, &source, &nid, sn, &msg)?;
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
                if !Self::is_contract(&e, sender) {
                    return Err(ContractError::RollbackNotPossible);
                }
                Self::ensure_rollback_size(&msg.rollback)?;

                if envelope.message.rollback().is_some() {
                    let rollback_data = envelope.message.rollback().unwrap();
                    let rollback = Rollback::new(
                        sender.clone(),
                        to.clone(),
                        envelope.sources.clone(),
                        rollback_data,
                        false,
                    );

                    Self::store_rollback(&e, sequence_no, &rollback);
                }
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
        if !rollback && Self::is_reply(&env, &nid, &sources) {
            return Ok(0_u128);
        }

        let mut sources = sources;
        if sources.is_empty() {
            let default_conn = Self::default_connection(&env, nid.clone())?;
            sources = vec![&env, default_conn.to_string()];
        }

        let protocol_fee = Self::protocol_fee(&env);
        let mut connections_fee = 0_u128;
        for source in sources.iter() {
            let fee = Self::query_connection_fee(&env, &nid, rollback, &source)?;
            if fee > 0 {
                connections_fee = connections_fee.checked_add(fee).expect("no overflow");
            }
        }

        Ok(connections_fee + protocol_fee)
    }

    pub fn is_reply(e: &Env, nid: &String, sources: &Vec<String>) -> bool {
        if let Some(req) = Self::get_reply_state(&e) {
            if req.from().nid(&e) != *nid {
                return false;
            }
            return Self::are_array_equal(req.protocols(), &sources);
        }
        false
    }

    pub fn are_array_equal(protocols: &Vec<String>, sources: &Vec<String>) -> bool {
        if protocols.len() != sources.len() {
            return false;
        }
        for protocol in protocols.iter() {
            if !sources.contains(protocol) {
                return false;
            }
        }
        return true;
    }
}
