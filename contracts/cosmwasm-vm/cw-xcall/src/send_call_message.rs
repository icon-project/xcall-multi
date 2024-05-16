use cosmwasm_std::{coins, BankMsg};
use cw_xcall_lib::message::call_message::CallMessage;
use cw_xcall_lib::message::msg_trait::IMessage;

use cw_xcall_lib::message::AnyMessage;
use cw_xcall_lib::message::{call_message_rollback::CallMessageWithRollback, envelope::Envelope};
use cw_xcall_lib::network_address::{NetId, NetworkAddress};

use crate::{assertion::is_contract, types::LOG_PREFIX};

use super::*;

impl<'a> CwCallService<'a> {
    pub fn send_call_message(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        _env: Env,
        to: NetworkAddress,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
        sources: Vec<String>,
        destinations: Vec<String>,
    ) -> Result<Response, ContractError> {
        let msg = if rollback.is_some() {
            AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
                data,
                rollback: rollback.unwrap(),
            })
        } else {
            AnyMessage::CallMessage(CallMessage { data })
        };
        let envelope = Envelope::new(msg, sources, destinations);
        self.send_call(deps, info, to, envelope)
    }

    pub fn validate_payload(
        &self,
        deps: Deps,
        caller: &Addr,
        envelope: &Envelope,
    ) -> Result<(), ContractError> {
        match &envelope.message {
            AnyMessage::CallMessage(_m) => Ok(()),
            AnyMessage::CallMessageWithRollback(m) => {
                if !is_contract(deps.querier, caller) {
                    return Err(ContractError::RollbackNotPossible);
                }
                self.ensure_rollback_length(&m.rollback().unwrap())?;
                Ok(())
            }
            AnyMessage::CallMessagePersisted(_) => Ok(()),
        }
    }

    pub fn send_call(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to: NetworkAddress,
        envelope: Envelope,
    ) -> Result<Response, ContractError> {
        let caller = info.sender.clone();
        let config = self.get_config(deps.as_ref().storage)?;
        let nid = config.network_id;
        self.validate_payload(deps.as_ref(), &caller, &envelope)?;

        let sequence_no = self.get_next_sn(deps.storage)?;

        let from = NetworkAddress::new(&nid, caller.as_ref());

        if envelope.message.rollback().is_some() {
            let rollback_data = envelope.message.rollback().unwrap();
            let request = Rollback::new(
                caller.clone(),
                to.clone(),
                envelope.sources.clone(),
                rollback_data,
                false,
            );

            self.store_call_request(deps.storage, sequence_no, &request)?;
        }
        let call_request = CSMessageRequest::new(
            from,
            to.account(),
            sequence_no,
            envelope.message.msg_type().clone(),
            envelope.message.data(),
            envelope.destinations,
        );
        let need_response = call_request.need_response();

        let event = event_xcall_message_sent(caller.to_string(), to.to_string(), sequence_no);
        // if contract is in reply state
        if envelope.message.rollback().is_none()
            && self.is_reply(deps.as_ref(), to.nid(), &envelope.sources)
        {
            self.save_call_reply(deps.storage, &call_request)?;
            let res = self.send_call_response(event, sequence_no);
            return Ok(res);
        }

        let mut confirmed_sources = envelope.sources;
        if confirmed_sources.is_empty() {
            let default = self.get_default_connection(deps.as_ref().storage, to.nid())?;
            confirmed_sources = vec![default.to_string()]
        }
        let message: CSMessage = call_request.into();
        let sn: i64 = if need_response { sequence_no as i64 } else { 0 };
        let mut total_spent = 0_u128;

        let submessages = confirmed_sources
            .iter()
            .map(|r| {
                return self
                    .query_connection_fee(deps.as_ref(), to.nid(), need_response, r)
                    .and_then(|fee| {
                        let fund = if fee > 0 {
                            total_spent = total_spent.checked_add(fee).unwrap();
                            coins(fee, config.denom.clone())
                        } else {
                            vec![]
                        };
                        let address = deps.api.addr_validate(r)?;

                        self.call_connection_send_message(&address, fund, to.nid(), sn, &message)
                    });
            })
            .collect::<Result<Vec<SubMsg>, ContractError>>()?;

        let total_paid = self.get_total_paid(deps.as_ref(), &info.funds)?;
        let fee_handler = self.fee_handler().load(deps.storage)?;
        let protocol_fee = self.get_protocol_fee(deps.as_ref().storage);
        let total_fee_required = protocol_fee + total_spent;

        if total_paid < total_fee_required {
            return Err(ContractError::InsufficientFunds);
        }
        let remaining = total_paid - total_spent;

        println!("{LOG_PREFIX} Sent Bank Message");
        let mut res = self
            .send_call_response(event, sequence_no)
            .add_submessages(submessages);

        if remaining > 0 {
            let msg = BankMsg::Send {
                to_address: fee_handler,
                amount: coins(remaining, config.denom),
            };
            res = res.add_message(msg);
        }

        Ok(res)
    }

    fn send_call_response(&self, event: Event, sequence_no: u128) -> Response {
        Response::new()
            .add_attribute("action", "xcall-service")
            .add_attribute("method", "send_packet")
            .add_attribute("sequence_no", sequence_no.to_string())
            .add_event(event)
    }

    pub fn is_reply(&self, deps: Deps, to: NetId, sources: &Vec<String>) -> bool {
        if self.get_call_reply(deps.storage).is_some() {
            return false;
        }

        if let Ok(reqid) = self.get_execute_request_id(deps.storage) {
            let request = self.get_proxy_request(deps.storage, reqid).unwrap();
            if request.from().nid() != to {
                return false;
            }
            return self.are_equal(request.protocols(), sources);
        }
        false
    }

    fn are_equal(&self, protocols: &Vec<String>, sources: &Vec<String>) -> bool {
        if protocols.len() != sources.len() {
            return false;
        }
        for protocol in protocols.iter() {
            if !sources.contains(protocol) {
                return false;
            }
        }
        true
    }
}
