use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_multi_test::App;


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TestApps {
    XCall,
    HubToken,
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

    pub fn get_xcall_app(&self) -> Addr {
        return self.contracts.get(&TestApps::XCall).unwrap().clone();
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

mod instantiate_test {
    use common::rlp::encode;
    use cosmwasm_std::{Addr, Empty};
    use cw_common::{x_call_msg::{self, XCallMsg}, hub_token_msg::{self, ExecuteMsg}, data_types::{CrossTransfer, CrossTransferRevert}};
    use cw_multi_test::{ContractWrapper, Executor, Contract};
    use x_call_mock::contract::{execute, instantiate, query};

    use super::*;

    fn init_x_call(mut ctx: TestContext) -> TestContext {
    
            let code: Box<dyn Contract<Empty>> = 
            Box::new(ContractWrapper::new(execute, instantiate, query).with_reply(x_call_mock::contract::reply));
            let code_id = ctx.app.store_code(code);
    
            let _addr = ctx.app.
            instantiate_contract(code_id, 
                                ctx.sender.clone(), 
                                &x_call_msg::InstantiateMsg{
                                },
                                &[], 
                                "XCall", 
                                None)
                                .unwrap();
        ctx.set_xcall_app(_addr); 
        ctx
    }

fn init_token(mut ctx:TestContext,_x_call_address: String) -> TestContext {
    use hub_token_msg::{InstantiateMsg};
    use cw_hub_bnusd::{contract::{execute, instantiate, query, reply}};

        let code:  Box<dyn Contract<Empty>> = Box::new(ContractWrapper::new(execute, instantiate, query).with_reply(reply));
        let code_id = ctx.app.store_code(code);

        let _addr = ctx.app.
        instantiate_contract(code_id, 
            ctx.sender.clone(), 
                            &InstantiateMsg{
                                x_call: Addr::unchecked(_x_call_address).into_string(),
                                hub_address: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                            },
                            &[], 
                            "HubToken", 
                            None)
                            .unwrap();
        ctx.set_hubtoken_app(_addr);
        ctx
}

fn execute_setup(mut ctx: TestContext) -> TestContext {
    let _resp = ctx.app
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_hubtoken_app(),
                &ExecuteMsg:: Setup {
                    x_call: Addr::unchecked(ctx.get_xcall_app()).into_string(),
                                hub_address: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                },
                &[],
            )
            .unwrap();

    ctx
}

fn handle_call_message(mut ctx: TestContext) -> TestContext {
    let call_data = CrossTransfer {
        method: "xCrossTransfer".to_string(),
        from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
        to: "btp://0x1.icon/archway123fdth".to_string(),
        value: 1000,
        data: vec![118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93],
    };

    // let mut stream = RlpStream::new();
    let data = encode(&call_data).to_vec();
    
    
    let _resp = ctx.app
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_xcall_app(),
                &XCallMsg:: TestHandleCallMessage {
                    from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                    data,
                    hub_token: ctx.get_hubtoken_app().into_string(),
                },
                &[],
            )
            .unwrap();

        let call_data = CrossTransferRevert {
            method: "xCrossTransferRevert".to_string(),
            from: "btp://0x1.icon/".to_owned()+ctx.sender.as_str(),
            value: 1000,
        };

        // let mut stream = RlpStream::new();
        let data = encode(&call_data).to_vec();

        let _resp = ctx.app
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_xcall_app(),
                &XCallMsg:: TestHandleCallMessage {
                    from: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                    data,
                    hub_token: ctx.get_hubtoken_app().into_string(),
                },
                &[],
            )
            .unwrap();  

    ctx
}

fn cross_transfer(mut ctx: TestContext) -> TestContext {
    let _resp = ctx.app
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_hubtoken_app(),
                &ExecuteMsg:: CrossTransfer {
                    to: "archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                    amount: 100,
                    data: vec![],
                },
                &[],
            )
            .unwrap();

    ctx
}

fn setup_contracts(mut ctx: TestContext) -> TestContext{
    ctx = init_x_call(ctx);
    let x_call_address = ctx.get_xcall_app().into_string();
    ctx = init_token(ctx, x_call_address);
    ctx
}

fn setup_test() -> TestContext {
    let mut context: TestContext = setup_context();
    context = setup_contracts(context);
    context = execute_setup(context);
    context = handle_call_message(context);
    context = cross_transfer(context);
    context
}

#[test]
fn contract_test() {
    setup_test();
}

}
