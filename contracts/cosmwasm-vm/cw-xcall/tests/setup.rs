use std::{collections::HashMap, str::FromStr};

use common::{rlp, utils::keccak256};
use cosmwasm::encoding::Binary;
use cosmwasm_std::{
    coins,
    testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
        MOCK_CONTRACT_ADDR,
    },
    to_json_binary, Addr, BlockInfo, ContractInfo, ContractResult, Empty, Env, MessageInfo,
    OwnedDeps, Storage, SystemResult, Timestamp, TransactionInfo, WasmQuery,
};
use cw_xcall::{
    state::CwCallService,
    types::{
        config::Config,
        message::CSMessage,
        request::CSMessageRequest,
        result::{CSMessageResult, CallServiceResponseType},
        rollback::Rollback,
    },
};
use cw_xcall_lib::{
    message::{
        call_message::CallMessage, call_message_rollback::CallMessageWithRollback,
        envelope::Envelope, msg_trait::IMessage, msg_type::MessageType, AnyMessage,
    },
    network_address::{NetId, NetworkAddress},
};

pub fn get_dummy_network_address(nid: &str) -> NetworkAddress {
    NetworkAddress::new(nid, "xcall")
}

pub fn get_dummy_call_msg_envelop() -> Envelope {
    let msg = AnyMessage::CallMessage(CallMessage {
        data: vec![1, 2, 3],
    });
    Envelope::new(msg, vec![], vec![])
}

pub fn get_dummy_req_msg() -> CSMessageRequest {
    CSMessageRequest::new(
        get_dummy_network_address("archway"),
        Addr::unchecked("dapp"),
        1,
        MessageType::CallMessage,
        keccak256(&[1, 2, 3]).to_vec(),
        vec![],
    )
}

pub fn get_dummy_rollback_data() -> Rollback {
    let msg = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
        data: vec![1, 2, 3],
        rollback: vec![1, 2, 3],
    });
    let envelope = Envelope::new(msg, vec![], vec![]);

    Rollback::new(
        Addr::unchecked("xcall"),
        get_dummy_network_address("archway"),
        envelope.sources,
        envelope.message.rollback().unwrap(),
        true,
    )
}

pub fn get_dummy_request_message() -> CSMessage {
    let payload = get_dummy_req_msg();
    CSMessage::new(
        cw_xcall::types::message::CSMessageType::CSMessageRequest,
        rlp::encode(&payload).to_vec(),
    )
}

pub fn get_dummy_result_message() -> CSMessageResult {
    let payload = get_dummy_req_msg();
    CSMessageResult::new(
        1,
        CallServiceResponseType::CallServiceResponseSuccess,
        Some(payload.as_bytes()),
    )
}

pub fn get_dummy_result_message_failure() -> CSMessageResult {
    let payload = get_dummy_req_msg();
    CSMessageResult::new(
        1,
        CallServiceResponseType::CallServiceResponseFailure,
        Some(payload.as_bytes()),
    )
}

pub fn mock_connection_fee_query(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>) {
    deps.querier.update_wasm(|r| match r {
        WasmQuery::Smart {
            contract_addr: _,
            msg: _,
        } => SystemResult::Ok(ContractResult::Ok(to_json_binary(&10_u128).unwrap())),
        _ => todo!(),
    });
}

pub struct MockEnvBuilder {
    env: Env,
}

impl Default for MockEnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEnvBuilder {
    pub fn new() -> MockEnvBuilder {
        MockEnvBuilder {
            env: Env {
                block: BlockInfo {
                    height: 0,
                    time: Timestamp::from_nanos(0),
                    chain_id: "".to_string(),
                },
                transaction: None,
                contract: ContractInfo {
                    address: Addr::unchecked("input"),
                },
            },
        }
    }
    pub fn add_block(mut self, block: BlockInfo) -> MockEnvBuilder {
        self.env.block = block;
        self
    }

    pub fn add_txn_info(mut self, txn_info: Option<TransactionInfo>) -> MockEnvBuilder {
        self.env.transaction = txn_info;
        self
    }

