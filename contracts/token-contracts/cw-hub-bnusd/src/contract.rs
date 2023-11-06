use std::str::FromStr;

use crate::constants::{
    TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    X_CROSS_TRANSFER, X_CROSS_TRANSFER_REVERT,
};
use crate::error::ContractError;
use crate::state::{
    DESTINATION_TOKEN_ADDRESS, DESTINATION_TOKEN_NET, NID, OWNER, X_CALL, X_CALL_NETWORK_ADDRESS,
};
use cw_common::network_address::IconAddressValidation;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response, StdResult, WasmQuery,
};

use cw2::set_contract_version;
use cw_common::hub_token_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw_common::x_call_msg::{GetNetworkAddress, XCallMsg};

use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_transfer, execute_update_minter, query_download_logo,
    query_marketing_info,
};
use cw20_base::contract::{query_balance, query_minter, query_token_info};
use cw20_base::enumerable::{query_all_accounts, query_owner_allowances, query_spender_allowances};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use cw_common::network_address::NetworkAddress;

use cw_ibc_rlp_lib::rlp::Rlp;
use debug_print::debug_println;

use cw_common::data_types::{CrossTransfer, CrossTransferRevert};
const CONTRACT_NAME: &str = "crates.io:cw-hub-bnusd";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;
    debug_println!("Instantiate cw-hub-bnusd contract...{:?}", msg);

    let x_call_addr = deps
        .api
        .addr_validate(&msg.x_call)
        .map_err(ContractError::Std)?;

    OWNER.save(deps.storage, &info.sender)?;

    let hub_network_address =
        NetworkAddress::from_str(&msg.hub_address).map_err(ContractError::Std)?;

    let token_info = TokenInfo {
        name: TOKEN_NAME.to_string(),
        symbol: TOKEN_SYMBOL.to_string(),
        decimals: TOKEN_DECIMALS,
        total_supply: TOKEN_TOTAL_SUPPLY,
        mint: Some(MinterData {
            minter: x_call_addr.clone(),
            cap: None,
        }),
    };
    setup_function(deps, env, x_call_addr, hub_network_address, token_info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Setup {
            x_call,
            hub_address,
        } => execute::setup(deps, env, info, x_call, hub_address),
        ExecuteMsg::HandleCallMessage { from, data } => {
            execute::handle_call_message(deps, env, info, from, data)
        }
        ExecuteMsg::CrossTransfer { to, amount, data } => {
            execute::cross_transfer(deps, env, info, to, amount, data)
        }
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
                .map_err(ContractError::Cw20BaseError)
        }
        ExecuteMsg::Burn { amount } => {
            execute_burn(deps, env, info, amount).map_err(ContractError::Cw20BaseError)
        }
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires)
            .map_err(ContractError::Cw20BaseError),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires)
            .map_err(ContractError::Cw20BaseError),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount)
            .map_err(ContractError::Cw20BaseError),
        ExecuteMsg::BurnFrom { owner, amount } => {
            execute_burn_from(deps, env, info, owner, amount).map_err(ContractError::Cw20BaseError)
        }
        ExecuteMsg::Mint { recipient, amount } => {
            execute_mint(deps, env, info, recipient, amount).map_err(ContractError::Cw20BaseError)
        }
        ExecuteMsg::UpdateMinter { new_minter } => {
            execute_update_minter(deps, env, info, new_minter).map_err(ContractError::Cw20BaseError)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_binary(&query_spender_allowances(
            deps,
            spender,
            start_after,
            limit,
        )?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    Ok(Response::default().add_attribute("migrate", "successful"))
}


mod execute {
    use std::str::from_utf8;

    use bytes::BytesMut;
    use cosmwasm_std::{to_binary, Addr, CosmosMsg, SubMsg};
    use cw_common::network_address::NetId;
    use cw_ibc_rlp_lib::rlp::{decode, encode};
    use debug_print::debug_println;

    use crate::events::{emit_cross_transfer_event, emit_cross_transfer_revert_event};

    use super::*;

    pub fn setup(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        x_call: Addr,
        hub_network_address: NetworkAddress,
    ) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized);
        }
        let mut token_info = TOKEN_INFO.load(deps.storage)?;
        token_info.mint = Some(MinterData {
            minter: x_call.clone(),
            cap: None,
        });
        deps.api
            .addr_validate(x_call.as_str())
            .map_err(ContractError::Std)?;
        setup_function(deps, env, x_call, hub_network_address, token_info)
    }

    pub fn handle_call_message(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: NetworkAddress,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let xcall = X_CALL.load(deps.storage)?;
        if info.sender != xcall {
            return Err(ContractError::OnlyCallService);
        }

        let rlp: Rlp = Rlp::new(&data);
        let data_list: Vec<BytesMut> = rlp.as_list().unwrap();

        if data_list.len() <= 2 {
            return Err(ContractError::InvalidData);
        }
        debug_println!("datalist {:?}", data_list);

        let data_list = &data_list[0].to_vec();
        let method = from_utf8(data_list).unwrap();
        debug_println!("method {:?}", method);
        match method {
            X_CROSS_TRANSFER => {
                let cross_transfer_data: CrossTransfer = decode(&data).unwrap();
                x_cross_transfer(deps, env, info, from, cross_transfer_data)?;
            }
            X_CROSS_TRANSFER_REVERT => {
                let cross_transfer_revert_data: CrossTransferRevert = decode(&data).unwrap();
                x_cross_transfer_revert(deps, env, info, from, cross_transfer_revert_data)?;
            }
            _ => {
                return Err(ContractError::InvalidMethod);
            }
        }

        Ok(Response::new().add_attribute("action", "handle_call_message"))
    }

    pub fn cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to: NetworkAddress,
        amount: u128,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        if !to.validate_foreign_addresses() {
            return Err(ContractError::InvalidNetworkAddress);
        }
        let funds = info.funds.clone();
        let nid = NID.load(deps.storage)?;
        let hub_net: NetId = DESTINATION_TOKEN_NET.load(deps.storage)?;
        let hub_address: Addr = DESTINATION_TOKEN_ADDRESS.load(deps.storage)?;
        let sender = &info.sender;

        let from = NetworkAddress::new(&nid.to_string(), info.sender.as_ref());

        let call_data = CrossTransfer {
            method: X_CROSS_TRANSFER.to_string(),
            from: from.clone(),
            to: to.clone(),
            value: amount,
            data: data.clone(),
        };
        let rollback_data = CrossTransferRevert {
            method: X_CROSS_TRANSFER_REVERT.to_string(),
            from: sender.clone(),
            value: amount,
        };

        let hub_token_address = NetworkAddress::new(&hub_net.to_string(), hub_address.as_ref());

        let call_message = XCallMsg::SendCallMessage {
            to: hub_token_address,
            data: encode(&call_data).to_vec(),
            rollback: Some(encode(&rollback_data).to_vec()),
            sources: None,
            destinations: None,
        };

        let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: X_CALL.load(deps.storage).unwrap().to_string(),
            msg: to_binary(&call_message)?,
            funds,
        });

        let sub_message = SubMsg::new(wasm_execute_message);
        debug_println!("this is {:?}", info.sender);

        debug_println!("burn from {:?}", sub_message);

        let result =
            execute_burn(deps, env, info, amount.into()).map_err(ContractError::Cw20BaseError)?;
        let event = emit_cross_transfer_event("CrossTransfer".to_string(), from, to, amount, data);

        Ok(result
            .add_submessage(sub_message)
            .add_attribute("method", "cross_transfer")
            .add_event(event))
    }

    pub fn x_cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: NetworkAddress,
        cross_transfer_data: CrossTransfer,
    ) -> Result<Response, ContractError> {
        let nid = NID.load(deps.storage)?;

        let hub_net: NetId = DESTINATION_TOKEN_NET.load(deps.storage)?;

        let destination_network_address: Addr = DESTINATION_TOKEN_ADDRESS.load(deps.storage)?;
        let network_address =
            NetworkAddress::new(&hub_net.to_string(), destination_network_address.as_ref());

        debug_println!("before network addr==from {:?},{:?}", network_address, from);
        if from != network_address {
            return Err(ContractError::WrongAddress {});
        }
        let (net, account) = (
            cross_transfer_data.to.nid(),
            cross_transfer_data.to.account(),
        );
        debug_println!("net nid comparison {:?},{:?}", net, nid);
        if net != nid {
            return Err(ContractError::WrongNetwork);
        }

        deps.api
            .addr_validate(account.as_ref())
            .map_err(ContractError::Std)?;
        debug_println!("mint to {:?}", account);
        let res = execute_mint(
            deps,
            env,
            info,
            account.to_string(),
            cross_transfer_data.value.into(),
        )
        .expect("Fail to mint");

        let event = emit_cross_transfer_event(
            "CrossTransfer".to_string(),
            cross_transfer_data.from,
            cross_transfer_data.to,
            cross_transfer_data.value,
            cross_transfer_data.data,
        );

        Ok(res
            .add_attribute("method", "x_cross_transfer")
            .add_event(event))
    }

    pub fn x_cross_transfer_revert(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: NetworkAddress,
        cross_transfer_revert_data: CrossTransferRevert,
    ) -> Result<Response, ContractError> {
        debug_println!("this is {:?},{:?}", cross_transfer_revert_data, from);
        let xcall_network_address = X_CALL_NETWORK_ADDRESS.load(deps.storage)?;
        if from != xcall_network_address {
            return Err(ContractError::WrongAddress {});
        }

        deps.api
            .addr_validate(cross_transfer_revert_data.from.as_ref())
            .map_err(ContractError::Std)?;

        let res = execute_mint(
            deps,
            env,
            info,
            cross_transfer_revert_data.from.to_string(),
            cross_transfer_revert_data.value.into(),
        )
        .expect("Fail to mint");
        let event = emit_cross_transfer_revert_event(
            "CrossTransferRevert".to_string(),
            cross_transfer_revert_data.from,
            cross_transfer_revert_data.value,
        );
        Ok(res
            .add_attribute("method", "x_cross_transfer_revert")
            .add_event(event))
    }
}

