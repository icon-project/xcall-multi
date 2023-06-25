use cosmwasm_schema::write_api;

use cw_common::x_call_msg::{InstantiateMsg, XCallMsg, XCallQuery};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: XCallMsg,
        query: XCallQuery,
    }
}
