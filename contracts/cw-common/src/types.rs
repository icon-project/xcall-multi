use common::rlp::{RlpStream};
use cosmwasm_schema::{cw_serde};

pub mod types {
    use super::*;

    #[cw_serde]
    pub struct CrossTransfer {
        pub from: String,
        pub to: String,
        pub value: u128,
        pub data: Vec<u8>,
    }

    #[cw_serde]
    pub struct CrossTransferRevert {
        pub from: String,
        pub value: u128,
    }
}

impl types::CrossTransfer {
    pub fn encode_cross_transfer_message(self) -> Vec<u8> {
        let method = "xCrossTransfer";

        let mut calldata = RlpStream::new_list(4);
        calldata.append(&method.to_string());
        calldata.append(&self.from);
        calldata.append(&self.to);
        calldata.append(&self.value);
        calldata.append(&self.data);

        let encoded = calldata.as_raw().to_vec();
        encoded
    }
}

impl types::CrossTransferRevert {
    pub fn encode_cross_transfer_revert_message(self) -> Vec<u8> {
        let method = "xCrossTransferRevert";

        let mut calldata = RlpStream::new_list(3);
        calldata.append(&method.to_string());
        calldata.append(&self.from);
        calldata.append(&self.value);

        let encoded = calldata.as_raw().to_vec();
        encoded
    }
}
 