fn setup_function(
    deps: DepsMut,
    _env: Env,
    x_call: Addr,
    hub_network_address: NetworkAddress,
    token_info: TokenInfo,
) -> Result<Response, ContractError> {
    TOKEN_INFO
        .save(deps.storage, &token_info)
        .map_err(ContractError::Std)?;

    if !hub_network_address.validate_foreign_addresses() {
        return Err(ContractError::InvalidNetworkAddress);
    }

    X_CALL
        .save(deps.storage, &x_call)
        .map_err(ContractError::Std)?;

    let query_message = GetNetworkAddress {};

    let query: QueryRequest<Empty> = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: x_call.to_string(),
        msg: to_binary(&query_message).map_err(ContractError::Std)?,
    });

    let x_call_network_address: NetworkAddress =
        deps.querier.query(&query).map_err(ContractError::Std)?;

    if x_call_network_address.to_string().is_empty() {
        return Err(ContractError::AddressNotFound);
    }

    let nid = x_call_network_address.nid();
    let (hub_net, hub_address) = (hub_network_address.nid(), hub_network_address.account());
    debug_println!("setup {:?},{:?},{:?}", hub_net, hub_address, nid);
    X_CALL_NETWORK_ADDRESS.save(deps.storage, &x_call_network_address)?;
    NID.save(deps.storage, &nid)?;
    DESTINATION_TOKEN_ADDRESS.save(deps.storage, &hub_address)?;
    DESTINATION_TOKEN_NET.save(deps.storage, &hub_net)?;
    Ok(Response::default())
}

