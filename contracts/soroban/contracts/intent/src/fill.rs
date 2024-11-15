use soroban_sdk::{Address, Env, String};

use crate::{
    connection::GeneralizedConnection, error::ContractError, event, helpers,
    interfaces::IGeneralizedConnection, storage, types::*,
};

pub fn fill_order(
    env: &Env,
    order: SwapOrder,
    sender: Address,
    solver_address: String,
) -> Result<(), ContractError> {
    sender.require_auth();

    let order_bytes = order.encode(&env);
    let order_hash = env.crypto().keccak256(&order_bytes);

    if storage::order_finished(&env, &order_hash) {
        return Err(ContractError::OrderAlreadyFilled);
    }
    storage::store_finished_order(&env, &order_hash);

    let protocol_fee = storage::protocol_fee(&env);
    let fee_handler = storage::get_fee_handler(&env)?;
    let to_token = Address::from_string(&order.to_token());
    let to_address = Address::from_string(&order.dst_address());

    let fee = (order.to_amount() * protocol_fee) / 10_000;
    let to_amount = order.to_amount() - fee;

    helpers::transfer_token(&env, &to_token, &sender, &fee_handler, fee);
    helpers::transfer_token(&env, &to_token, &sender, &to_address, to_amount);

    let fill = OrderFill::new(order.id(), order_bytes, solver_address);

    if order.src_nid() == order.dst_nid() {
        let nid = storage::nid(&env)?;
        resolve_fill(&env, nid, fill)?;
        return Ok(());
    }

    let order_msg = OrderMessage::new(MessageType::FILL, fill.encode(&env));
    GeneralizedConnection::send_message(&env, order.src_nid(), order_msg.encode(&env));

    event::order_filled(&env, order.id(), order.src_nid());

    Ok(())
}

pub fn resolve_fill(env: &Env, src_network: String, fill: OrderFill) -> Result<(), ContractError> {
    let order = storage::get_order(&env, fill.id())?;
    if order.get_hash(&env) != env.crypto().keccak256(&fill.order_bytes()) {
        return Err(ContractError::OrderMismatched);
    }

    if src_network != order.dst_nid() {
        return Err(ContractError::InvalidNetwork);
    }

    storage::remove_order(&env, fill.id());
    event::order_closed(&env, fill.id());

    helpers::transfer_token(
        &env,
        &Address::from_string(&order.token()),
        &env.current_contract_address(),
        &Address::from_string(&fill.solver()),
        order.amount(),
    );

    Ok(())
}
