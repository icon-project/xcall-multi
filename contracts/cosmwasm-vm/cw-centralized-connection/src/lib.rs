pub mod contract;
pub mod errors;
pub mod helper;
pub mod msg;
pub mod state;
pub mod types;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult, Storage, SubMsg, WasmMsg,
};

pub use contract::*;
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
pub use errors::*;
pub use helper::*;
use msg::{ExecuteMsg, QueryMsg};
use state::CwCentralizedConnection;
use thiserror::Error;
pub use types::*;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut centralized_connection = CwCentralizedConnection::default();

    centralized_connection.instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut call_service = CwCentralizedConnection::default();
    match msg {
        ExecuteMsg::SendMessage { to, svc, sn, msg } => {
            call_service.send_message(deps, info, to, svc, sn, msg)
        }
        ExecuteMsg::RecvMessage { src_network, conn_sn, msg } => {
            call_service.recv_message(deps, info, src_network, conn_sn, msg)
        }
        ExecuteMsg::ClaimFees {} => call_service.claim_fees(deps, info),
        ExecuteMsg::RevertMessage { sn } => call_service.revert_message(deps, info, sn),
        ExecuteMsg::SetAdmin { address } => {
            call_service.set_admin(deps, info, address)
        }
        ExecuteMsg::SetFee {
            network_id, message_fee, response_fee
        } => call_service.set_fee(deps, info, network_id, message_fee, response_fee),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let call_service = CwCentralizedConnection::default();
    match msg {
        QueryMsg::GetFee { to, response } => {
            to_binary(&call_service.get_fee(deps.storage, to, response).unwrap())
        }

        QueryMsg::GetReceipt { src_network, conn_sn } => {
            to_binary(&call_service.get_receipt(deps.storage, src_network, conn_sn))
        }

        QueryMsg::Admin {} => to_binary(&call_service.admin().load(deps.storage).unwrap()),
    }
}
