use std::{collections::HashMap, str::FromStr};

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{ensure_eq, Addr, BalanceResponse, BankQuery, Coin, QueryRequest};
use cw_xcall_lib::network_address::{NetId, NetworkAddress};
use sha2::Digest;
use sha3::Keccak256;

use super::*;

#[cw_serde]
#[derive(QueryResponses)]
pub enum XcallQueryMsg {
    #[returns(String)]
    GetNetworkAddress {},
}

pub fn keccak256(input: &[u8]) -> Keccak256 {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(input);
    hasher
}

impl<'a> ClusterConnection<'a> {
    pub fn ensure_admin(&self, store: &dyn Storage, address: Addr) -> Result<(), ContractError> {
        let admin = self.get_admin(store)?;
        ensure_eq!(admin, address, ContractError::OnlyAdmin);

        Ok(())
    }

    pub fn ensure_relayer(&self, store: &dyn Storage, address: Addr) -> Result<(), ContractError> {
        let relayer = self.get_relayer(store)?;
        ensure_eq!(relayer, address, ContractError::OnlyRelayer);

        Ok(())
    }

    pub fn ensure_xcall(&self, store: &dyn Storage, address: Addr) -> Result<(), ContractError> {
        let xcall = self.get_xcall(store)?;
        ensure_eq!(xcall, address, ContractError::OnlyXCall);

        Ok(())
    }

    pub fn get_amount_for_denom(&self, funds: &Vec<Coin>, target_denom: String) -> u128 {
        for coin in funds.iter() {
            if coin.denom == target_denom {
                return coin.amount.into();
            }
        }
        0
    }

    pub fn get_balance(&self, deps: &DepsMut, env: Env, denom: String) -> u128 {
        let address = env.contract.address.to_string();
        let balance_query = BankQuery::Balance { denom, address };
        let balance_response: BalanceResponse = deps.querier.query(&balance_query.into()).unwrap();

        balance_response.amount.amount.u128()
    }

    pub fn hex_encode(&self, data: Vec<u8>) -> String {
        if data.is_empty() {
            "null".to_string()
        } else {
            hex::encode(data)
        }
    }

    pub fn hex_decode(&self, val: String) -> Result<Vec<u8>, ContractError> {
        let hex_string_trimmed = val.trim_start_matches("0x");
        hex::decode(hex_string_trimmed)
            .map_err(|e| ContractError::InvalidHexData { msg: e.to_string() })
    }

    pub fn get_network_id(&self, deps: Deps) -> Result<String, ContractError> {
        let xcall_host = self.get_xcall(deps.storage)?;

        let query_msg = XcallQueryMsg::GetNetworkAddress {};

        let query_request = QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
            contract_addr: xcall_host.to_string(),
            msg: to_json_binary(&query_msg).map_err(ContractError::Std)?,
        });

        let network_address: String = deps
            .querier
            .query(&query_request)
            .map_err(ContractError::Std)?;

        Ok(NetworkAddress::from_str(network_address.as_str())?
            .nid()
            .to_string())
    }

    pub fn call_xcall_handle_message(
        &self,
        store: &dyn Storage,
        nid: &NetId,
        msg: Vec<u8>,
    ) -> Result<SubMsg, ContractError> {
        let xcall_host = self.get_xcall(store)?;
        let xcall_msg = cw_xcall_lib::xcall_msg::ExecuteMsg::HandleMessage {
            from_nid: nid.clone(),
            msg,
        };
        let call_message: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: xcall_host.to_string(),
            msg: to_json_binary(&xcall_msg).unwrap(),
            funds: vec![],
        });
        let sub_msg: SubMsg = SubMsg::new(call_message);
        Ok(sub_msg)
    }

    pub fn verify_signatures(
        &self,
        deps: Deps,
        threshold: u8,
        signed_msg: Vec<u8>,
        signatures: Vec<Vec<u8>>,
    ) -> Result<(), ContractError> {
        if signatures.len() < threshold.into() {
            return Err(ContractError::InsufficientSignatures);
        }

        let message_hash = keccak256(&signed_msg).finalize().to_vec();

        let mut signers: HashMap<Vec<u8>, bool> = HashMap::new();

        for signature in signatures {
            if signature.len() != 65 {
                return Err(ContractError::InvalidSignature);
            }
            let mut recovery_code = signature[64];
            if recovery_code >= 27 {
                recovery_code -= 27;
            }
            match deps
                .api
                .secp256k1_recover_pubkey(&message_hash, &signature[0..64], recovery_code)
            {
                Ok(pubkey) => {
                    if self.is_validator(deps.storage, pubkey.clone())
                        && !signers.contains_key(&pubkey)
                    {
                        signers.insert(pubkey, true);
                        if signers.len() >= threshold.into() {
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    continue;
                }
            }
        }

        Err(ContractError::InsufficientSignatures)
    }
}