    pub fn add_contract_info(mut self, contract_info: ContractInfo) -> MockEnvBuilder {
        self.env.contract = contract_info;
        self
    }

    pub fn build(self) -> Env {
        Env {
            block: self.env.block,
            transaction: self.env.transaction,
            contract: self.env.contract,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    pub fn create_mock_info(creator: &str, denom: &str, amount: u128) -> MessageInfo {
        let funds = coins(amount, denom);
        mock_info(creator, &funds)
    }

    pub fn deps() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        mock_dependencies()
    }
}

#[test]
fn test() {
    let mock = mock_env();

    let block_info = BlockInfo {
        height: 12_345,
        time: Timestamp::from_nanos(1_571_797_419_879_305_533),
        chain_id: "cosmos-testnet-14002".to_string(),
    };

    let transaction = None;
    let contract = ContractInfo {
        address: Addr::unchecked(MOCK_CONTRACT_ADDR),
    };

    let mock_env_builder: Env = MockEnvBuilder::new()
        .add_block(block_info)
        .add_txn_info(transaction)
        .add_contract_info(contract)
        .build();

    assert_ne!(mock, mock_env_builder)
}

pub struct TestContext {
    pub nid: NetId,
    pub network_address: NetworkAddress,
    pub env: Env,
    pub info: MessageInfo,
    pub request_id: u128,
    pub request_message: Option<CSMessageRequest>,
    pub envelope: Option<Envelope>,
    pub mock_queries: HashMap<Binary, Binary>,
}

impl TestContext {
    pub fn default() -> Self {
        Self {
            nid: NetId::from_str("icon").unwrap(),
            network_address: get_dummy_network_address("icon"),
            env: MockEnvBuilder::new().env,
            info: mock_info("admin", &coins(100, "icx")),
            request_id: u128::default(),
            request_message: Some(get_dummy_req_msg()),
            envelope: None,
            mock_queries: HashMap::<Binary, Binary>::new(),
        }
    }

    pub fn init_context(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        self.store_config(storage, contract);
        self.set_admin(storage, contract);
        self.init_last_request_id(storage, contract);
        self.init_last_sequence_no(storage, contract);
        self.store_default_connection(storage, contract);
        self.store_protocol_fee_handler(storage, contract)
    }

    pub fn init_execute_call(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        self.init_context(storage, contract);
        self.store_proxy_request(storage, contract);
    }

    pub fn init_reply_state(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        self.init_context(storage, contract);
        self.store_proxy_request(storage, contract);
        self.store_execute_request_id(storage, contract);
    }

    pub fn set_admin(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        contract
            .set_admin(storage, self.info.sender.clone())
            .unwrap();
    }

    pub fn store_config(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        let config = Config {
            network_id: "icon".to_string(),
            denom: "icx".to_string(),
        };
        contract.store_config(storage, &config).unwrap();
    }

    pub fn store_protocol_fee_handler(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        contract
            .store_protocol_fee_handler(storage, self.info.sender.to_string())
            .unwrap();
    }

    pub fn store_default_connection(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        let network = get_dummy_network_address("archway");
        let address = Addr::unchecked("centralized");
        contract
            .store_default_connection(storage, network.nid(), address)
            .unwrap();
    }

    pub fn store_proxy_request(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        if let Some(proxy_request) = &self.request_message {
            contract
                .store_proxy_request(storage, self.request_id, proxy_request)
                .unwrap();
        }
    }

    pub fn store_execute_request_id(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        contract
            .store_execute_request_id(storage, self.request_id)
            .unwrap();
    }

    pub fn init_last_request_id(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        contract
            .init_last_request_id(storage, self.request_id)
            .unwrap();
    }

    pub fn init_last_sequence_no(&self, storage: &mut dyn Storage, contract: &CwCallService) {
        contract
            .init_last_sequence_no(storage, u128::default())
            .unwrap();
    }

    pub fn set_successful_response(
        &self,
        storage: &mut dyn Storage,
        contract: &CwCallService,
        sn: u128,
    ) {
        contract.set_successful_response(storage, sn).unwrap();
    }
}
