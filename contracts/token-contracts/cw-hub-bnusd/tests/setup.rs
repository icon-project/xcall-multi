use std::collections::HashMap;
use std::str::FromStr;

use cw_common::x_call_msg::XCallMsg as XCallExecuteMsg;
use cw_hub_bnusd::contract::{execute, instantiate, query, reply};
use cw_multi_test::App;
use cw_multi_test::{Contract, ContractWrapper, Executor};
use cw_xcall_ibc_connection::{
    execute as execute_conn, instantiate as instantiate_conn, query as query_conn,
    reply as reply_conn,
};
use cw_xcall_multi::msg::InstantiateMsg as XCallInstantiateMsg;
use cw_xcall_multi::{
    execute as execute_xcall, instantiate as instantiate_xcall, query as query_xcall,
    reply as reply_xcall,
};

use hub_token_msg::InstantiateMsg;

use cosmwasm_std::{Addr, Empty};
use cw_common::{
    hub_token_msg::{self, ExecuteMsg},
    network_address::NetworkAddress,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TestApps {
    XCall,
    HubToken,
    XcallConnection,
    IbcCore,
}

pub struct TestContext {
    pub app: App,
    pub contracts: HashMap<TestApps, Addr>,
    pub sender: Addr,
    pub admin: Option<String>,
    pub caller: Option<String>,
}

impl TestContext {
    pub fn set_xcall_app(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::XCall, addr)
    }

    pub fn set_hubtoken_app(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::HubToken, addr)
    }

    pub fn set_xcall_connection(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::XcallConnection, addr)
    }
    pub fn set_ibc_core(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::IbcCore, addr)
    }
    pub fn get_ibc_core(&self) -> Addr {
        return self.contracts.get(&TestApps::IbcCore).unwrap().clone();
    }

    pub fn get_xcall_app(&self) -> Addr {
        return self.contracts.get(&TestApps::XCall).unwrap().clone();
    }

    pub fn get_xcall_connection(&self) -> Addr {
        return self
            .contracts
            .get(&TestApps::XcallConnection)
            .unwrap()
            .clone();
    }

    pub fn get_hubtoken_app(&self) -> Addr {
        return self.contracts.get(&TestApps::HubToken).unwrap().clone();
    }
}

pub fn setup_context() -> TestContext {
    let router = App::default();
    let sender = Addr::unchecked("sender");
    TestContext {
        app: router,
        contracts: HashMap::new(),
        sender,
        admin: None,
        caller: None,
    }
}

pub fn x_call_contract_setup() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(execute_xcall, instantiate_xcall, query_xcall).with_reply(reply_xcall),
    )
}
pub fn ibc_mock_core_setup() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            cw_mock_ibc_core::contract::execute,
            cw_mock_ibc_core::contract::instantiate,
            cw_mock_ibc_core::contract::query,
        )
        .with_reply(cw_mock_ibc_core::contract::reply),
    )
}
pub fn hub_token_contract_setup() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query).with_reply(reply))
}

pub fn x_call_connection_setup() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(execute_conn, instantiate_conn, query_conn).with_reply(reply_conn),
    )
}

use cosmwasm_std::{Attribute, Event, Uint128};

use cw_common::network_address::NetId;
use cw_multi_test::AppResponse;

pub fn init_x_call(mut ctx: TestContext) -> TestContext {
    let code: Box<dyn Contract<Empty>> = x_call_contract_setup();
    let code_id = ctx.app.store_code(code);

    let _addr = ctx
        .app
        .instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &XCallInstantiateMsg {
                network_id: "icon".to_string(),
                denom: "xcalToken".to_string(),
            },
            &[],
            "XCall",
            None,
        )
        .unwrap();
    ctx.set_xcall_app(_addr);
    ctx
}

