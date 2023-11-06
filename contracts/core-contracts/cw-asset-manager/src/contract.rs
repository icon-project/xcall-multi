use std::str::FromStr;

use cosmwasm_std::{ensure, ensure_eq, entry_point};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, Response,
    StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{AllowanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use cw_common::asset_manager_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw_common::network_address::IconAddressValidation;
use cw_common::network_address::NetworkAddress;
use cw_common::x_call_msg::{GetNetworkAddress, XCallMsg};
use cw_common::xcall_data_types::Deposit;

use crate::constants::SUCCESS_REPLY_MSG;
use crate::contract::exec::setup;
use crate::error::ContractError;
use crate::helpers::{decode_encoded_bytes, is_contract, DecodedStruct};
use crate::state::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;
    OWNER.save(deps.storage, &info.sender)?;

    setup(deps, msg.source_xcall, msg.destination_asset_manager)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ConfigureXcall {
            source_xcall,
            destination_asset_manager,
        } => {
            let owner = OWNER.load(deps.storage).map_err(ContractError::Std)?;
            ensure_eq!(owner, info.sender, ContractError::OnlyOwner);
            setup(deps, source_xcall, destination_asset_manager)
        }
        ExecuteMsg::HandleCallMessage { from, data } => {
            exec::handle_xcall_msg(deps, env, info, from, data)
        }
        ExecuteMsg::ConfigureNative {
            native_token_address,
            native_token_manager,
        } => {
            let owner = OWNER.load(deps.storage).map_err(ContractError::Std)?;
            ensure_eq!(owner, info.sender, ContractError::OnlyOwner);
            exec::setup_native_token(deps, native_token_address, native_token_manager)
        }
        ExecuteMsg::Deposit {
            token_address,
            amount,
            to,
            data,
        } => {
            let nid = NID.load(deps.storage)?;
            let depositor = NetworkAddress::new(nid.as_str(), info.sender.as_str());

            // Performing necessary validation and logic for the Deposit variant
            let token = deps.api.addr_validate(token_address.as_ref())?;
            ensure!(
                is_contract(deps.querier, &token),
                ContractError::InvalidTokenAddress
            );
            ensure!(!amount.is_zero(), ContractError::InvalidAmount);

            let recipient: NetworkAddress = match to {
                Some(to_address) => {
                    let nw_addr = NetworkAddress::from_str(&to_address).unwrap();
                    if !nw_addr.validate_foreign_addresses() {
                        return Err(ContractError::InvalidRecipientAddress);
                    }
                    nw_addr
                }
                // if `to` is not provided, sender address is used as recipient
                None => depositor,
            };

            let data = data.unwrap_or_default();

            let res = exec::deposit_cw20_tokens(
                deps,
                env,
                token_address,
                info.sender.clone(),
                amount,
                recipient,
                data,
                info,
            )?;
            Ok(res)
        }
    }
}

mod exec {
    use std::str::FromStr;

    use cosmwasm_std::CosmosMsg;
    use cw_ibc_rlp_lib::rlp::Encodable;

    use cw_common::xcall_data_types::DepositRevert;

    use super::*;

