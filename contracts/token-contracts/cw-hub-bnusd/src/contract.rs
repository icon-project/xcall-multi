#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, StdError};
// use cw2::set_contract_version;
use crate::constants::{
    REPLY_MSG_SUCCESS, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    X_CROSS_TRANSFER, X_CROSS_TRANSFER_REVERT,
};
use crate::error::ContractError;
use crate::state::{HUB_ADDRESS, HUB_NET, NID, OWNER, X_CALL, X_CALL_BTP_ADDRESS};
use bytes::Bytes;
use cw_common::hub_token_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_common::x_call_msg::{XCallMsg, XCallQuery};

use cw20_base::contract::{execute_burn, execute_mint};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use cw_common::network_address::NetworkAddress;

use common::rlp::Rlp;

use cw_common::data_types::{CrossTransfer, CrossTransferRevert};

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
    let x_call_addr = deps
        .api
        .addr_validate(&msg.x_call)
        .map_err(ContractError::Std)?;
    let data = TokenInfo {
        name: TOKEN_NAME.to_string(),
        symbol: TOKEN_SYMBOL.to_string(),
        decimals: TOKEN_DECIMALS,
        total_supply: TOKEN_TOTAL_SUPPLY,
        mint: Some(MinterData {
            minter: x_call_addr,
            cap: None,
        }),
    };
    let _save_token = TOKEN_INFO
        .save(deps.storage, &data)
        .map_err(ContractError::Std)?;
    Ok(Response::default())
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
        ExecuteMsg::HandleCallMessage { from, data } => 
            execute::handle_call_message(deps, env, info, from, data),
        ExecuteMsg::CrossTransfer { to, amount, data } => 
            execute::cross_transfer(deps, env, info, to, amount, data.into()),        
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_MSG_SUCCESS => reply_msg_success(deps, env, msg),
        _ => Err(ContractError::InvalidReply),
    }
}

pub fn reply_msg_success(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            Ok(Response::default())
        }
        cosmwasm_std::SubMsgResult::Err(error) => {
            Err(StdError::GenericErr { msg: error }).map_err(Into::<ContractError>::into)
        }
    }
}

mod execute {
    use std::str::from_utf8;

    use bytes::BytesMut;
    use common::rlp::{decode, encode};
    use cosmwasm_std::{to_binary, CosmosMsg, Empty, QueryRequest, SubMsg, WasmQuery};
    use debug_print::debug_println;

    use crate::constants::PROTOCOL;

    use super::*;

