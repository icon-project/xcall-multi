use soroban_rlp::decoder;
use soroban_sdk::{
    bytes, contract, contractimpl, panic_with_error, Address, Bytes, Env, String, Vec,
};

use xcall::{
    messages::{
        call_message::CallMessage, call_message_persisted::CallMessagePersisted,
        call_message_rollback::CallMessageWithRollback, envelope::Envelope, AnyMessage,
    },
    types::{message::MessageType, network_address::NetworkAddress},
};

use crate::{errors::ContractError, types::Connection};

#[contract]
pub struct MockDapp;

#[contractimpl]
impl MockDapp {
    pub fn init(env: Env, xcall_address: Address) -> Result<(), ContractError> {
        let sn = u128::default();
        Self::store_sn_no(&env, &sn);
        Self::store_xcall_address(&env, &xcall_address);

        Ok(())
    }

    pub fn send_call_message(
        env: Env,
        to: NetworkAddress,
        data: Bytes,
        msg_type: u32,
        rollback: Option<Bytes>,
        fee: u128,
    ) -> Result<(), ContractError> {
        let network_id = to.nid(&env);
        let message = Self::process_message(msg_type as u8, data, rollback)?;
        let (sources, destinations) = Self::get_network_connections(&env, network_id)?;

        let envelope = Envelope {
            message,
            sources,
            destinations,
        };

        let xcall_address = Self::get_xcall_address(&env)?;
        Self::xcall_send_call(&env, &to, &envelope, &fee, &xcall_address);

        Ok(())
    }

    pub fn handle_call_message(
        env: Env,
        sender: Address,
        from: NetworkAddress,
        data: Bytes,
        _protocols: Option<Vec<String>>,
    ) {
        let (nid, account) = from.parse_network_address(&env);
        if sender.to_string() == account {
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

                let xcall_address = Self::get_xcall_address(&env).ok();
                if xcall_address.is_none() {
                    panic_with_error!(&env, ContractError::Uninitialized)
                }

                let (sources, destinations) = Self::get_network_connections(&env, nid)
                    .unwrap_or_else(|error| panic_with_error!(&env, error));

                let envelope = Envelope {
                    message,
                    sources,
                    destinations,
                };
                Self::xcall_send_call(&env, &from, &envelope, &100_u128, &xcall_address.unwrap());
            }
        }
    }

    pub fn add_connection(
        env: Env,
        src_endpoint: String,
        dst_endpoint: String,
        network_id: String,
    ) {
        Self::add_new_connection(
            &env,
            network_id,
            Connection::new(src_endpoint, dst_endpoint),
        )
    }

    pub fn get_sequence(env: Env) -> Result<u128, ContractError> {
        Self::get_sn(&env)
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
        let connections = Self::get_connections(&env, network_id)?;

        let mut sources = Vec::new(&env);
        let mut destinations = Vec::new(&env);
        for conn in connections {
            sources.push_back(conn.src_endpoint);
            destinations.push_back(conn.dst_endpoint);
        }

        Ok((sources, destinations))
    }
}
