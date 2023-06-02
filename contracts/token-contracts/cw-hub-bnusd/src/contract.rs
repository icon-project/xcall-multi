#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Types};
use crate::state::{HUB_ADDRESS, HUB_NET, NID, OWNER, X_CALL, X_CALL_BTP_ADDRESS};
use bytes::Bytes;

use common::btpAddress::BTPAddress;
use common::icallservice::ICallService;
use common::parseAddress::ParseAddress;

use common::rlpdecode::RLPItem;

use cw20_base::contract::{execute_burn, execute_mint};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

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
    // create initial accounts
    // store token info using cw20-base format
    let data = TokenInfo {
        name: "HubToken".to_string(),
        symbol: "HUBT".to_string(),
        decimals: 18,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: _env.contract.address,
            cap: None,
        }),
    };

    TOKEN_INFO.save(deps.storage, &data)?;

    X_CALL.save(deps.storage, &msg.x_call)?;
    let x_call = X_CALL.load(deps.storage)?;
    // xCallBTPAddress = ICallService::get_btp_address(&x_call, _env)?;
    let (nid, _) = BTPAddress::parse_btp_address(&"aaa")?;
    let (hubNet, hubAddress) = BTPAddress::parse_network_address(&msg.hub_address)?;

    X_CALL_BTP_ADDRESS.save(deps.storage, &"sss".to_string())?;
    NID.save(deps.storage, &nid.to_string())?;
    HUB_ADDRESS.save(deps.storage, &hubAddress.to_string())?;
    HUB_NET.save(deps.storage, &hubNet.to_string())?;
    OWNER.save(deps.storage, &_info.sender)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        Setup {
            _xCall,
            _hubAddress,
        } => execute::setup(deps, _env, info, _xCall, _hubAddress),
        HandleCallMessage { _from, _data } => {
            execute::handle_call_message(deps, _env, info, _from, _data)
        }
        CrossTransfer { to, amount, data } => {
            execute::cross_transfer(deps, _env, info, to, amount, data.into())
        }
        XCrossTransfer {
            from,
            crossTransferData,
        } => execute::x_cross_transfer(deps, _env, info, from, crossTransferData),
        XCrossTransferRevert {
            from,
            crossTransferRevertData,
        } => execute::x_cross_transfer_revert(deps, _env, info, from, crossTransferRevertData),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

mod execute {
    use super::*;

    pub fn setup(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        x_call: Addr,
        hub_address: String,
    ) -> Result<Response, ContractError> {
        X_CALL.save(deps.storage, &x_call)?;
        // xCallBTPAddress = ICallService::get_btp_address(&x_call, _env)?;
        let (nid, _) = BTPAddress::parse_btp_address(&"xCallBTPAddress")?;
        let (hubNet, hubAddress) = BTPAddress::parse_network_address(&hub_address)?;

        X_CALL_BTP_ADDRESS.save(deps.storage, &"xCallBTPAddress".to_string())?;
        NID.save(deps.storage, &nid.to_string())?;
        HUB_ADDRESS.save(deps.storage, &hubAddress.to_string())?;
        HUB_NET.save(deps.storage, &hubNet.to_string())?;
        OWNER.save(deps.storage, &_info.sender)?;
        Ok(Response::default())
    }

    pub fn handle_call_message(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        from: Addr,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        // let mut data = RLPItem::new(data.len(),data.as_ptr()).to_list();
        // let method: String = String::from(data[0].to_bytes()?);

        // match method.as_str() {
        //     "xCrossTransfer" => {
        //         let cross_transfer_data: Types::CrossTransfer =
        //             RLPDecodeStruct::decode_cross_transfer(&data)?;
        //         x_cross_transfer(from, cross_transfer_data)?;
        //     }
        //     "xCrossTransferRevert" => {
        //         let cross_transfer_revert_data: Types::CrossTransferRevert =
        //             RLPDecodeStruct::decode_cross_transfer_revert(&data)?;
        //         x_cross_transfer_revert(from, cross_transfer_revert_data)?;
        //     }
        //     _ => {
        //         return Err(ContractError::InvalidMethod);
        //     }
        // }

        Ok(Response::default())
    }

    pub fn cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to: Addr,
        amount: u128,
        data: Bytes,
    ) -> Result<Response, ContractError> {
        execute_burn(deps, env, info.clone(), amount.into());
        let mut nid = NID.load(deps.storage)?;
        let mut hub_net: String = HUB_NET.load(deps.storage)?;
        let mut hub_address: String = HUB_ADDRESS.load(deps.storage)?;

        let from = BTPAddress::btp_address(&nid, &info.sender.to_string());

        let call_data = Types::CrossTransfer {
            from: from.clone(),
            to: to.to_string().clone(),
            value: amount.clone(),
            data: data.to_vec(),
        };

        let rollback_data = Types::CrossTransferRevert {
            from,
            value: amount,
        };

        let hub_btp_address = BTPAddress::btp_address(&hub_net, &hub_address);
        // let call_message = CallMessage {
        //     address: hub_btp_address,
        //     method: call_data.encode_cross_transfer_message()?,
        //     revert_data: Some(rollback_data.encode_cross_transfer_revert_message()?),
        //     value: info.funds,
        // };

        // let res: Response = deps.querier.query(&call_message.into())?;

        Ok(Response::default())
    }

    pub fn x_cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: Addr,
        cross_transfer_data: Types::CrossTransfer,
    ) -> Result<Response, ContractError> {
        let mut nid = NID.load(deps.storage)?;
        let mut hub_net: String = HUB_NET.load(deps.storage)?;
        let mut hub_address: String = HUB_ADDRESS.load(deps.storage)?;

        let btp_address = BTPAddress::btp_address(&hub_net, &hub_address);

        if from != btp_address {
            return Err(ContractError::Unauthorized {});
        }

        let (net, account) = BTPAddress::parse_network_address(&cross_transfer_data.to)?;
        if net != nid {
            return Err(ContractError::WrongNetwork);
        }

        let to = ParseAddress::parse_address(&account, "Invalid to Address")?;

        let res = execute_mint(deps, env, info, to, cross_transfer_data.value.into())
            .expect("Fail to mint");

        // let res = _mint(deps, to, cross_transfer_data.value)?;

        // TODO Emit log

        Ok(res)
    }

    pub fn x_cross_transfer_revert(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: Addr,
        cross_transfer_revert_data: Types::CrossTransferRevert,
    ) -> Result<Response, ContractError> {
        let mut nid = NID.load(deps.storage)?;
        let mut x_call_btp_address = X_CALL_BTP_ADDRESS.load(deps.storage)?;

        if from != x_call_btp_address {
            return Err(ContractError::OnlyCallService);
        }

        let (net, account) = BTPAddress::parse_network_address(&cross_transfer_revert_data.from)?;
        if net != nid {
            return Err(ContractError::InvalidBTPAddress);
        }

        let to = ParseAddress::parse_address(&account, "Invalid to Address")?;

        let res = execute_mint(deps, env, info, to, cross_transfer_revert_data.value.into())
            .expect("Fail to mint");

        Ok(res)
    }
}

#[cfg(test)]
mod tests {}
