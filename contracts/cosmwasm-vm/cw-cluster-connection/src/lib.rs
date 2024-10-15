pub mod contract;
pub mod errors;
pub mod helper;
pub mod msg;
pub mod state;
pub mod types;
pub mod utils;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
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
        ExecuteMsg::SendMessage { to, sn, msg } => conn.send_message(deps, info, to, sn, msg),
        ExecuteMsg::RecvMessage {
            src_network,
            conn_sn,
            msg,
        } => conn.recv_message(deps, info, src_network, conn_sn, msg),
        ExecuteMsg::RecvMessageWithSignatures {
            src_network,
            conn_sn,
            msg,
            account_prefix,
            signatures,
        } => conn.recv_message_with_signatures(
            deps,
            info,
            src_network,
            conn_sn,
            msg,
            account_prefix,
            signatures,
        ),
        ExecuteMsg::ClaimFees {} => conn.claim_fees(deps, env, info),
        ExecuteMsg::RevertMessage { sn } => conn.revert_message(deps, info, sn),
        ExecuteMsg::SetAdmin { address } => conn.set_admin(deps, info, address),
        ExecuteMsg::SetFee {
            network_id,
            message_fee,
            response_fee,
        } => conn.set_fee(deps, info, network_id, message_fee, response_fee),

        ExecuteMsg::SetRelayers { relayers } => conn.set_relayers(deps, info, relayers),
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

        QueryMsg::Admin {} => to_json_binary(&conn.admin().load(deps.storage).unwrap()),

        QueryMsg::GetRelayers {} => {
            let relayers = conn.get_relayers(deps.storage)?;
            let relayers_str: Vec<String> = relayers.iter().map(|addr| addr.to_string()).collect();
            to_json_binary(&relayers_str)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let conn = ClusterConnection::default();
    conn.reply(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let conn = ClusterConnection::default();
    conn.migrate(deps, _env, _msg)
}
