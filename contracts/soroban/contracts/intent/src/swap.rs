use soroban_sdk::{Address, Env};

use crate::{error::ContractError, event, helpers, storage, types::*};

pub fn swap_order(env: &Env, order: SwapOrder) -> Result<(), ContractError> {
    let mut order = order;
    let contract_address = env.current_contract_address();
    let sender = Address::from_string(&order.creator());

    sender.require_auth();

    if order.src_nid() != storage::nid(&env)? {
        return Err(ContractError::NetworkIdMisconfigured);
    }
    if order.emitter() != contract_address.to_string() {
        return Err(ContractError::InvalidEmitterAddress);
    }

    let token = Address::from_string(&order.token());
    helpers::transfer_token(&env, &token, &sender, &contract_address, order.amount());

    let deposit_id = storage::increment_deposit_id(&env);
    order.set_id(deposit_id);

    storage::store_order(&env, deposit_id, &order);
    event::swap_intent(
        &env,
        order.id(),
        order.emitter(),
        order.src_nid(),
        order.dst_nid(),
        order.creator(),
        order.dst_address(),
        order.token(),
        order.amount(),
        order.to_token(),
        order.to_amount(),
        order.data(),
    );

    Ok(())
}
