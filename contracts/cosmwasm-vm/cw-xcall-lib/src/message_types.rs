use common::rlp::Encodable;
use common::rlp::Decodable;


pub struct Message {
    msg_type:MessageType,
    data:Vec<u8>,
}


pub struct MessageWithRollback{
    msg_type:MessageType,
    data:Vec<u8>,
    rollback:Vec<u8>,
}


