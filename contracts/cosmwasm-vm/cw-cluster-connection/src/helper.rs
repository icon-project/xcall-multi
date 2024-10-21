use std::collections::HashMap;

use crate::utils::sha256;
use cosmwasm_std::{ensure_eq, Addr, BalanceResponse, BankQuery, Coin};
use cw_xcall_lib::network_address::NetId;
use k256::ecdsa::VerifyingKey;

pub const XCALL_HANDLE_MESSAGE_REPLY_ID: u64 = 1;
pub const XCALL_HANDLE_ERROR_REPLY_ID: u64 = 2;
use super::*;

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
        let sub_msg: SubMsg = SubMsg::reply_always(call_message, XCALL_HANDLE_MESSAGE_REPLY_ID);
        Ok(sub_msg)
    }

    pub fn call_xcall_handle_error(
        &self,
        store: &dyn Storage,
        sn: u128,
    ) -> Result<SubMsg, ContractError> {
        let xcall_host = self.get_xcall(store)?;
        let xcall_msg = cw_xcall_lib::xcall_msg::ExecuteMsg::HandleError {
            sn: sn.try_into().unwrap(),
        };
        let call_message: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: xcall_host.to_string(),
            msg: to_json_binary(&xcall_msg).unwrap(),
            funds: vec![],
        });
        let sub_msg: SubMsg = SubMsg::reply_always(call_message, XCALL_HANDLE_ERROR_REPLY_ID);
        Ok(sub_msg)
    }

    pub fn verify_signatures(
        &self,
        deps: Deps,
        threshold: u8,
        relayers: Vec<String>,
        data: Vec<u8>,
        signatures: Vec<Vec<u8>>,
    ) -> Result<(), ContractError> {
        if signatures.len() < threshold.into() {
            return Err(ContractError::InsufficientSignatures);
        }

        let message_hash = sha256(&data);

        let mut signers: HashMap<String, bool> = HashMap::new();

        for signature in signatures {
            if signature.len() != 65 {
                return Err(ContractError::InvalidSignature);
            }
            match deps
                .api
                .secp256k1_recover_pubkey(&message_hash, &signature[0..64], signature[64])
            {
                Ok(pubkey) => {
                    let pk = VerifyingKey::from_sec1_bytes(&pubkey)
                        .map_err(|_| ContractError::InvalidSignature)?;

                    let pk_hex = hex::encode(pk.to_bytes());
                    if relayers.contains(&pk_hex) && !signers.contains_key(&pk_hex) {
                        signers.insert(pk_hex, true);
                        if signers.len() >= threshold.into() {
                            return Ok(());
                        }
                        break;
                    }
                }
                Err(_) => continue,
            }
        }

        return Err(ContractError::InsufficientSignatures);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    #[test]
    fn test_verify_signatures_simple() {
        let deps = mock_dependencies();
        let connection = ClusterConnection::new();
        let message = b"hello";
        let threshold = 1;
        let relayers =
            vec!["02e5e9769497fbc7c7ee57ab39ccedcb612018577d30ca090033dc67ba5d68b8ab".to_string()];

        let hex_sign = "62249c41d09297800f35174e041ad53ec85c5dcad6a6bd0db3267d36a56eb92d7645b7a64c22ae7e1f93c6c3867d2a33e6534e64093600861916e3299e4cc922";
        let mut signature = hex::decode(hex_sign).expect("Failed to decode hex signature");
        signature.push(1);
        let signatures = vec![signature];

        let result = connection.verify_signatures(
            deps.as_ref(),
            threshold,
            relayers,
            message.to_vec(),
            signatures,
        );

        assert!(result.is_ok());
    }
}
