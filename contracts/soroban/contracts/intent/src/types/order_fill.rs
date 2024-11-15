use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, vec, Bytes, Env, String, Vec};

#[contracttype]
#[derive(Debug, Clone)]
pub struct OrderFill {
    /// ID of the order being filled
    id: u128,
    /// Encoded order data
    order_bytes: Bytes,
    /// Address of the solver filling the order
    solver: String,
}

impl OrderFill {
    pub fn new(id: u128, order_bytes: Bytes, solver: String) -> Self {
        Self {
            id,
            order_bytes,
            solver,
        }
    }

    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn order_bytes(&self) -> Bytes {
        self.order_bytes.clone()
    }

    pub fn solver(&self) -> String {
        self.solver.clone()
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = vec![&e];

        list.push_back(encoder::encode_u128(&e, self.id()));
        list.push_back(encoder::encode(&e, self.order_bytes()));
        list.push_back(encoder::encode_string(&e, self.solver()));

        encoder::encode_list(&e, list, false)
    }

    pub fn decode(e: &Env, list: Bytes) -> Self {
        let decoded = decoder::decode_list(&e, list);
        if decoded.len() != 3 {
            panic!("Invalid rlp bytes length")
        }

        let id = decoder::decode_u128(&e, decoded.get(0).unwrap());
        let order_bytes = decoded.get(1).unwrap();
        let solver = decoder::decode_string(&e, decoded.get(2).unwrap());

        Self {
            id,
            order_bytes,
            solver,
        }
    }
}