    fn query_network_address(
        deps: &DepsMut,
        x_call_addr: &Addr,
    ) -> Result<NetworkAddress, ContractError> {
        let query_msg = GetNetworkAddress {};
        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_call_addr.to_string(),
            msg: to_binary(&query_msg).map_err(ContractError::Std)?,
        });

        deps.querier.query(&query).map_err(ContractError::Std)
    }

    pub fn setup(
        deps: DepsMut,
        source_xcall: String,
        destination_asset_manager: String,
    ) -> Result<Response, ContractError> {
        // validate source xcall
        let x_call_addr = deps
            .api
            .addr_validate(&source_xcall)
            .map_err(ContractError::Std)?;

        let xcall_network_address: NetworkAddress = query_network_address(&deps, &x_call_addr)?;

        if xcall_network_address.to_string().is_empty() {
            return Err(ContractError::XAddressNotFound);
        }

        // Obtain native network id
        let nid = xcall_network_address.nid();

        // validate icon asset manager
        let icon_asset_manager =
            NetworkAddress::from_str(&destination_asset_manager).map_err(ContractError::Std)?;
        if !icon_asset_manager.validate_foreign_addresses() {
            return Err(ContractError::InvalidNetworkAddressFormat);
        }

        //update state
        SOURCE_XCALL
            .save(deps.storage, &x_call_addr)
            .map_err(ContractError::Std)?;
        X_CALL_NETWORK_ADDRESS.save(deps.storage, &xcall_network_address)?;
        NID.save(deps.storage, &nid)?;
        ICON_ASSET_MANAGER.save(deps.storage, &icon_asset_manager)?;
        ICON_NET_ID.save(deps.storage, &icon_asset_manager.nid())?;

        Ok(Response::default())
    }

    pub fn setup_native_token(
        deps: DepsMut,
        native_token_address: String,
        native_token_manager: String,
    ) -> Result<Response, ContractError> {
        let token_addr = deps
            .api
            .addr_validate(&native_token_address)
            .map_err(ContractError::Std)?;
        let token_manager_addr = deps
            .api
            .addr_validate(&native_token_manager)
            .map_err(ContractError::Std)?;
        NATIVE_TOKEN_ADDRESS.save(deps.storage, &token_addr)?;
        NATIVE_TOKEN_MANAGER.save(deps.storage, &token_manager_addr)?;

        Ok(Response::default())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn deposit_cw20_tokens(
        deps: DepsMut,
        env: Env,
        token_address: String,
        from: Addr,
        amount: Uint128,
        to: NetworkAddress,
        data: Vec<u8>,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let dest_am = ICON_ASSET_MANAGER.load(deps.storage)?;

        let contract_address = &env.contract.address;

        let query_msg = &Cw20QueryMsg::Allowance {
            owner: from.to_string(),
            spender: contract_address.to_string(),
        };

        let query_resp: AllowanceResponse = deps
            .querier
            .query_wasm_smart::<AllowanceResponse>(token_address.clone(), &query_msg)?;

        //check allowance
        ensure!(
            query_resp.allowance >= amount,
            ContractError::InsufficientTokenAllowance
        );

        let transfer_token_msg = to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: from.to_string(),
            recipient: contract_address.into(),
            amount,
        })?;

        let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_owned(),
            msg: transfer_token_msg,
            funds: vec![],
        });

        //transfer sub msg
        let transfer_sub_msg = SubMsg {
            id: SUCCESS_REPLY_MSG,
            msg: execute_msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };

        //create xcall rlp encode data
        let xcall_data = Deposit {
            token_address: token_address.to_owned(),
            from: from.to_string(),
            to: to.to_string(),
            amount: Uint128::u128(&amount),
            data,
        };

        let source_xcall = SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  send call
        let xcall_message = XCallMsg::SendCallMessage {
            to: dest_am.to_string().parse()?,
            data: xcall_data.rlp_bytes().to_vec(),
            rollback: Some(
                DepositRevert {
                    token_address: token_address.to_owned(),
                    account: from.to_string(),
                    amount: Uint128::u128(&amount),
                }
                .rlp_bytes()
                .to_vec(),
            ),
            sources: None,
            destinations: None,
        };

        let xcall_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: source_xcall.to_string(),
            msg: to_binary(&xcall_message)?,
            funds: info.funds,
        });

        let xcall_sub_msg = SubMsg {
            id: SUCCESS_REPLY_MSG,
            msg: xcall_msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };

        let attributes = vec![
            ("Token", token_address),
            ("To", to.to_string()),
            ("Amount", amount.to_string()),
        ];

        let event = Event::new("Deposit").add_attributes(attributes);

        let resp = Response::new()
            .add_submessages(vec![transfer_sub_msg, xcall_sub_msg])
            .add_event(event);

        Ok(resp)
    }

    pub fn handle_xcall_msg(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        from: String,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let x_call_addr = SOURCE_XCALL.load(deps.storage)?;
        let x_network = X_CALL_NETWORK_ADDRESS.load(deps.storage)?;

        if info.sender != x_call_addr {
            return Err(ContractError::OnlyXcallService);
        }

        let (_, decoded_struct) = decode_encoded_bytes(&data)?;

        let res = match decoded_struct {
            DecodedStruct::DepositRevert(data) => {
                if from != x_network.to_string() {
                    return Err(ContractError::FailedXcallNetworkMatch);
                }

                let token_address = data.token_address;
                let account = data.account;
                let amount = Uint128::from(data.amount);

                transfer_tokens(deps, account, token_address, amount)?
            }

            DecodedStruct::WithdrawTo(data_struct) => {
                let icon_am = ICON_ASSET_MANAGER.load(deps.storage)?;
                if from != icon_am.to_string() {
                    return Err(ContractError::OnlyIconAssetManager {});
                }

                let token_address = data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::from(data_struct.amount);

                transfer_tokens(deps, account, token_address, amount)?
            }

            DecodedStruct::WithdrawNativeTo(data_struct) => {
                let icon_am = ICON_ASSET_MANAGER.load(deps.storage)?;
                if from != icon_am.to_string() {
                    return Err(ContractError::OnlyIconAssetManager {});
                }

                let token_address = data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::from(data_struct.amount);

                swap_to_native(deps, account, token_address, amount)?
            }
        };

        Ok(res)
    }

    //internal function to transfer tokens from contract to account
    fn transfer_tokens(
        deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        deps.api.addr_validate(&account)?;
        deps.api.addr_validate(&token_address)?;

        let transfer_msg = &Cw20ExecuteMsg::Transfer {
            recipient: account,
            amount,
        };

        let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address,
            msg: to_binary(transfer_msg)?,
            funds: vec![],
        });

        let sub_msg = SubMsg {
            id: SUCCESS_REPLY_MSG,
            msg: execute_msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };
        Ok(Response::new().add_submessage(sub_msg))
    }

    #[cfg(feature = "archway")]
    fn swap_to_native(
        deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        use crate::external::{ConfigResponse, Cw20HookMsg, StakingQueryMsg};

        deps.api.addr_validate(&account)?;
        deps.api.addr_validate(&token_address)?;
        let query_msg = &StakingQueryMsg::ConfigInfo {};
        let manager = NATIVE_TOKEN_MANAGER.load(deps.storage)?;
        let query_resp: ConfigResponse = deps
            .querier
            .query_wasm_smart::<ConfigResponse>(manager.clone(), &query_msg)?;
        let swap_contract = query_resp.swap_contract_addr;

        let hook = &Cw20HookMsg::Swap {
            belief_price: None,
            max_spread: None,
            to: Some(account.clone()),
        };
        let transfer_msg = &Cw20ExecuteMsg::Send {
            contract: swap_contract.clone(),
            amount,
            msg: to_binary(hook)?,
        };

        let execute_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.clone(),
            msg: to_binary(transfer_msg)?,
            funds: vec![],
        });

        let sub_msg = SubMsg {
            id: SUCCESS_REPLY_MSG,
            msg: execute_msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };
        Ok(Response::new().add_submessage(sub_msg))
    }

    #[cfg(not(any(feature = "archway")))]
    fn swap_to_native(
        deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        transfer_tokens(deps, account, token_address, amount)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    Ok(Response::default().add_attribute("migrate", "successful"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query::query_get_owner(deps)?),
        QueryMsg::GetConfiguration {} => to_binary(&query::query_config(deps)?),
        QueryMsg::GetNetIds {} => to_binary(&query::query_nid(deps)?),
    }
}