pub fn init_mock_ibc_core(mut ctx: TestContext) -> TestContext {
    let code: Box<dyn Contract<Empty>> = ibc_mock_core_setup();
    let code_id = ctx.app.store_code(code);

    let _addr = ctx
        .app
        .instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &cw_mock_ibc_core::msg::InstantiateMsg {},
            &[],
            "IbcCore",
            None,
        )
        .unwrap();
    ctx.set_ibc_core(_addr);
    ctx
}

pub fn init_xcall_connection_contract(mut ctx: TestContext) -> TestContext {
    let connection_contract_code_id = ctx.app.store_code(x_call_connection_setup());
    let connection_contract_addr = ctx
        .app
        .instantiate_contract(
            connection_contract_code_id,
            ctx.sender.clone(),
            &cw_xcall_ibc_connection::msg::InstantiateMsg {
                ibc_host: ctx.get_ibc_core(),
                denom: "uarch".to_string(),
                port_id: "mock".to_string(),
                xcall_address: ctx.get_xcall_app(),
            },
            &[],
            "IBCConnection",
            Some(ctx.sender.clone().to_string()),
        )
        .unwrap();
    ctx.set_xcall_connection(connection_contract_addr);
    ctx
}

pub fn init_token(mut ctx: TestContext, _x_call_address: String) -> TestContext {
    let code: Box<dyn Contract<Empty>> = hub_token_contract_setup();
    let code_id = ctx.app.store_code(code);

    let _addr = ctx
        .app
        .instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &InstantiateMsg {
                x_call: Addr::unchecked(_x_call_address).into_string(),
                hub_address: "icon/cx9876543210fedcba9876543210fedcba98765432".to_owned(),
            },
            &[],
            "HubToken",
            None,
        )
        .unwrap();
    ctx.set_hubtoken_app(_addr);
    ctx
}

pub fn call_set_xcall_host(ctx: &mut TestContext) -> AppResponse {
    ctx.app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_connection(),
            &cw_common_ibc::xcall_connection_msg::ExecuteMsg::SetXCallHost {
                address: ctx.get_xcall_app().to_string(),
            },
            &[],
        )
        .unwrap()
}

pub fn execute_setup(mut ctx: TestContext) -> TestContext {
    let _resp = ctx
        .app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_hubtoken_app(),
            &ExecuteMsg::Setup {
                x_call: Addr::unchecked(ctx.get_xcall_app()),
                hub_address: NetworkAddress::from_str(
                    "icon/cx7866543210fedcba9876543210fedcba987654df",
                )
                .unwrap(),
            },
            &[],
        )
        .unwrap();

    ctx
}

pub fn instantiate_contracts(mut ctx: TestContext) -> TestContext {
    ctx = init_x_call(ctx);
    let x_call_address = ctx.get_xcall_app().into_string();
    ctx = init_token(ctx, x_call_address);
    ctx = init_mock_ibc_core(ctx);
    ctx = init_xcall_connection_contract(ctx);
    ctx
}

pub fn to_attribute_map(attrs: &Vec<Attribute>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in attrs {
        map.insert(attr.key.clone(), attr.value.clone());
    }
    map
}

pub fn get_event(res: &AppResponse, event: &str) -> Option<HashMap<String, String>> {
    let event = res
        .events
        .iter()
        .filter(|e| e.ty == event)
        .collect::<Vec<&Event>>();
    if !event.is_empty() {
        let map = to_attribute_map(&event[0].attributes);
        return Some(map);
    }
    None
}

pub fn mint_token(mut context: TestContext, recipient: String, amount: Uint128) -> TestContext {
    let _response = context
        .app
        .execute_contract(
            context.get_xcall_app(),
            context.get_hubtoken_app(),
            &ExecuteMsg::Mint { recipient, amount },
            &[],
        )
        .unwrap();
    context
}

pub fn set_default_connection(mut context: TestContext, address: Addr) -> TestContext {
    let _response = context
        .app
        .execute_contract(
            context.sender.clone(),
            context.get_xcall_app(),
            &XCallExecuteMsg::SetDefaultConnection {
                nid: NetId::from("icon".to_owned()),
                address,
            },
            &[],
        )
        .unwrap();
    context
}
