#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use cw_common::{
    x_call_msg::{InstantiateMsg, XCallMsg, XCallQuery},
};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:x-call-mock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

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
    _msg: XCallMsg,
) -> Result<Response, ContractError> {
    match _msg {
        XCallMsg::SendCallMessage { to, data, rollback } => {
            let _network_address = to;
            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: XCallQuery) -> StdResult<Binary> {
    match _msg {
        XCallQuery::GetNetworkAddress { } => {
            Ok(to_binary(
                "btp://0x38.bsc/0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798",
            )?)
        }
    }
}

#[cfg(test)]
mod tests {}
