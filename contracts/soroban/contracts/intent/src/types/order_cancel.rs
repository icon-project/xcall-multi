use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, vec, Bytes, Env, Vec};

#[contracttype]
#[derive(Debug, Clone)]
pub struct Cancel {
    /// Encoded order data
    order_bytes: Bytes,
}

impl Cancel {
    pub fn new(order_bytes: Bytes) -> Self {
        Self { order_bytes }
    }

    pub fn order_bytes(&self) -> Bytes {
        self.order_bytes.clone()
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = vec![&e];

        list.push_back(encoder::encode(&e, self.order_bytes()));

        encoder::encode_list(&e, list, false)
    }

    pub fn decode(e: &Env, list: Bytes) -> Self {
        let decoded = decoder::decode_list(&e, list);
        if decoded.len() != 1 {
            panic!("Invalid rlp bytes length")
        }

        let order_bytes = decoded.get(0).unwrap();

        Self { order_bytes }
    }
}
