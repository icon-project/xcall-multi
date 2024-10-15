use cosmwasm_std::Addr;
use cw_xcall_lib::network_address::NetId;

use crate::types::StorageKey;

use super::*;

pub struct ClusterConnection<'a> {
    message_fee: Map<'a, NetId, u128>,
    response_fee: Map<'a, NetId, u128>,
    admin: Item<'a, Addr>,
    conn_sn: Item<'a, u128>,
    receipts: Map<'a, (String, u128), bool>,
    xcall: Item<'a, Addr>,
    denom: Item<'a, String>,
    signature_threshold: Item<'a, u16>,
    relayers: Map<'a, Addr, bool>,
}

impl<'a> Default for ClusterConnection<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ClusterConnection<'a> {
    pub fn new() -> Self {
        Self {
            message_fee: Map::new(StorageKey::MessageFee.as_str()),
            response_fee: Map::new(StorageKey::ResponseFee.as_str()),
            admin: Item::new(StorageKey::Admin.as_str()),
            conn_sn: Item::new(StorageKey::ConnSn.as_str()),
            receipts: Map::new(StorageKey::Receipts.as_str()),
            xcall: Item::new(StorageKey::XCall.as_str()),
            denom: Item::new(StorageKey::Denom.as_str()),
            signature_threshold: Item::new(StorageKey::SignatureThreshold.as_str()),
            relayers: Map::new(StorageKey::Relayers.as_str()),
        }
    }

    pub fn get_next_conn_sn(&self, store: &mut dyn Storage) -> Result<u128, ContractError> {
        let mut connsn = self.conn_sn.load(store).unwrap_or(0);
        connsn += 1;
        self.conn_sn.save(store, &connsn)?;
        Ok(connsn)
    }

    pub fn store_conn_sn(&mut self, store: &mut dyn Storage, sn: u128) -> StdResult<()> {
        self.conn_sn.save(store, &sn)?;
        Ok(())
    }

    pub fn store_fee(
        &mut self,
        store: &mut dyn Storage,
        to: NetId,
        message_fee: u128,
        response_fee: u128,
    ) -> StdResult<()> {
        self.message_fee.save(store, to.clone(), &message_fee)?;
        self.response_fee.save(store, to, &response_fee)?;
        Ok(())
    }
    pub fn query_message_fee(&self, store: &dyn Storage, to: NetId) -> u128 {
        self.message_fee.load(store, to).unwrap_or(0)
    }

    pub fn query_response_fee(&self, store: &dyn Storage, to: NetId) -> u128 {
        self.response_fee.load(store, to).unwrap_or(0)
    }

    pub fn store_receipt(
        &mut self,
        store: &mut dyn Storage,
        src_network: NetId,
        connsn: u128,
    ) -> StdResult<()> {
        self.receipts
            .save(store, (src_network.to_string(), connsn), &true)?;
        Ok(())
    }

    pub fn get_receipt(&self, store: &dyn Storage, src_network: NetId, sn: u128) -> bool {
        self.receipts
            .load(store, (src_network.to_string(), sn))
            .unwrap_or(false)
    }

    pub fn store_xcall(&mut self, store: &mut dyn Storage, address: Addr) -> StdResult<()> {
        self.xcall.save(store, &address)?;
        Ok(())
    }

    pub fn store_admin(&mut self, store: &mut dyn Storage, address: Addr) -> StdResult<()> {
        self.admin.save(store, &address)?;
        Ok(())
    }

    pub fn store_denom(&mut self, store: &mut dyn Storage, denom: String) -> StdResult<()> {
        self.denom.save(store, &denom)?;
        Ok(())
    }

    pub fn query_admin(&self, store: &dyn Storage) -> Result<Addr, ContractError> {
        Ok(self.admin.load(store)?)
    }

    pub fn query_xcall(&self, store: &dyn Storage) -> Result<Addr, ContractError> {
        Ok(self.xcall.load(store)?)
    }
    pub fn denom(&self, store: &dyn Storage) -> String {
        self.denom.load(store).unwrap()
    }
    pub fn admin(&self) -> &Item<'a, Addr> {
        &self.admin
    }

    pub fn store_relayer(&mut self, store: &mut dyn Storage, relayer: Addr) -> StdResult<()> {
        self.relayers.save(store, relayer, &true)?;
        Ok(())
    }

    pub fn remove_relayer(&mut self, store: &mut dyn Storage, relayer: Addr) -> StdResult<()> {
        self.relayers.remove(store, relayer);
        Ok(())
    }

    pub fn clear_relayers(&mut self, store: &mut dyn Storage) -> StdResult<()> {
        self.relayers.clear(store);
        Ok(())
    }

    pub fn get_relayers(&self, store: &dyn Storage) -> StdResult<Vec<Addr>> {
        let mut relayers_list: Vec<Addr> = Vec::new();
        let relayers_iter = self
            .relayers
            .range(store, None, None, cosmwasm_std::Order::Ascending);

        for item in relayers_iter {
            let (relayer_addr, is_active) = item?;
            if is_active {
                relayers_list.push(relayer_addr);
            }
        }

        Ok(relayers_list)
    }

    pub fn store_signature_threshold(
        &mut self,
        store: &mut dyn Storage,
        threshold: u16,
    ) -> StdResult<()> {
        self.signature_threshold.save(store, &threshold)?;
        Ok(())
    }

    pub fn get_signature_threshold(&self, store: &dyn Storage) -> u16 {
        self.signature_threshold.load(store).unwrap()
    }
}