mod query {
    use cw_common::asset_manager_msg::{ConfigureResponse, NetIdResponse, OwnerResponse};

    use super::*;

    pub fn query_get_owner(deps: Deps) -> StdResult<OwnerResponse> {
        let owner = OWNER.load(deps.storage)?;
        Ok(OwnerResponse { owner })
    }

    pub fn query_config(deps: Deps) -> StdResult<ConfigureResponse> {
        let source_x_call = SOURCE_XCALL.load(deps.storage)?;
        let source_xcall = Addr::unchecked(source_x_call);
        let icon_asset_manager = (ICON_ASSET_MANAGER.load(deps.storage)?).to_string();

        Ok(ConfigureResponse {
            source_xcall,
            icon_asset_manager,
        })
    }

    pub fn query_nid(deps: Deps) -> StdResult<NetIdResponse> {
        let x_call_nid = NID.load(deps.storage)?.to_string();
        let icon_nid = ICON_NET_ID.load(deps.storage)?.to_string();

        Ok(NetIdResponse {
            x_call_nid,
            icon_nid,
        })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        ContractInfoResponse, ContractResult, MemoryStorage, OwnedDeps, SystemResult, Uint128,
        WasmQuery,
    };
    use cw_ibc_rlp_lib::rlp::Encodable;