#[cfg(test)]
mod rlp_test {
    use std::str::{from_utf8, FromStr};

    use bytes::BytesMut;
    use cw_common::{data_types::CrossTransfer, network_address::NetworkAddress};
    use cw_ibc_rlp_lib::rlp::{decode, encode, Rlp};

    #[test]
    fn encode_test() {
        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765432")
                .unwrap(),
            to: NetworkAddress::from_str("archway/archway1ryhtghkyx9kac8m9xl02ac839g4f9qhqkd9slk")
                .unwrap(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };

        let encoded_bytes = encode(&call_data).to_vec();

        let data: CrossTransfer = decode(&encoded_bytes).unwrap();

        print!("this is {:?}", data);

        let rlp: Rlp = Rlp::new(&encoded_bytes);
        let data: Vec<BytesMut> = rlp.as_list().unwrap();
        let data = &data[0].to_vec();
        let method = from_utf8(data).unwrap();

        print!("this is {:?}", method)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        to_binary, Addr, ContractResult, MemoryStorage, OwnedDeps, SystemResult, Uint128,
        WasmQuery,
    };
    use cw_ibc_rlp_lib::rlp::encode;
    use debug_print::debug_println;

    use super::*;

    fn setup(
        sender: &str,
    ) -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier> = mock_dependencies();
        let env = mock_env();
        let info = mock_info(sender, &[]);
        let msg = InstantiateMsg {
            x_call: "archway123fdth".to_owned(),
            hub_address: "0x01.icon/cx9876543210fedcba9876543210fedcba98765432".to_owned(),
        };

