use soroban_sdk::{contracttype, vec, Bytes, BytesN, Env, String, Vec};
use soroban_rlp::{decoder, encoder};

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct SwapOrder {
    /// Unique identifier for each order
    id: u128,
    /// Address of emitter contract
    emitter: String,
    /// Network ID of the source chain
    src_nid: String,
    /// Netword ID of the destination chain
    dst_nid: String,
    /// Address of the user who created the swap order
    creator: String,
    /// Address where the swapped token should be sent
    destination_address: String,
    /// Address of the token to be swapped
    token: String,
    /// Amount of the token to be swapped
    amount: u128,
    /// Address of the token to receive on the destination chain
    to_token: String,
    /// Amount of `to_token` expected to be received
    to_amount: u128,
    /// Additional data for the swap
    data: Bytes,
}

impl SwapOrder {
    pub fn new(
        id: u128,
        emitter: String,
        src_nid: String,
        dst_nid: String,
        creator: String,
        destination_address: String,
        token: String,
        amount: u128,
        to_token: String,
        to_amount: u128,
        data: Bytes,
    ) -> Self {
        Self {
            id,
            emitter,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            to_amount,
            data,
        }
    }

    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn set_id(&mut self, id: u128) {
        self.id = id
    }

    pub fn emitter(&self) -> String {
        self.emitter.clone()
    }

    pub fn src_nid(&self) -> String {
        self.src_nid.clone()
    }

    pub fn dst_nid(&self) -> String {
        self.dst_nid.clone()
    }

    pub fn creator(&self) -> String {
        self.creator.clone()
    }

    pub fn dst_address(&self) -> String {
        self.destination_address.clone()
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }

    pub fn amount(&self) -> u128 {
        self.amount
    }

    pub fn to_token(&self) -> String {
        self.to_token.clone()
    }

    pub fn to_amount(&self) -> u128 {
        self.to_amount
    }

    pub fn data(&self) -> Bytes {
        self.data.clone()
    }

    pub fn set_data(&mut self, data: Bytes) {
        self.data = data
    }

    pub fn get_hash(&self, e: &Env) -> BytesN<32> {
        e.crypto().keccak256(&self.encode(&e))
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut bytes: Vec<Bytes> = vec![&e];

        bytes.push_back(encoder::encode_u128(&e, self.id()));
        bytes.push_back(encoder::encode_string(&e, self.emitter()));
        bytes.push_back(encoder::encode_string(&e, self.src_nid()));
        bytes.push_back(encoder::encode_string(&e, self.dst_nid()));
        bytes.push_back(encoder::encode_string(&e, self.creator()));
        bytes.push_back(encoder::encode_string(&e, self.dst_address()));
        bytes.push_back(encoder::encode_string(&e, self.token()));
        bytes.push_back(encoder::encode_u128(&e, self.amount()));
        bytes.push_back(encoder::encode_string(&e, self.to_token()));
        bytes.push_back(encoder::encode_u128(&e, self.to_amount()));
        bytes.push_back(encoder::encode(&e, self.data()));

        encoder::encode_list(&e, bytes, false)
    }

    pub fn decode(e: &Env, bytes: Bytes) -> Self {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 11 {
            panic!("Invalid rlp bytes")
        }

        let id = decoder::decode_u128(&e, decoded.get(0).unwrap());
        let emitter = decoder::decode_string(&e, decoded.get(1).unwrap());
        let src_nid = decoder::decode_string(&e, decoded.get(2).unwrap());
        let dst_nid = decoder::decode_string(&e, decoded.get(3).unwrap());
        let creator = decoder::decode_string(&e, decoded.get(4).unwrap());
        let destination_address = decoder::decode_string(&e, decoded.get(5).unwrap());
        let token = decoder::decode_string(&e, decoded.get(6).unwrap());
        let amount = decoder::decode_u128(&e, decoded.get(7).unwrap());
        let to_token = decoder::decode_string(&e, decoded.get(8).unwrap());
        let to_amount = decoder::decode_u128(&e, decoded.get(9).unwrap());
        let data = decoded.get(10).unwrap();

        Self {
            id,
            emitter,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            to_amount,
            data,
        }
    }
}
