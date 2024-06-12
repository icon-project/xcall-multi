
use anchor_lang::AnchorDeserialize;
use borsh::{BorshDeserialize,BorshSerialize};

#[derive(Clone,Debug,PartialEq,BorshDeserialize,BorshSerialize, AnchorDeserialize)]
pub enum MessageType {
    CallMessage = 0,
    CallMessageWithRollback = 1,
    CallMessagePersisted = 2,
}

impl From<MessageType> for u8 {
    fn from(val : MessageType) -> Self{
        match val {
            MessageType::CallMessage => 0,
            MessageType::CallMessageWithRollback =>1,
            MessageType::CallMessagePersisted =>2,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self{
        match value {
            0 => MessageType::CallMessage,
            1 => MessageType::CallMessageWithRollback,
            2 => MessageType::CallMessagePersisted,
            _ => panic!("unsupported message type"),
        }
    }
}

impl MessageType {

    pub fn as_int(&self) -> u8{
        // from ko implemenation bata into ma lagyo but 
        // kun from ko implementation leko ta
        self.clone().into()
    }

    pub fn from_int(val: u8) -> Self {
        MessageType::from(val)
    }

    
}


#[cfg(test)]
mod tests{

    #[test]
    pub fn test_match_from_u8(){
        
    }
}
