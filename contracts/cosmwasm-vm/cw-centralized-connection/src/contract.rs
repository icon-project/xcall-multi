use cosmwasm_std::{coins, Addr, BankMsg, Event, Uint128};
use cw_xcall_lib::network_address::NetId;

use super::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-mock-dapp";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl<'a> CwCentralizedConnection<'a> {
    pub fn instantiate(
        &mut self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let relayer = deps.api.addr_validate(&msg.relayer)?;
        self.store_admin(deps.storage, relayer)?;

        let xcall_address = deps.api.addr_validate(&msg.xcall_address)?;
        self.store_xcall(deps.storage, xcall_address)?;
        self.store_denom(deps.storage, msg.denom)?;

        let _ = self.store_conn_sn(deps.storage, 0);

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("relayer", msg.relayer)
            .add_attribute("xcall_address", msg.xcall_address))
    }

    pub fn send_message(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        to: NetId,
        _svc: String,
        sn: i64,
        msg: Vec<u8>,
    ) -> Result<Response, ContractError> {
        self.ensure_xcall(deps.storage, info.sender)?;

        let next_conn_sn = self.get_next_conn_sn(deps.storage)?;
        self.store_conn_sn(deps.storage, next_conn_sn)?;

        let mut fee = 0;

        if sn >= 0 {
            fee = self.get_fee(deps.storage, to.clone(), sn > 0)?.into();
        }

        let value = self.get_amount_for_denom(&info.funds, self.denom(deps.storage));

        if fee > value {
            return Err(ContractError::InsufficientFunds);
        }
        
        Ok(Response::new()
            .add_attribute("action", "send_message")
            .add_event(
                Event::new("Message")
                    .add_attribute("targetNetwork", to.to_string())
                    .add_attribute("connSn", to.to_string())
                    .add_attribute("msg", self.hex_encode(msg)),
            ))
    }

    pub fn recv_message(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        src_network: NetId,
        conn_sn: u128,
        msg: Vec<u8>,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let receipt = self.get_receipt(deps.as_ref().storage, src_network.clone(), conn_sn);
        if receipt {
            return Err(ContractError::DuplicateMessage);
        }
        self.store_receipt(deps.storage, src_network.clone(), conn_sn)?;

        let xcall_submessage = self.call_xcall_handle_message(deps.storage, &src_network, msg)?;

        Ok(Response::new().add_submessage(xcall_submessage))
    }

    pub fn claim_fees(&self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let contract_balance = self.get_balance(&deps, env, self.denom(deps.storage));
        let msg = BankMsg::Send {
            to_address: self.query_admin(deps.storage)?.to_string(),
            amount: coins(contract_balance, self.denom(deps.storage)),
        };
        Ok(Response::new()
            .add_attribute("action", "claim fees")
            .add_message(msg))
    }

    pub fn revert_message(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        sn: u128,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let xcall_submessage = self.call_xcall_handle_error(deps.storage, sn)?;

        Ok(Response::new().add_submessage(xcall_submessage))
    }

    pub fn set_admin(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let admin = deps.api.addr_validate(&address.as_str())?;
        let _ = self.store_admin(deps.storage, admin);
        Ok(Response::new().add_attribute("action", "set_admin"))
    }

    pub fn set_fee(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        network_id: NetId,
        message_fee: u128,
        response_fee: u128,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        self.store_fee(deps.storage, network_id, message_fee, response_fee)?;
        Ok(Response::new().add_attribute("action", "set_fee"))
    }

    pub fn get_fee(
        &self,
        store: &dyn Storage,
        network_id: NetId,
        response: bool,
    ) -> Result<Uint128, ContractError> {
        let mut fee = self.query_message_fee(store, network_id.clone());
        if response {
            fee = fee + self.query_response_fee(store, network_id);
        }
        Ok(fee.into())
    }
}
