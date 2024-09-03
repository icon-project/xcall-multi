use soroban_rlp::decoder;
use soroban_sdk::{
    bytes, contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String, Vec,
};

use soroban_xcall_lib::{
    messages::{
        call_message::CallMessage, call_message_persisted::CallMessagePersisted,
        call_message_rollback::CallMessageWithRollback, envelope::Envelope, msg_type::MessageType,
        AnyMessage,
    },
    network_address::NetworkAddress,
};

use crate::{errors::ContractError, helpers, storage, types::Connection};

#[contract]
pub struct MockDapp;

#[contractimpl]
impl MockDapp {
    pub fn init(
        env: Env,
        admin: Address,
        xcall_address: Address,
        native_token: Address,
    ) -> Result<(), ContractError> {
        storage::store_admin(&env, admin);
        storage::store_native_token(&env, native_token);
        storage::store_sn_no(&env, &u128::default());
        storage::store_xcall_address(&env, &xcall_address);

        Ok(())
    }

    pub fn send_call_message(
        env: Env,
        to: NetworkAddress,
        data: Bytes,
        msg_type: u32,
        rollback: Option<Bytes>,
        sender: Address,
    ) -> Result<u128, ContractError> {
        sender.require_auth();

        let network_id = to.nid(&env);
        let message = Self::process_message(msg_type as u8, data, rollback)?;
        let (sources, destinations) = Self::get_network_connections(&env, network_id.clone())?;

        let envelope = Envelope {
            message,
            sources,
            destinations,
        };

        let xcall_address = storage::get_xcall_address(&env)?;
        let res = Self::xcall_send_call(&env, &sender, &to, &envelope, &xcall_address);

        Ok(res)
    }

    pub fn handle_call_message(
        env: Env,
        from: String,
        data: Bytes,
        _protocols: Option<Vec<String>>,
    ) {
        let xcall_address = storage::get_xcall_address(&env)
            .unwrap_or_else(|_| panic_with_error!(&env, ContractError::Uninitialized));

        xcall_address.require_auth();

        let network_from = NetworkAddress::from_string(from);
        let (nid, account) = network_from.parse_network_address(&env);
        if xcall_address.to_string() == account {
            return;
        }

        let msg_data = decoder::decode_string(&env, data);
        if msg_data == String::from_str(&env, "rollback") {
            panic_with_error!(&env, ContractError::RevertFromDapp)
        } else {
            if msg_data == String::from_str(&env, "reply-response") {
                let message = AnyMessage::CallMessage(CallMessage {
                    data: bytes!(&env, 0xabc),
                });

                let (sources, destinations) = Self::get_network_connections(&env, nid)
                    .unwrap_or_else(|error| panic_with_error!(&env, error));

                let envelope = Envelope {
                    message,
                    sources,
                    destinations,
                };
                Self::xcall_send_call(
                    &env,
                    &env.current_contract_address(),
                    &network_from,
                    &envelope,
                    &xcall_address,
                );
            }
        }
    }

    pub fn add_connection(
        env: Env,
        src_endpoint: String,
        dst_endpoint: String,
        network_id: String,
    ) {
        storage::add_new_connection(
            &env,
            network_id,
            Connection::new(src_endpoint, dst_endpoint),
        )
    }

    pub fn get_sequence(env: Env) -> Result<u128, ContractError> {
        storage::get_sn(&env)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    fn process_message(
        message_type: u8,
        data: Bytes,
        rollback: Option<Bytes>,
    ) -> Result<AnyMessage, ContractError> {
        let msg_type: MessageType = message_type.into();

        let message = if msg_type == MessageType::CallMessagePersisted {
            AnyMessage::CallMessagePersisted(CallMessagePersisted { data })
        } else if msg_type == MessageType::CallMessageWithRollback {
            if let Some(rollback) = rollback {
                AnyMessage::CallMessageWithRollback(CallMessageWithRollback { data, rollback })
            } else {
                return Err(ContractError::InvalidRollbackMessage);
            }
        } else {
            AnyMessage::CallMessage(CallMessage { data })
        };

        Ok(message)
    }

    fn get_network_connections(
        env: &Env,
        network_id: String,
    ) -> Result<(Vec<String>, Vec<String>), ContractError> {
        let connections = storage::get_connections(&env, network_id)?;

        let mut sources = Vec::new(&env);
        let mut destinations = Vec::new(&env);
        for conn in connections {
            sources.push_back(conn.src_endpoint);
            destinations.push_back(conn.dst_endpoint);
        }

        Ok((sources, destinations))
    }
}