    pub fn setup(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        x_call: String,
        hub_address: String,
    ) -> Result<Response, ContractError> {
        deps.api
            .addr_validate(&x_call)
            .map_err(ContractError::Std)?;

        X_CALL
            .save(deps.storage, &x_call)
            .map_err(ContractError::Std)?;

        let query_message = XCallQuery::GetNetworkAddress {};

        let query: QueryRequest<Empty> = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_call.to_string(),
            msg: to_binary(&query_message).map_err(ContractError::Std)?,
        });

        let x_call_btp_address: String = deps.querier.query(&query).map_err(ContractError::Std)?;

        if x_call_btp_address.is_empty() {
            return Err(ContractError::AddressNotFound);
        }
        let (nid, _) = NetworkAddress::parse_network_address(&x_call_btp_address)?;
        let (hub_net, hub_address) = NetworkAddress::parse_network_address(&hub_address)?;

        X_CALL_BTP_ADDRESS.save(deps.storage, &x_call_btp_address.to_string())?;
        NID.save(deps.storage, &nid.to_string())?;
        HUB_ADDRESS.save(deps.storage, &hub_address.to_string())?;
        HUB_NET.save(deps.storage, &hub_net.to_string())?;
        OWNER.save(deps.storage, &info.sender)?;

        Ok(Response::default())
    }

    pub fn handle_call_message(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: String,
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
        debug_println!("this is {:?}", data_list);

        let data_list = &data_list[0].to_vec();
        let method = from_utf8(&data_list).unwrap();
        debug_println!("this is {:?}", method);
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

        Ok(Response::default())
    }

    pub fn cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to: String,
        amount: u128,
        data: Bytes,
    ) -> Result<Response, ContractError> {
        let funds = info.funds.clone();
        let nid = NID.load(deps.storage)?;
        let hub_net: String = HUB_NET.load(deps.storage)?;
        let hub_address: String = HUB_ADDRESS.load(deps.storage)?;

        let from = NetworkAddress::get_network_address(PROTOCOL, &nid, info.sender.as_ref());

        let call_data = CrossTransfer {
            method: X_CROSS_TRANSFER.to_string(),
            from: from.clone(),
            to,
            value: amount,
            data: data.to_vec(),
        };
        let rollback_data = CrossTransferRevert {
            method: X_CROSS_TRANSFER_REVERT.to_string(),
            from,
            value: amount,
        };
        let hub_btp_address = NetworkAddress::get_network_address(PROTOCOL, &hub_net, &hub_address);

        let call_message = XCallMsg::SendCallMessage {
            to: hub_btp_address,
            data: encode(&call_data).to_vec(),
            rollback: Some(encode(&rollback_data).to_vec()),
        };

        let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: X_CALL.load(deps.storage).unwrap(),
            msg: to_binary(&call_message)?,
            funds,
        });

        let sub_message = SubMsg::reply_always(wasm_execute_message, REPLY_MSG_SUCCESS);
        debug_println!("this is {:?}", info.sender);

        let _result =
            execute_burn(deps, env, info, amount.into()).map_err(ContractError::Cw20BaseError)?;

        Ok(Response::new()
            .add_submessage(sub_message)
            .add_attribute("method", "cross_transfer"))
    }

    pub fn x_cross_transfer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: String,
        cross_transfer_data: CrossTransfer,
    ) -> Result<Response, ContractError> {
        let nid = NID.load(deps.storage)?;

        let hub_net: String = HUB_NET.load(deps.storage)?;

        let hub_address: String = HUB_ADDRESS.load(deps.storage)?;
        let btp_address = NetworkAddress::get_network_address(PROTOCOL, &hub_net, &hub_address);

        if from != btp_address {
            return Err(ContractError::WrongAddress {});
        }

        let (net, account) = NetworkAddress::parse_network_address(&cross_transfer_data.to)?;
        debug_println!("this is {:?},{:?}", net, nid);
        if net != nid {
            return Err(ContractError::WrongNetwork);
        }

        deps.api
            .addr_validate(&account)
            .map_err(ContractError::Std)?;

        let res = execute_mint(
            deps,
            env,
            info,
            account.to_string(),
            cross_transfer_data.value.into(),
        )
        .expect("Fail to mint");

        Ok(res
            .add_attribute("method", "x_cross_transfer"))
    }

    pub fn x_cross_transfer_revert(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        from: String,
        cross_transfer_revert_data: CrossTransferRevert,
    ) -> Result<Response, ContractError> {
        let nid = NID.load(deps.storage)?;
        let x_call_btp_address = X_CALL_BTP_ADDRESS.load(deps.storage)?;
        debug_println!("this is {:?}, {:?}", from, x_call_btp_address);

        if from != x_call_btp_address {
            return Err(ContractError::OnlyCallService);
        }

        let (net, account) =
            NetworkAddress::parse_network_address(&cross_transfer_revert_data.from)?;
        debug_println!("this is {:?},{:?}", net, nid);

        if net != nid {
            return Err(ContractError::InvalidBTPAddress);
        }

        deps.api
            .addr_validate(&account)
            .map_err(ContractError::Std)?;

        let res = execute_mint(
            deps,
            env,
            info,
            account.to_string(),
            cross_transfer_revert_data.value.into(),
        )
        .expect("Fail to mint");

        Ok(res)
    }
}

