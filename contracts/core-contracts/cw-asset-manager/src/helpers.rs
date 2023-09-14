use cosmwasm_std::{Addr, QuerierWrapper};
use cw_ibc_rlp_lib::rlp::{DecoderError, Rlp};

use cw_common::xcall_data_types::{DepositRevert, WithdrawTo};

use crate::error::ContractError;

#[derive(Debug)]
pub enum DecodedStruct {
    WithdrawTo(WithdrawTo),
    DepositRevert(DepositRevert),
}

pub fn decode_encoded_bytes(data: &[u8]) -> Result<(&str, DecodedStruct), ContractError> {
    // Decode RLP bytes into an Rlp object
    let rlp = Rlp::new(data);

    if !rlp.is_list() {
        return Err(DecoderError::RlpExpectedToBeList.into());
    }

    // Extract method name
    let method: String = rlp.val_at(0).unwrap();

    // Convert method: String -> &str
    match method.as_str() {
        "WithdrawTo" => {
            if rlp.item_count()? != 4 {
                return Err(DecoderError::RlpInvalidLength.into());
            }

            // Extract the fields
            let token: String = rlp.val_at(1)?;
            let user_address: String = rlp.val_at(2)?;
            let amount: u128 = rlp.val_at(3)?;

            // Create a new WithdrawTo instance
            let withdraw_to = WithdrawTo {
                token_address: token,
                user_address,
                amount,
            };

            // Return the decoded struct as an OK variant
            Ok(("WithdrawTo", DecodedStruct::WithdrawTo(withdraw_to)))
        }

        "DepositRevert" => {
            if rlp.item_count()? != 4 {
                return Err(DecoderError::RlpInvalidLength.into());
            }

            // Extract the fields
            let token_address = rlp.val_at(1)?;
            let account: String = rlp.val_at(2)?;
            let amount: u128 = rlp.val_at(3)?;

            // Create a new DepositRevert instance
            let deposit_revert = DepositRevert {
                token_address,
                account,
                amount,
            };

            // Return the decoded struct as an OK variant
            Ok((
                "DepositRevert",
                DecodedStruct::DepositRevert(deposit_revert),
            ))
        }

        _ => Err(ContractError::UnknownMethod),
    }
}

pub fn is_contract(querier: QuerierWrapper, address: &Addr) -> bool {
    querier.query_wasm_contract_info(address).is_ok()
}

#[cfg(test)]
mod tests {
    use cw_ibc_rlp_lib::rlp::Encodable;

    use cw_common::xcall_data_types::Deposit;

    use super::*;

    #[test]
    fn test_encode_decode_withdraw_to() {
        let withdraw_to = WithdrawTo {
            token_address: String::from("token"),
            user_address: String::from("user"),
            amount: 1000,
        };

        let encoded_withdraw_to = withdraw_to.rlp_bytes();
        let (method, decoded_struct) = decode_encoded_bytes(&encoded_withdraw_to).unwrap();
        assert_eq!(method, "WithdrawTo");

        if let DecodedStruct::WithdrawTo(decoded_withdraw_to) = decoded_struct {
            assert_eq!(decoded_withdraw_to, withdraw_to);
        } else {
            panic!("Expected DecodedStruct::WithdrawTo variant");
        }
    }

    #[test]
    fn test_encode_decode_incoming_msg() {
        let deposit_revert = DepositRevert {
            token_address: String::from("token"),
            account: String::from("account"),
            amount: 2000,
        };

        let encoded_deposit_revert = deposit_revert.rlp_bytes();
        let (method, decoded_struct) = decode_encoded_bytes(&encoded_deposit_revert).unwrap();
        assert_eq!(method, "DepositRevert");

        if let DecodedStruct::DepositRevert(decoded_deposit_revert) = decoded_struct {
            assert_eq!(decoded_deposit_revert, deposit_revert);
        } else {
            panic!("Expected DecodedStruct::DepositRevert variant");
        }
    }

    #[test]
    fn test_unhandled_incoming_msg() {
        let unknown_method = Deposit {
            token_address: String::from("token"),
            from: String::from("user"),
            to: String::from("another_user"),
            amount: 1000,
            data: vec![],
        };

        let encoded_bytes = unknown_method.rlp_bytes();

        let result = decode_encoded_bytes(&encoded_bytes);

        assert!(result.is_err());
    }
}
