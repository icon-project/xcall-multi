extern crate rlp;

use crate::rlp::{DecoderError, Rlp, RlpStream};

mod constants {
    pub const STRING_SHORT_START: u8 = 0x80;
    pub const STRING_LONG_START: u8 = 0xb8;
    pub const LIST_SHORT_START: u8 = 0xc0;
    pub const LIST_LONG_START: u8 = 0xf8;
    pub const WORD_SIZE: u8 = 32;
}

#[derive(Debug)]
pub struct RLPItem {
    len: usize,
    mem_ptr: *const u8,
}

impl RLPItem {
    pub fn new(len: usize, mem_ptr: *const u8) -> RLPItem {
        RLPItem { len, mem_ptr }
    }

    pub fn to_list(&self) -> Result<Vec<RLPItem>, DecoderError> {
        if !self.is_list() {
            return Err(DecoderError::RlpExpectedToBeList);
        }

        let items = self.num_items();
        let mut result = Vec::with_capacity(items);

        let mut mem_ptr = self.mem_ptr as usize + self.payload_offset();
        let mut data_len: usize;
        for _ in 0..items {
            data_len = self.item_length(mem_ptr as *const u8)?;
            result.push(RLPItem::new(data_len, mem_ptr as *const u8));
            mem_ptr = mem_ptr + data_len;
        }

        Ok(result)
    }

    fn is_list(&self) -> bool {
        if self.len == 0 {
            return false;
        }

        let byte0 = unsafe { *self.mem_ptr };
        if byte0 < constants::LIST_SHORT_START {
            return false;
        }

        true
    }

    fn item_length(&self, ptr: *const u8) -> Result<usize, DecoderError> {
        let byte0 = unsafe { *ptr };
        match byte0 {
            0x80..=0xb7 => Ok(1),
            0xb8..=0xbf => {
                let len_bytes = byte0  - constants::LIST_LONG_START + 1;
                let length = self.decode_length(unsafe { ptr.add(1) }, len_bytes.into())?;
                Ok((1 + len_bytes + (length as u8)).into())
            }
            _ => Err(DecoderError::RlpInvalidIndirection),
        }
    }

    fn payload_offset(&self) -> usize {
        if self.len == 0 {
            return 0;
        }

        let byte0 = unsafe { *self.mem_ptr };
        match byte0 {
            0x80..=0xb7 => 0,
            0xb8..=0xbf => (byte0 - constants::LIST_LONG_START + 1).into(),
            _ => 0,
        }
    }

    fn decode_length(&self, ptr: *const u8, len: usize) -> Result<usize, DecoderError> {
        let mut length = 0;
        for i in 0..len {
            let byte = unsafe { *ptr.add(i) };
            length = length * 256 + byte as usize;
        }

        Ok(length)
    }

    fn num_items(&self) -> usize {
        if self.len == 0 {
            return 0;
        }

        let byte0 = unsafe { *self.mem_ptr };
        match byte0 {
            0x80..=0xb7 => (byte0 - constants::LIST_SHORT_START).into(),
            0xb8..=0xbf => self
                .decode_length(unsafe { self.mem_ptr.add(1) }, (byte0 - constants::LIST_LONG_START + 1).into())
                .unwrap(),
            _ => 0,
        }
    }
}
