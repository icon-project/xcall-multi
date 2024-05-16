use soroban_sdk::Bytes;

pub trait IMessage {
    fn data(&self) -> Bytes;
    fn rollback(&self) -> Option<Bytes>;
}
