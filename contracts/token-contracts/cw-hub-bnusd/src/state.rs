use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const CONNECTED_CHAINS: &'static str = "connected_chains";
pub const SPOKE_CONTRACTS: &'static str = "spoke_contract";
pub const CROSS_CHAIN_SUPPLY: &'static str = "cross_chain_supply";

pub const CROSSCHAINSUPPLY: Map<&String,u128> = Map::new(CROSS_CHAIN_SUPPLY);
pub const CONNECTEDCHAINS: Item<Vec<String>> = Item::new(CONNECTED_CHAINS);

pub const TEST: Map<&Addr,Map<&Addr,u128>> = Map::new("test");


// pub const NAME: Item<String> = Item::new("name");
// pub const SYMBOL: Item<String> = Item::new("symbol");
// pub const DECIMAL: Item<u128> = Item::new("decimal");
pub const OWNER: Item<Addr> = Item::new("owner");
// pub const TOTALSUPPLY: Item<u128> = Item::new("total_supply");

// pub const BALANCES: Map<&Addr,u128> = Map::new("balances");

// const  ZERO_ADDRESSES: &'static str = "0000000000000000000000000000000000000000";

pub const X_CALL : Item<String> = Item::new("xCall");
pub const X_CALL_BTP_ADDRESS : Item<String> = Item::new("xCallBTPAddress");
pub const NID : Item<String> = Item::new("nid");
pub const HUB_ADDRESS : Item<String> = Item::new("hubAddress");
pub const HUB_NET : Item<String> = Item::new("hubNet");

