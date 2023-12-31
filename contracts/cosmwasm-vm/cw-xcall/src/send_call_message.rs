use cosmwasm_std::{coins, BankMsg};
use cw_xcall_lib::network_address::NetworkAddress;

use crate::types::LOG_PREFIX;

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
        let caller = info.sender;
        let config = self.get_config(deps.as_ref().storage)?;
        let nid = config.network_id;

        self.ensure_caller_is_contract_and_rollback_is_null(deps.as_ref(), &caller, &rollback)?;

        let need_response = rollback.is_some();

        let rollback_data = match rollback {
            Some(data) => data,
            None => vec![],
        };

        self.ensure_rollback_length(&rollback_data)?;
        println!("{LOG_PREFIX} Packet Validated");

        let sequence_no = self.get_next_sn(deps.storage)?;
        let mut confirmed_sources = sources.clone();
        let from = NetworkAddress::new(&nid, caller.as_ref());

        if confirmed_sources.is_empty() {
            let default = self.get_default_connection(deps.as_ref().storage, to.nid())?;
            confirmed_sources = vec![default.to_string()]
        }

        if need_response {
            let request =
                CallRequest::new(caller.clone(), to.clone(), sources, rollback_data, false);

            self.store_call_request(deps.storage, sequence_no, &request)?;
        }

        let call_request = CSMessageRequest::new(
            from,
            to.account(),
            sequence_no,
            need_response,
            data.to_vec(),
            destinations,
        );

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

        let event = event_xcall_message_sent(caller.to_string(), to.to_string(), sequence_no);
        println!("{LOG_PREFIX} Sent Bank Message");
        let mut res = Response::new()
            .add_submessages(submessages)
            .add_attribute("action", "xcall-service")
            .add_attribute("method", "send_packet")
            .add_attribute("sequence_no", sequence_no.to_string())
            .add_event(event);
        if remaining > 0 {
            let msg = BankMsg::Send {
                to_address: fee_handler,
                amount: coins(remaining, config.denom),
            };
            res = res.add_message(msg);
        }

        Ok(res)
    }
}
