pub mod ParseAddress{
use cosmwasm_std::{StdError, StdResult};

pub fn parse_address(account: &str, revert_msg: &str) -> StdResult<String> {
    let account_bytes = account.as_bytes();

    if account_bytes.len() != 42 || account_bytes[0] != b'0' || account_bytes[1] != b'x' {
        return Err(StdError::generic_err(revert_msg));
    }

    let mut account_address_bytes: Vec<u8> = vec![0; 20];

    for i in 0..40 {
        let b = account_bytes[i + 2];

        let is_valid_ascii = match b {
            48..=57 | 65..=70 | 97..=102 => true,
            _ => false,
        };

        if !is_valid_ascii {
            return Err(StdError::generic_err(revert_msg));
        }

        let ascii_offset = if b < 65 {
            48 // 0-9
        } else if b > 102 {
            87 // a-f
        } else {
            55 // A-F
        };

        let nibble = if i % 2 == 0 {
            b - ascii_offset
        } else {
            (account_address_bytes[(i - 1) / 2] << 4) + (b - ascii_offset)
        };

        if i % 2 == 1 {
            account_address_bytes[(i - 1) / 2] = nibble;
        }
    }

    let packed = hex::encode(account_address_bytes);
    let mut account_address: &str = "";
    
    if account_address == "0000000000000000000000000000000000000000" {
        for i in 2..account_bytes.len() {
            if account_bytes[i] != b'0' {
                return Err(StdError::generic_err(revert_msg));
            }
        }
    }

    Ok(account_address.to_string())
}

}