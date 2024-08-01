use common::{rlp, utils::keccak256};
use cw_xcall_lib::network_address::NetId;

use super::*;

impl<'a> CwCallService<'a> {
    pub fn handle_message(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        from_nid: NetId,
        message: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let cfg = self.get_config(deps.storage).unwrap();
        if cfg.network_id == from_nid.to_string() {
            return Err(ContractError::ProtocolsMismatch);
        }

        let call_service_message: CSMessage = CSMessage::try_from(message)?;
        match call_service_message.message_type() {
            CSMessageType::CSMessageRequest => {
                self.handle_request(deps, info, from_nid, call_service_message.payload())
            }
            CSMessageType::CSMessageResult => {
                self.handle_result(deps, info, call_service_message.payload())
            }
        }
    }

    pub fn handle_reply(
        &self,
        deps: DepsMut,
        rollback: Rollback,
        request: CSMessageRequest,
    ) -> Result<Event, ContractError> {
        // reply can  be targeted to source nid but any contract
        if !(rollback.to().nid() == request.from().nid()) {
            return Err(ContractError::InvalidReplyReceived);
        }
        let request_id = self.increment_last_request_id(deps.storage)?;

        let req = CSMessageRequest::new(
            request.from().clone(),
            request.to().clone(),
            request.sequence_no(),
            request.msg_type(),
            keccak256(request.data().unwrap()).to_vec(),
            rollback.protocols().clone(),
        );
        self.store_proxy_request(deps.storage, request_id, &req)?;

        let event = event_call_message(
            request.from().to_string(),
            request.to().to_string(),
            request.sequence_no(),
            request_id,
            request.data().unwrap().to_vec(),
        );
        Ok(event)
    }

    pub fn handle_request(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        src_net: NetId,
        data: &[u8],
    ) -> Result<Response, ContractError> {
        let request: CSMessageRequest = rlp::decode(data).unwrap();

        let from = request.from().clone();
        if from.nid() != src_net {
            return Err(ContractError::ProtocolsMismatch);
        }
        let source = info.sender.to_string();
        let source_valid =
            self.is_valid_source(deps.as_ref().storage, src_net, &source, request.protocols())?;
        if !source_valid {
            return Err(ContractError::ProtocolsMismatch);
        }

        let to = deps.api.addr_validate(request.to().as_str())?;

        if request.protocols().len() > 1 {
            let key = keccak256(data).to_vec();
            let caller = info.sender;
            self.save_pending_requests(deps.storage, key.clone(), caller.to_string())?;
            let registered =
                self.get_pending_requests_by_hash(deps.as_ref().storage, key.clone())?;

            if registered.len() != request.protocols().len() {
                return Ok(Response::new());
            }

            self.remove_pending_request_by_hash(deps.storage, key)?;
        }
        let request_id = self.increment_last_request_id(deps.storage)?;

        let req = CSMessageRequest::new(
            request.from().clone(),
            request.to().clone(),
            request.sequence_no(),
            request.msg_type(),
            keccak256(request.data().unwrap()).to_vec(),
            request.protocols().clone(),
        );
        self.store_proxy_request(deps.storage, request_id, &req)?;

        let event = event_call_message(
            from.to_string(),
            to.to_string(),
            request.sequence_no(),
            request_id,
            request.data().unwrap().to_vec(),
        );

        Ok(Response::new()
            .add_attribute("action", "call_service")
            .add_attribute("method", "handle_response")
            .add_event(event))
    }

    pub fn handle_result(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        data: &[u8],
    ) -> Result<Response, ContractError> {
        let result: CSMessageResult = rlp::decode(data).unwrap();

        let response_sequence_no = result.sequence_no();

        let mut call_request = self
            .get_call_request(deps.storage, response_sequence_no)
            .map_err(|_e| ContractError::CallRequestNotFound {
                sn: response_sequence_no,
            })?;

        let source = info.sender.to_string();
        let source_valid = self.is_valid_source(
            deps.as_ref().storage,
            call_request.to().nid(),
            &source,
            call_request.protocols(),
        )?;
        if !source_valid {
            return Err(ContractError::ProtocolsMismatch);
        }

        if call_request.protocols().len() > 1 {
            let key = keccak256(data).to_vec();
            let caller = info.sender;
            self.save_pending_responses(deps.storage, key.clone(), caller.to_string())?;
            let registered =
                self.get_pending_responses_by_hash(deps.as_ref().storage, key.clone())?;

            if registered.len() != call_request.protocols().len() {
                return Ok(Response::new());
            }

            self.remove_pending_responses_by_hash(deps.storage, key)?;
        }
        let response_event = event_response_message(
            response_sequence_no,
            (result.response_code().clone()).into(),
        );

        match result.response_code() {
            CallServiceResponseType::CallServiceResponseSuccess => {
                self.cleanup_request(deps.storage, response_sequence_no);
                self.set_successful_response(deps.storage, response_sequence_no)?;
                let mut res = Response::new()
                    .add_attribute("action", "call_service")
                    .add_attribute("method", "handle_response")
                    .add_event(response_event);
                if result.get_message().is_some() {
                    let reply = result.get_message().unwrap();
                    let event = self.handle_reply(deps, call_request, reply)?;
                    res = res.add_event(event);
                }

                Ok(res)
            }
            _ => {
                self.ensure_rollback_length(call_request.rollback())
                    .unwrap();
                call_request.set_enabled();
                self.store_call_request(deps.storage, response_sequence_no, &call_request)?;

                let rollback_event = event_rollback_message(response_sequence_no);

                Ok(Response::new()
                    .add_attribute("action", "call_service")
                    .add_attribute("method", "handle_response")
                    .add_event(response_event)
                    .add_event(rollback_event))
            }
        }
    }

    /// The function removes a call request from storage based on a given sequence number.
    ///
    /// Arguments:
    ///
    /// * `store`: `store` is a mutable reference to a trait object of type `dyn Storage`. This means
    /// that the function can accept any object that implements the `Storage` trait. The `Storage` trait
    /// defines methods for storing and retrieving data in a persistent storage, such as a database or a
    /// file system.
    /// * `sequence_no`: `sequence_no` is an unsigned 128-bit integer that represents the sequence
    /// number of a call request that needs to be cleaned up. It is used as an identifier to locate and
    /// remove the corresponding call request from the storage.
    pub fn cleanup_request(&self, store: &mut dyn Storage, sequence_no: u128) {
        self.remove_call_request(store, sequence_no);
    }

    pub fn is_valid_source(
        &self,
        store: &dyn Storage,
        src_net: NetId,
        source: &String,
        protocols: &Vec<String>,
    ) -> Result<bool, ContractError> {
        if protocols.contains(source) {
            return Ok(true);
        }
        if protocols.is_empty() {
            let default_conn = self.get_default_connection(store, src_net)?;
            Ok(source.clone() == default_conn)
        } else {
            Ok(false)
        }
    }

    pub fn handle_error(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        sn: u128,
    ) -> Result<Response, ContractError> {
        let msg = CSMessageResult::new(
            sn,
            CallServiceResponseType::CallServiceResponseFailure,
            None,
        );
        self.handle_result(deps, info, &rlp::encode(&msg))
    }
}
