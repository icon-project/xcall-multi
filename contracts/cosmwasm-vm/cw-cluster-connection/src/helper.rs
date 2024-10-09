use std::collections::HashMap;

use crate::utils::{pubkey_to_address, sha256};
use cosmwasm_std::{ensure_eq, Addr, BalanceResponse, BankQuery, Coin};
use cw_xcall_lib::network_address::NetId;
use k256::ecdsa::VerifyingKey;

pub const XCALL_HANDLE_MESSAGE_REPLY_ID: u64 = 1;
pub const XCALL_HANDLE_ERROR_REPLY_ID: u64 = 2;
use super::*;

impl<'a> ClusterConnection<'a> {
    pub fn ensure_admin(&self, store: &dyn Storage, address: Addr) -> Result<(), ContractError> {
        let admin = self.query_admin(store)?;
        ensure_eq!(admin, address, ContractError::OnlyAdmin);

        Ok(())
    }

    pub fn ensure_xcall(&self, store: &dyn Storage, address: Addr) -> Result<(), ContractError> {
        let xcall = self.query_xcall(store)?;
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

    pub fn call_xcall_handle_message(
        &self,
        store: &dyn Storage,
        nid: &NetId,
        msg: Vec<u8>,
    ) -> Result<SubMsg, ContractError> {
        let xcall_host = self.query_xcall(store)?;
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
        let xcall_host = self.query_xcall(store)?;
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
        threshold: u16,
        relayers: Vec<Addr>,
        account_prefix: &str,
        data: Vec<u8>,
        signatures: Vec<Vec<u8>>,
    ) -> Result<(), ContractError> {
        if signatures.len() < threshold.into() {
            return Err(ContractError::InsufficientSignatures);
        }

        let message_hash = sha256(&data);

        let mut signers: HashMap<Addr, bool> = HashMap::new();

        for signature in signatures {
            for recovery_param in 0..2 {
                match deps
                    .api
                    .secp256k1_recover_pubkey(&message_hash, &signature, recovery_param)
                {
                    Ok(pubkey) => {
                        let pk = VerifyingKey::from_sec1_bytes(&pubkey)
                            .map_err(|_| ContractError::InvalidSignature)?;

                        let address = pubkey_to_address(&pk.to_bytes(), account_prefix)?;
                        if relayers.contains(&address) && !signers.contains_key(&address) {
                            signers.insert(address, true);
                            if signers.len() >= threshold.into() {
                                return Ok(());
                            }
                            break;
                        }
                    }
                    Err(_) => continue,
                }
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
        let relayers = vec![Addr::unchecked(
            "archway1a06mhyewfajqcf5dujzyqd6ps9a4v9usauxxqw",
        )];

        let hex_sign = "4ed34a3f55d51615dc1582cba1a86c776157e00e399c53c1e5a128d09325c6684855161da419df8d74e3b8629424f860b7d0a10a2713ce36f4d7a8228cc23838";

        let signature = hex::decode(hex_sign).expect("Failed to decode hex signature");
        let signatures = vec![signature];

        let result = connection.verify_signatures(
            deps.as_ref(),
            threshold,
            relayers,
            "archway",
            message.to_vec(),
            signatures,
        );

        assert!(result.is_ok());
    }
}
