pub mod contract;
pub mod errors;
pub mod helper;
pub mod msg;
pub mod state;
pub mod types;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult, Storage, SubMsg, WasmMsg,
};

use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
pub use errors::*;
pub use helper::*;
use msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use state::ClusterConnection;
use thiserror::Error;
pub use types::*;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut conn = ClusterConnection::default();
    conn.instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut conn = ClusterConnection::default();
    match msg {
        ExecuteMsg::SetAdmin { address } => conn.set_admin(deps, info, address),

        ExecuteMsg::SetRelayer { address } => conn.set_relayer(deps, info, address),

        ExecuteMsg::SetValidators {
            validators,
            threshold,
        } => conn.set_validators(deps, info, validators, threshold),

        ExecuteMsg::SetSignatureThreshold { threshold } => {
            conn.set_signature_threshold(deps, info, threshold)
        }

        ExecuteMsg::SetFee {
            network_id,
            message_fee,
            response_fee,
        } => conn.set_fee(deps, info, network_id, message_fee, response_fee),

        ExecuteMsg::ClaimFees {} => conn.claim_fees(deps, env, info),

        ExecuteMsg::SendMessage { to, sn, msg } => conn.send_message(deps, info, to, sn, msg),

        ExecuteMsg::RecvMessage {
            src_network,
            conn_sn,
            msg,
            signatures,
        } => conn.recv_message(deps, info, src_network, conn_sn, msg, signatures),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let conn = ClusterConnection::default();
    match msg {
        QueryMsg::GetFee { nid, response } => {
            to_json_binary(&conn.get_fee(deps.storage, nid, response).unwrap())
        }

        QueryMsg::GetReceipt {
            src_network,
            conn_sn,
        } => to_json_binary(&conn.get_receipt(deps.storage, src_network, conn_sn)),

        QueryMsg::GetAdmin {} => {
            let admin = conn.get_admin(deps.storage).unwrap();
            to_json_binary(&admin)
        }

        QueryMsg::GetRelayer {} => {
            let relayer = conn.get_relayer(deps.storage).unwrap();
            to_json_binary(&relayer)
        }

        QueryMsg::GetValidators {} => {
            let validators = conn.get_validators(deps.storage)?;
            let validators_str: Vec<String> =
                validators.iter().map(|addr| addr.to_string()).collect();
            to_json_binary(&validators_str)
        }

        QueryMsg::GetSignatureThreshold {} => {
            let threshold = conn.get_signature_threshold(deps.storage);
            to_json_binary(&threshold)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let conn = ClusterConnection::default();
    conn.migrate(deps, _env, _msg)
}
