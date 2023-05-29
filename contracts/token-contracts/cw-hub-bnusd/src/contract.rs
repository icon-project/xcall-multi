#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::X_CALL;

use common::icallservice::ICallService;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-hub-bnusd";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    X_CALL.save(deps.storage, &msg.xCall)?;
    let x_call = X_CALL.load(deps.storage)?;
    xCallBTPAddress = ICallService::get_btp_address(&xCall);
    let (nid, _) = BTPAddress::parse_btp_address(&xCallBTPAddress)?;
    let (hubNet, hubAddress) = BTPAddress::parse_network_address(&_hubAddress)?;
    OWNER.save(deps.storage, &deps.api.addr_validate(&info.sender).expect("Issue with Transfer"))?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