        deps.querier.update_wasm(|r| match r {
            WasmQuery::Smart {
                contract_addr: _,
                msg: _,
            } => SystemResult::Ok(ContractResult::Ok(
                to_binary("0x01.icon/cx9876543210fedcba9876543210fedcba98765432").unwrap(),
            )),
            _ => todo!(),
        });

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg);
        debug_println!("res {:?}", res);
        assert!(res.is_ok());

        (deps, env, info)
    }

    #[test]
    fn instantiate_test() {
        setup("archway123fdth");
    }

    #[test]
    fn execute_handle_call_x_cross_transfer_test() {
        let (mut deps, env, info) = setup("archway123fdth");

        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765432")
                .unwrap(),
            to: NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765432")
                .unwrap(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };
        let data = encode(&call_data).to_vec();

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::HandleCallMessage {
                from: NetworkAddress::from_str(
                    "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
                )
                .unwrap(),
                data,
            },
        );
        assert!(res.is_ok());
    }

    #[test]
    fn cross_transfer_test() {
        let (mut deps, env, info) = setup("archway123fdth");

        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765432")
                .unwrap(),
            to: NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765452")
                .unwrap(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };
        let data = encode(&call_data).to_vec();

        let _res: Response = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::HandleCallMessage {
                from: NetworkAddress::from_str(
                    "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
                )
                .unwrap(),
                data,
            },
        )
        .unwrap();

        let info = mock_info("cx9876543210fedcba9876543210fedcba98765452", &[]);

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CrossTransfer {
                to: NetworkAddress::from_str(
                    "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
                )
                .unwrap(),
                amount: 1000,
                data: vec![1, 2, 3, 4, 5],
            },
        );
        debug_println!("this is {:?}", _res);
        assert!(res.is_ok());
    }

    #[test]
    fn change_xcall_address() {
        let (mut deps, env, info) = setup("archway123fdth");

        let balance1: u128 = 1000;
        let balance2: u128 = 200;
        let call_data = ExecuteMsg::Mint {
            recipient: "alice".to_string(),
            amount: Uint128::from(balance1),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), call_data).unwrap();

        let call_data = ExecuteMsg::Mint {
            recipient: "bob".to_string(),
            amount: Uint128::from(balance2),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), call_data).unwrap();

        let alice_balance = query_balance(deps.as_ref(), "alice".to_string()).unwrap();
        let bob_balance = query_balance(deps.as_ref(), "bob".to_string()).unwrap();

        assert_eq!(alice_balance.balance, Uint128::from(balance1));
        assert_eq!(bob_balance.balance, Uint128::from(balance2));

        let total_balance = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(
            total_balance.total_supply,
            Uint128::from(balance1 + balance2)
        );

        let setup_message = ExecuteMsg::Setup {
            x_call: Addr::unchecked("archwayxcalladdress".to_owned()),
            hub_address: NetworkAddress::from_str(
                "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
            )
            .unwrap(),
        };

        deps.querier.update_wasm(|r| match r {
            WasmQuery::Smart {
                contract_addr: _,
                msg: _,
            } => SystemResult::Ok(ContractResult::Ok(
                to_binary("0x01.icon/cx9876543210fedcba9876543210fedcba98765432").unwrap(),
            )),
            _ => todo!(),
        });

        let res = execute(deps.as_mut(), env.clone(), info.clone(), setup_message);
        assert!(res.is_ok());

        let call_data = ExecuteMsg::Mint {
            recipient: "alice".to_string(),
            amount: Uint128::from(balance1),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), call_data);
        assert!(res.is_err());

        let info = mock_info("archwayxcalladdress", &[]);

        let call_data = ExecuteMsg::Mint {
            recipient: "alice".to_string(),
            amount: Uint128::from(balance2),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), call_data).unwrap();

        let alice_balance = query_balance(deps.as_ref(), "alice".to_string()).unwrap();
        assert_eq!(alice_balance.balance, Uint128::from(balance1 + balance2));

        let total_balance = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(
            total_balance.total_supply,
            Uint128::from(balance1 + balance2 + balance2)
        );
    }

    #[test]
    fn execute_handle_call_test_xcross_revert() {
        let (mut deps, env, info) = setup("archway123fdth");

        let call_data = CrossTransferRevert {
            method: "xCrossTransferRevert".to_string(),
            from: Addr::unchecked(
                "0x01.icon/cx9876543210fedcba9876543210fedcba98765432".to_owned(),
            ),
            value: 1000,
        };
        let data = encode(&call_data).to_vec();

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::HandleCallMessage {
                from: NetworkAddress::from_str(
                    "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
                )
                .unwrap(),
                data,
            },
        );
        assert!(res.is_ok());
    }
}