    use cw_common::xcall_data_types::DepositRevert;
    use cw_common::{asset_manager_msg::InstantiateMsg, xcall_data_types::WithdrawTo};

    use super::*;

    //similar to fixtures
    fn test_setup() -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        Response,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("user", &[]);
        //to pretend us as xcall contract during handle call execution testing
        let xcall = "xcall";

        // mocking response for external query i.e. allowance
        deps.querier.update_wasm(|r: &WasmQuery| match r {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } => {
                if contract_addr == &xcall.to_owned() {
                    SystemResult::Ok(ContractResult::Ok(
                        to_binary(&"0x44.archway/xcall".to_owned()).unwrap(),
                    ))
                } else {
                    //mock allowance resp
                    let allowance_resp = AllowanceResponse {
                        allowance: Uint128::new(1000),
                        expires: cw_utils::Expiration::Never {},
                    };
                    SystemResult::Ok(ContractResult::Ok(to_binary(&allowance_resp).unwrap()))
                }
            }
            WasmQuery::ContractInfo { contract_addr: _ } => {
                let mut response = ContractInfoResponse::default();
                response.code_id = 1;
                response.creator = "sender".to_string();
                SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
            }
            _ => todo!(),
        });

        let instantiated_resp = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                source_xcall: xcall.to_owned(),
                destination_asset_manager: "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c"
                    .to_owned(),
            },
        )
        .unwrap();

        (deps, env, info, instantiated_resp)
    }

    #[test]
    fn test_instantiate() {
        let (deps, _, info, res) = test_setup();

        //check proper instantiation
        assert_eq!(res.attributes.len(), 0);

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, info.sender);
    }

    #[test]
    fn test_deposit_for_sufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message (checking expected field value)
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(100),
            to: None,
            data: None,
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Verify the response contains the expected sub-messages
        assert_eq!(response.messages.len(), 2);

        // Verify the event attributes
        if let Some(event) = response.events.get(0) {
            assert_eq!(event.ty, "Deposit");
            assert_eq!(event.attributes.len(), 3);

            // Verify the individual event attributes
            for attribute in &event.attributes {
                match attribute.key.as_str() {
                    "Token" => assert_eq!(attribute.value, "token1"),
                    "To" => assert_eq!(attribute.value, "0x44.archway/user"),
                    "Amount" => assert_eq!(attribute.value, "100"),
                    _ => panic!("Unexpected attribute key"),
                }
            }
        } else {
            panic!("No event found in the response");
        }

        //check for some address for to field
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(100),
            to: Some(String::from(
                "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
            )),
            data: None,
        };

        let result = execute(deps.as_mut(), env, info, msg).unwrap();
        for attribute in &result.events[0].attributes {
            match attribute.key.as_str() {
                "Token" => assert_eq!(attribute.value, "token1"),
                "To" => println!("value: {:?}", attribute.value),
                "Amount" => assert_eq!(attribute.value, "100"),
                _ => panic!("Unexpected attribute key"),
            }
        }
    }

    #[test]
    fn test_deposit_for_insufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(1500),
            to: None,
            data: None,
        };

        let result = execute(deps.as_mut(), env, info, msg);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn test_deposit_for_invalid_zero_amount() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(0),
            to: None,
            data: None,
        };

        execute(deps.as_mut(), env, info, msg).unwrap();
    }

    #[test]
    fn test_handle_xcall() {
        let (mut deps, env, _, _) = test_setup();
        let mocked_xcall_info = mock_info("xcall", &[]);

        let xcall_nw = "0x44.archway/xcall";
        let token = "token1";
        let account = "account1";
        //create deposit revert(expected)  xcall msg deps
        let x_deposit_revert = DepositRevert {
            token_address: token.to_string(),
            account: account.to_string(),
            amount: 100,
        };

        //create valid handle_call_message
        let msg = ExecuteMsg::HandleCallMessage {
            from: xcall_nw.to_string(),
            data: x_deposit_revert.rlp_bytes().to_vec(),
        };

        let result = execute(deps.as_mut(), env.clone(), mocked_xcall_info.clone(), msg);

        //check for valid xcall expected msg data

        assert!(result.is_ok());

        //for withdrawTo
        let am_nw = "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c";
        let withdraw_msg = WithdrawTo {
            token_address: token.to_string(),
            amount: 1000,
            user_address: account.to_string(),
        };

        let exe_msg = ExecuteMsg::HandleCallMessage {
            from: am_nw.to_string(),
            data: withdraw_msg.rlp_bytes().to_vec(),
        };
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mocked_xcall_info.clone(),
            exe_msg,
        );
        assert!(resp.is_ok());

        //----------------------------------------------//
        //check for unhandled xcall msg data
        //----------------------------------------------//

        let x_msg = Deposit {
            token_address: String::from("token1"),
            from: String::from("user"),
            amount: 100,
            to: String::from("account1"),
            data: vec![],
        };

        let unknown_msg = ExecuteMsg::HandleCallMessage {
            from: xcall_nw.to_string(),
            data: x_msg.rlp_bytes().to_vec(),
        };

        //check for error due to unknown xcall handle data
        let result = execute(deps.as_mut(), env, mocked_xcall_info, unknown_msg);
        assert!(result.is_err());
    }

    #[cfg(feature = "archway")]
    #[test]
    fn test_withdraw_native_archway() {
        use cw_common::xcall_data_types::WithdrawNativeTo;

        use crate::external::ConfigResponse;

        let (mut deps, env, info, _) = test_setup();
        let mocked_xcall_info = mock_info("xcall", &[]);

        let staking = "staking";
        let swap = "swap";
        let token = "token1";
        let account = "account1";

        deps.querier.update_wasm(|r: &WasmQuery| match r {
            WasmQuery::Smart {
                contract_addr: _,
                msg: _,
            } => SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    admin: "".to_string(),
                    pause_admin: "".to_string(),
                    bond_denom: "".to_string(),
                    liquid_token_addr: "".to_string(),
                    swap_contract_addr: swap.to_string(),
                    treasury_contract_addr: "".to_string(),
                    team_wallet_addr: "".to_string(),
                    commission_percentage: 1,
                    team_percentage: 1,
                    liquidity_percentage: 1,
                    delegations: vec![],
                    contract_state: false,
                })
                .unwrap(),
            )),
            _ => todo!(),
        });

        let resp = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::ConfigureNative {
                native_token_address: token.to_string(),
                native_token_manager: staking.to_string(),
            },
        );
        assert!(resp.is_ok());

        let am_nw = "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c";
        let withdraw_msg = WithdrawNativeTo {
            token_address: token.to_string(),
            amount: 1000,
            user_address: account.to_string(),
        };

        let exe_msg = ExecuteMsg::HandleCallMessage {
            from: am_nw.to_string(),
            data: withdraw_msg.rlp_bytes().to_vec(),
        };
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mocked_xcall_info.clone(),
            exe_msg,
        );
        assert!(resp.is_ok());
    }

    #[test]
    fn test_configure_network() {
        //verify configuration updates from owner side
        let (mut deps, env, info, _) = test_setup();

        let source_xcall = "xcall".to_string();
        let destination_asset_manager =
            "0x01.icon/hx9876543210fedcba9876543210fedcba98765432".to_string();
        // Execute the function
        let msg = ExecuteMsg::ConfigureXcall {
            source_xcall: source_xcall.to_owned(),
            destination_asset_manager: destination_asset_manager.to_owned(),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

        // Check the response
        assert!(res.is_ok());
        let response: Response = res.unwrap();
        assert_eq!(response, Response::default());

        // Verify the saved values
        let saved_source_xcall: String = SOURCE_XCALL
            .load(deps.as_ref().storage)
            .unwrap()
            .to_string();
        let icon_am = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        let saved_destination_asset_manager = icon_am.to_string();

        assert_eq!(saved_source_xcall, source_xcall);
        assert_eq!(saved_destination_asset_manager, destination_asset_manager);

        // Verify that only the owner can configure the network
        let other_info = mock_info("other_sender", &[]);
        let res = execute(deps.as_mut(), env, other_info, msg);

        //check for error
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err, ContractError::OnlyOwner);
    }
}
