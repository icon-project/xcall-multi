use cosmwasm_std::Addr;
use cw_common::network_address::{NetId, NetworkAddress};
use cw_storage_plus::{Item, Map};

pub const CONNECTED_CHAINS: &str = "connected_chains";
pub const SPOKE_CONTRACTS: &str = "spoke_contract";
pub const CROSS_CHAIN_SUPPLY: &str = "cross_chain_supply";

pub const CROSSCHAINSUPPLY: Map<&String, u128> = Map::new(CROSS_CHAIN_SUPPLY);
pub const CONNECTEDCHAINS: Item<Vec<String>> = Item::new(CONNECTED_CHAINS);

pub const OWNER: Item<Addr> = Item::new("owner");
pub const X_CALL: Item<Addr> = Item::new("xCall");
pub const X_CALL_NETWORK_ADDRESS: Item<NetworkAddress> = Item::new("xCallBTPAddress");
pub const NID: Item<NetId> = Item::new("nid");
pub const DESTINATION_TOKEN_ADDRESS: Item<Addr> = Item::new("hubAddress");
pub const DESTINATION_TOKEN_NET: Item<NetId> = Item::new("hubNet");
