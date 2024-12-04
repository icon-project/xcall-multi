use common::rlp;
use cosmwasm_std::{coins, Addr, BankMsg, Event, Uint128};
use cw_xcall_lib::network_address::NetId;

use super::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cluster-connection";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_SIGNATURE_THRESHOLD: u8 = 1;

impl<'a> ClusterConnection<'a> {
    pub fn instantiate(
        &mut self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let xcall_address = deps.api.addr_validate(&msg.xcall_address)?;
        self.store_xcall(deps.storage, xcall_address)?;

        self.store_admin(deps.storage, _info.sender)?;

        let relayer = deps.api.addr_validate(&msg.relayer)?;
        self.store_relayer(deps.storage, relayer)?;

        self.store_denom(deps.storage, msg.denom)?;

        let _ = self.store_conn_sn(deps.storage, 0);

        self.store_signature_threshold(deps.storage, DEFAULT_SIGNATURE_THRESHOLD)?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("relayer", msg.relayer)
            .add_attribute("xcall_address", msg.xcall_address))
    }

    pub fn set_admin(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let new_admin = deps.api.addr_validate(address.as_str())?;
        let _ = self.store_admin(deps.storage, new_admin);
        Ok(Response::new().add_attribute("action", "set_admin"))
    }

    pub fn set_relayer(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;
        let new_relayer = deps.api.addr_validate(address.as_str())?;
        let _ = self.store_relayer(deps.storage, new_relayer);
        Ok(Response::new().add_attribute("action", "set_relayer"))
    }

    pub fn set_validators(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        validators: Vec<Vec<u8>>,
        threshold: u8,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;

        if threshold < 1 {
            return Err(ContractError::InvalidThreshold {
                msg: "threshold should be at least 1".to_string(),
            });
        }

        if !validators.is_empty() {
            self.clear_validators(deps.storage)?;
            for rlr in validators {
                self.store_validator(deps.storage, rlr)?;
            }
        }

        let validators_set = self.get_validators(deps.storage)?;

        if validators_set.len() < threshold as usize {
            return Err(ContractError::InvalidThreshold {
                msg: "threshold should be at most the size of validators".to_string(),
            });
        }

        self.store_signature_threshold(deps.storage, threshold)?;

        Ok(Response::new().add_attribute("action", "set_validators"))
    }

    pub fn send_message(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        to: NetId,
        sn: i64,
        msg: Vec<u8>,
    ) -> Result<Response, ContractError> {
        self.ensure_xcall(deps.storage, info.sender)?;

        let next_conn_sn = self.get_next_conn_sn(deps.storage)?;

        let mut fee = 0;

        if sn >= 0 {
            fee = self.get_fee(deps.storage, to.clone(), sn > 0)?.into();
        }

        let value = self.get_amount_for_denom(&info.funds, self.get_denom(deps.storage));

        if fee > value {
            return Err(ContractError::InsufficientFunds);
        }

        Ok(Response::new()
            .add_attribute("action", "send_message")
            .add_event(
                Event::new("Message")
                    .add_attribute("targetNetwork", to.to_string())
                    .add_attribute("connSn", next_conn_sn.to_string())
                    .add_attribute("msg", self.hex_encode(msg)),
            ))
    }

    pub fn set_signature_threshold(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        threshold: u8,
    ) -> Result<Response, ContractError> {
        self.ensure_admin(deps.storage, info.sender)?;

        self.store_signature_threshold(deps.storage, threshold)?;

        Ok(Response::new().add_attribute("action", "set_signature_threshold"))
    }

    pub fn recv_message(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        src_network: NetId,
        conn_sn: u128,
        msg: String,
        dst_network: NetId,
        signatures: Vec<Vec<u8>>,
    ) -> Result<Response, ContractError> {
        self.ensure_relayer(deps.storage, info.sender)?;

        if self.get_receipt(deps.as_ref().storage, src_network.clone(), conn_sn) {
            return Err(ContractError::DuplicateMessage);
        }

        let msg_vec: Vec<u8> = self.hex_decode(msg)?;

        let signed_msg = SignableMsg {
            src_network: src_network.to_string(),
            conn_sn: conn_sn,
            data: msg_vec.clone(),
            dst_network: dst_network.to_string(),
        };
        let signed_msg = rlp::encode(&signed_msg).to_vec();

        let threshold = self.get_signature_threshold(deps.storage);

        self.verify_signatures(deps.as_ref(), threshold, signed_msg, signatures)?;

        self.store_receipt(deps.storage, src_network.clone(), conn_sn)?;

        let xcall_submessage =
            self.call_xcall_handle_message(deps.storage, &src_network, msg_vec)?;

        Ok(Response::new().add_submessage(xcall_submessage))
    }

    pub fn claim_fees(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        self.ensure_relayer(deps.storage, info.sender)?;
        let contract_balance = self.get_balance(&deps, env, self.get_denom(deps.storage));
        let msg = BankMsg::Send {
            to_address: self.get_relayer(deps.storage)?.to_string(),
            amount: coins(contract_balance, self.get_denom(deps.storage)),
        };
        Ok(Response::new()
            .add_attribute("action", "claim fees")
            .add_message(msg))
    }

    pub fn set_fee(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        network_id: NetId,
        message_fee: u128,
        response_fee: u128,
    ) -> Result<Response, ContractError> {
        self.ensure_relayer(deps.storage, info.sender)?;
        self.store_fee(deps.storage, network_id, message_fee, response_fee)?;
        Ok(Response::new().add_attribute("action", "set_fee"))
    }

    pub fn get_fee(
        &self,
        store: &dyn Storage,
        network_id: NetId,
        response: bool,
    ) -> Result<Uint128, ContractError> {
        let mut fee = self.get_message_fee(store, network_id.clone());
        if response {
            fee += self.get_response_fee(store, network_id);
        }
        Ok(fee.into())
    }

    pub fn migrate(
        &self,
        deps: DepsMut,
        _env: Env,
        _msg: MigrateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
            .map_err(ContractError::Std)?;
        Ok(Response::default().add_attribute("migrate", "successful"))
    }
}
