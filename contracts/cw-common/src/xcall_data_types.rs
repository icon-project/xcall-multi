use cosmwasm_schema::cw_serde;
use cw_ibc_rlp_lib::rlp::{Encodable, RlpStream};

//for testing
#[cw_serde]
pub struct Deposit {
    pub token_address: String,
    // archway address of the caller
    pub from: String,
    // network address for receiver of hub token
    pub to: String,
    pub amount: u128,
    pub data: Vec<u8>,
}

#[cw_serde]
pub struct DepositRevert {
    pub token_address: String,
    pub account: String,
    pub amount: u128,
}

#[cw_serde]
pub struct WithdrawTo {
    pub token_address: String,
    pub user_address: String,
    pub amount: u128,
}

#[cw_serde]
pub struct WithdrawNativeTo {
    pub token_address: String,
    pub user_address: String,
    pub amount: u128,
}

//for testing
impl Encodable for Deposit {
    //specify the encoding logic for struct's fields so that rlp_bytes() can alo use
    fn rlp_append(&self, s: &mut RlpStream) {
        //append struct's each field to stream object
        let method = "Deposit".to_string();
        s.begin_list(6)
            .append(&method)
            .append(&self.token_address)
            .append(&self.from)
            .append(&self.to)
            .append(&self.amount)
            .append(&self.data);
    }
}

impl Encodable for DepositRevert {
    fn rlp_append(&self, s: &mut RlpStream) {
        let method = "DepositRevert".to_string();
        s.begin_list(4)
            .append(&method)
            .append(&self.token_address)
            .append(&self.account)
            .append(&self.amount);
    }
}

impl Encodable for WithdrawTo {
    fn rlp_append(&self, s: &mut RlpStream) {
        let method = "WithdrawTo".to_string();
        s.begin_list(4)
            .append(&method)
            .append(&self.token_address)
            .append(&self.user_address)
            .append(&self.amount);
    }
}

impl Encodable for WithdrawNativeTo {
    fn rlp_append(&self, s: &mut RlpStream) {
        let method = "WithdrawNativeTo".to_string();
        s.begin_list(4)
            .append(&method)
            .append(&self.token_address)
            .append(&self.user_address)
            .append(&self.amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Addr;

    #[test]
    fn test_encode() {
        let token = Addr::unchecked("token").to_string();
        let from = Addr::unchecked("from").to_string();
        let to = Addr::unchecked("to").to_string();

        let deposit = Deposit {
            token_address: token,
            from,
            to,
            amount: 1000,
            data: vec![],
        };

        let deposit_revert = DepositRevert {
            token_address: "contract1".to_string(),
            account: "sender".to_string(),
            amount: 100,
        };

        //use rlp bytes
        //internally relies on rlp_append to perform the actual encoding(you can check bro !)
        let encoded_deposit = deposit.rlp_bytes();

        let encode_deposit_revert = deposit_revert.rlp_bytes();
        println!("deposit reert data: {:?}", encode_deposit_revert.to_vec());

        // Use rlp_append
        let mut stream = RlpStream::new();
        deposit.rlp_append(&mut stream);
        let encoded_append = stream.out();

        //ensuring both methods generates identical encoded bytes
        assert_eq!(encoded_deposit, encoded_append);

        //checking if encoded structs are different
        assert_ne!(encoded_deposit, encode_deposit_revert);
    }
}
