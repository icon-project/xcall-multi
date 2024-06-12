use crate::error::ErrorCode;
use rlp::{Decodable, Encodable, Rlp};

#[derive(Clone,Debug,PartialEq)]
pub enum CallServiceResponseType {
    CallServiceResponseFailure ,
    CallServiceResponseSuccess ,
}

impl From<CallServiceResponseType> for u8 {
    fn from(val: CallServiceResponseType) -> Self {
        val as u8
    }
}

impl TryFrom<u8> for CallServiceResponseType {
    type Error = rlp::DecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CallServiceResponseType::CallServiceResponseFailure),
            1 => Ok(CallServiceResponseType::CallServiceResponseSuccess),
            _ => Err(rlp::DecoderError::Custom("Invalid type")),
        }
    }
}


impl CallServiceResponseType {
    // pub fn as_int(&self) -> u8 {
    //     match &self {
    //         CallServiceResponseType::CallServiceResponseFailure => 0,
    //         CallServiceResponseType::CallServiceResponseSuccess => 1,
    //     }
    // }

    // pub fn from(v : u8) -> Self {
    //     let a = match v {
    //         0 => Ok(CallServiceResponseType::CallServiceResponseFailure),
    //         1 => Ok(CallServiceResponseType::CallServiceResponseSuccess),
    //         _ => Err("eerr"), // TODO: define the error codes here
    //     };
    //     return a.unwrap();
    // }
}

#[derive(Clone,Debug,PartialEq)]
pub struct CSMessageResult{
    sequence_no : u128,
    response_code: CallServiceResponseType,
    message: Vec<u8>,
}

impl  CSMessageResult {
    
    pub fn new(
        sequence_no: u128,
        response_code: CallServiceResponseType,
        reply: Option<Vec<u8>>, // TODO: is reply an optional thing?

    ) -> Self {
        Self {
            sequence_no,
            response_code,
            message: reply.unwrap_or(vec![]),
        }
    }

    pub fn sequence_no(&self) -> u128{
        self.sequence_no
    }

    pub fn response_code(&self) -> &CallServiceResponseType {
        &self.response_code
    }

    pub fn message(&self) -> Option<CSMessageResult>{
        if self.message.is_empty() {
            return None;
        }
        rlp::decode(&self.message).ok()

    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(&self.clone()).to_vec()
    }
    
    pub fn decode_from(data: &[u8]) -> std::result::Result<Self, ErrorCode> {
        let rlp = Rlp::new(data);

        CSMessageResult::decode(&rlp).map_err(|_error| ErrorCode::CSMessageRequestDecodeError)
        
    }
}

impl Encodable for CSMessageResult {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        let code: u8 = self.response_code.clone().into();
        stream.begin_list(3);
        stream.append(&self.sequence_no());
        stream.append(&code);
        stream.append(&self.message);
    }
}

impl Decodable for CSMessageResult {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let code: u8 = rlp.val_at(1)?;

        Ok(Self {
            sequence_no: rlp.val_at(0)?,
            response_code: CallServiceResponseType::try_from(code)?,
            message: rlp.val_at(2).unwrap_or(vec![]),
        })
    }
}



#[cfg(test)]
mod tests {

    /*
    CSMessageResponse
     sn: 1
     code: CSMessageResponse.SUCCESS
     errorMessage: errorMessage
     RLP: c3010180

     CSMessageResponse
     sn: 2
     code: CSMessageResponse.FAILURE
     errorMessage: errorMessage
     RLP: c3020080
     */


    
    
    use rlp::Encodable;

    use super::{CSMessageResult,CallServiceResponseType};

    #[test]
    fn test_cs_message_reponse_encoding(){
        let cs_response = CSMessageResult::new(
            1, CallServiceResponseType::CallServiceResponseSuccess, 
            None);

        let mut stream = rlp::RlpStream::new();
        cs_response.rlp_append(&mut stream);
        let encoded = stream.as_raw();
        
        assert_eq!("c3010180", hex::encode(encoded));
        let decoded=CSMessageResult::decode_from(&encoded).unwrap();
        assert_eq!(cs_response.sequence_no(),decoded.sequence_no());
        assert_eq!(cs_response.message(),decoded.message());
        assert_eq!(cs_response.response_code(),decoded.response_code());
    
        }

    #[test]
    fn test_cs_message_reponse_encoding2(){
        let cs_response =
            CSMessageResult::new(2, 
                CallServiceResponseType::CallServiceResponseFailure, 
                None);

        let mut stream = rlp::RlpStream::new();
        cs_response.rlp_append(&mut stream);
        let encoded = stream.as_raw();

        assert_eq!("c3020080", hex::encode(encoded));

        let decoded = CSMessageResult::decode_from(&encoded).unwrap();
        assert_eq!(cs_response.sequence_no(),decoded.sequence_no());
        assert_eq!(cs_response.message(),decoded.message());
        assert_eq!(cs_response.response_code(),decoded.response_code());
        
        }


}