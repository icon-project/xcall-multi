use common::rlp::{RlpStream, Encodable, Decodable};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct CrossTransfer {
    pub method: String,
    pub from: String,
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
}

#[cw_serde]
pub struct CrossTransferRevert {
    pub method: String,
    pub from: String,
    pub value: u128,
}

impl Encodable for CrossTransfer {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(5)
            .append(&self.method)
            .append(&self.from)
            .append(&self.to)
            .append(&self.value)
            .append(&self.data);
    }
}

impl Decodable for CrossTransfer {
    fn decode(rlp: &common::rlp::Rlp<'_>) -> Result<CrossTransfer, common::rlp::DecoderError> {
        Ok(Self {
            method: rlp.val_at(0)?,
            from: rlp.val_at(1)?,
            to: rlp.val_at(2)?,
            value: rlp.val_at(3)?,
            data: rlp.val_at(4)?,
        })
    }
}


impl Encodable for CrossTransferRevert {
    fn rlp_append(&self, stream: &mut RlpStream) {

        stream
            .begin_list(3)
            .append(&self.method)
            .append(&self.from)
            .append(&self.value);
}
}

impl Decodable for CrossTransferRevert {
    fn decode(rlp: &common::rlp::Rlp<'_>) -> Result<CrossTransferRevert, common::rlp::DecoderError> {
        Ok(Self {
            method: rlp.val_at(0)?,
            from: rlp.val_at(1)?,
            value: rlp.val_at(2)?,
    })
    }
}
impl CrossTransfer {
    pub fn encode_cross_transfer_message(self) -> Vec<u8> {
        let method = "xCrossTransfer";

        let mut calldata = RlpStream::new_list(5);
        calldata.append(&method.to_string());
        calldata.append(&self.from);
        calldata.append(&self.to);
        calldata.append(&self.value.to_string());
        calldata.append(&self.data);

        let encoded = calldata.as_raw().to_vec();
        encoded
    }
}

impl CrossTransferRevert {
    pub fn encode_cross_transfer_revert_message(self) -> Vec<u8> {
        let method = "xCrossTransferRevert";

        let mut calldata = RlpStream::new_list(3);
        calldata.append(&method.to_string());
        calldata.append(&self.from);
        calldata.append(&self.value.to_string());

        let encoded = calldata.as_raw().to_vec();
        encoded
    }
}
