#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use cosmwasm_std::{Reply, StdError};
use cw_common::{
    hub_token_msg::ExecuteMsg,
    x_call_msg::{InstantiateMsg, XCallMsg, XCallQuery},
};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:x-call-mock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

const REPLY_MSG_SUCCESS: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: XCallMsg,
) -> Result<Response, ContractError> {
    match msg {
        XCallMsg::SendCallMessage { to, data, rollback } => {
            print!("to: {}", to);
            print!("data: {:?}", data);
            print!("rollback: {:?}", rollback);
            let _network_address = to;
            Ok(Response::default())
        }
        XCallMsg::TestHandleCallMessage {
            from,
            data,
            hub_token,
        } => {
            let call_message = ExecuteMsg::HandleCallMessage { from, data };

            let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: hub_token,
                msg: to_binary(&call_message)?,
                funds: vec![],
            });
            let sub_message = SubMsg::reply_always(wasm_execute_message, REPLY_MSG_SUCCESS);

            Ok(Response::new()
                .add_submessage(sub_message)
                .add_attribute("method", "testhandlecallmessage"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: XCallQuery) -> StdResult<Binary> {
    match _msg {
        XCallQuery::GetNetworkAddress {} => Ok(to_binary(
            "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e",
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_MSG_SUCCESS => reply_msg_success(deps, env, msg),
        _ => Err(ContractError::Std(StdError::generic_err(
            "reply id not found",
        ))),
    }
}

pub fn reply_msg_success(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {}
        cosmwasm_std::SubMsgResult::Err(error) => {
            Err(StdError::GenericErr { msg: error }).map_err(Into::<ContractError>::into)?
        }
    }
    Ok(Response::default())
}

#[cfg(test)]
mod tests {}
