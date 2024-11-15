use soroban_sdk::{Address, Bytes, Env, String};

use crate::{
    connection::GeneralizedConnection, error::ContractError, event,
    interfaces::IGeneralizedConnection, storage, types::*,
};

pub fn cancel_order(env: &Env, id: u128) -> Result<(), ContractError> {
    let order = storage::get_order(&env, id)?;

    let sender = Address::from_string(&order.creator());
    sender.require_auth();

    if order.src_nid() == order.dst_nid() {
        let nid = storage::nid(&env)?;
        resolve_cancel(&env, nid, order.encode(&env))?;

        return Ok(());
    }

    let cancel = Cancel::new(order.encode(&env));
    let order_msg = OrderMessage::new(MessageType::CANCEL, cancel.encode(&env));
    GeneralizedConnection::send_message(&env, order.dst_nid(), order_msg.encode(&env));

    Ok(())
}

pub fn resolve_cancel(
    env: &Env,
    src_network: String,
    order_bytes: Bytes,
) -> Result<(), ContractError> {
    let order = SwapOrder::decode(&env, order_bytes.clone());

    let order_hash = &order.get_hash(&env);
    if storage::order_finished(&env, order_hash) {
        return Ok(());
    }

    if src_network != order.src_nid() {
        return Err(ContractError::InvalidNetwork);
    }

    storage::store_finished_order(&env, order_hash);

    let fill = OrderFill::new(order.id(), order_bytes, order.creator());
    let msg = OrderMessage::new(MessageType::FILL, fill.encode(&env));

    GeneralizedConnection::send_message(&env, order.src_nid(), msg.encode(&env));
    event::order_cancelled(&env, order.id(), src_network);

    Ok(())
}
