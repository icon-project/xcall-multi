use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use cw_common::network_address::{NetId, NetworkAddress};

pub const OWNER: Item<Addr> = Item::new("contract_owner");

pub const SOURCE_XCALL: Item<Addr> = Item::new("source_xcall_address");
pub const X_CALL_NETWORK_ADDRESS: Item<NetworkAddress> = Item::new("source_xcall_network_address");
pub const NID: Item<NetId> = Item::new("network_id");

pub const ICON_ASSET_MANAGER: Item<Addr> = Item::new("icon_asset_manager_network_address");
pub const ICON_NET_ID: Item<NetId> = Item::new("icon_asset_manager_network_id");