#[cfg(test)]
mod rlp_test {
    use std::str::from_utf8;

    use bytes::BytesMut;
    use common::rlp::{decode, encode, Rlp};
    use cw_common::data_types::CrossTransfer;

    #[test]
    fn encodetest() {
        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            to: "btp://btp/archway123fdth".to_string(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };

        // let mut stream = RlpStream::new();
        let encoded_bytes = encode(&call_data).to_vec();

        // let encoded_data: Vec<u8> = stream.out().to_vec();

        let data: CrossTransfer = decode(&encoded_bytes).unwrap();

        print!("this is {:?}", data);

        let rlp: Rlp = Rlp::new(&encoded_bytes);
        let data: Vec<BytesMut> = rlp.as_list().unwrap();
        let data = &data[0].to_vec();
        let method = from_utf8(&data).unwrap();

        print!("this is {:?}", method)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use common::rlp::encode;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        to_binary, ContractResult, MemoryStorage, OwnedDeps, SystemResult, WasmQuery,
    };

    use super::*;

    fn setup() -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier> = mock_dependencies();
        let env = mock_env();
        let info = mock_info("archway123fdth", &[]);
        let msg = InstantiateMsg {
            x_call: "archway123fdth".to_owned(),
            hub_address: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
        };

        let _res: Response = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        let setup_message = ExecuteMsg::Setup {
            x_call: "archway123fdth".to_owned(),
            hub_address: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
        };
        
        deps.querier.update_wasm(|r| match r {
            WasmQuery::Smart {
                contract_addr: _,
                msg: _,
            } => SystemResult::Ok(ContractResult::Ok(
                to_binary("btp://0x38.bsc/archway192kfvz2vrxv4hhaz3tjdk39maa69xs75n5cea8").unwrap(),
            )),
            _ => todo!(),
        });

        execute(deps.as_mut(), env.clone(), info.clone(), setup_message).unwrap();

        (deps, env, info)
    }

    #[test]
    fn instantiate_test() {
        setup();
    }

    #[test]
    fn execute_handle_call_xcrosstransfer_test() {
        let (mut deps, env, info) = setup();

        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            to: "btp://0x38.bsc/archway123fdth".to_string(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };

        // let mut stream = RlpStream::new();
        let data = encode(&call_data).to_vec();

        let _res: Response = execute(
            deps.as_mut(),
            env,
            info.clone(),
            ExecuteMsg::HandleCallMessage {
                from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
            },
        )
        .unwrap();
    }

    #[test]
    fn cross_transfer_test() {
        let (mut deps, env, info) = setup();

        let call_data = CrossTransfer {
            method: "xCrossTransfer".to_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            to: "btp://0x38.bsc/archway123fdth".to_string(),
            value: 1000,
            data: vec![
                118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
            ],
        };

        // let mut stream = RlpStream::new();
        let data = encode(&call_data).to_vec();

        let _res: Response = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::HandleCallMessage {
                from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
            },
        )
        .unwrap();

        let _res: Response = execute(
            deps.as_mut(),
            env,
            info.clone(),
            ExecuteMsg::CrossTransfer {
                to: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                amount: 1000,
                data: vec![1, 2, 3, 4, 5],
            },
        )
        .unwrap();
    }

    #[test]
    fn execute_handle_call_test_xcrossrevert() {
        let (mut deps, env, info) = setup();

        let call_data = CrossTransferRevert {
            method: "xCrossTransferRevert".to_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            value: 1000,
        };

        // let mut stream = RlpStream::new();
        let data = encode(&call_data).to_vec();

        let _res: Response = execute(
            deps.as_mut(),
            env,
            info.clone(),
            ExecuteMsg::HandleCallMessage {
                from: "btp://0x38.bsc/archway192kfvz2vrxv4hhaz3tjdk39maa69xs75n5cea8".to_owned(),
                data,
            },
        )
        .unwrap();
    }
}
