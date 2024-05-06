use super::utils::*;
use soroban_sdk::{bytes, vec, Bytes, Env, String, Vec};

pub fn encode(env: &Env, bytes: Bytes) -> Bytes {
    let len = bytes.len();

    let encoded = if len == 0 {
        bytes!(&env, 0x80)
    } else if len == 1 && bytes.get(0).unwrap() < 128 {
        bytes
    } else {
        let mut res = encode_length(&env, len as u64, 0x80);
        res.append(&bytes);
        res
    };

    encoded
}

pub fn encode_list(env: &Env, list: Vec<Bytes>, raw: bool) -> Bytes {
    let mut res = Bytes::new(&env);
    if list.len() == 0 {
        res.push_back(0xc0);
    } else {
        for bytes in list {
            if raw == true {
                res.append(&encode(&env, bytes.clone()))
            } else {
                res.append(&bytes)
            }
        }
        let len = res.len();
        let mut len_buffer = encode_length(&env, len as u64, 0xc0);

        len_buffer.append(&res);
        res = len_buffer
    }
    res
}

pub fn encode_length(env: &Env, len: u64, offset: u8) -> Bytes {
    let mut len_info = Bytes::new(&env);

    if len < 56 {
        let len_u8 = len as u8;
        len_info.push_back(len_u8 + offset)
    } else {
        let mut bytes_length = u64_to_bytes(&env, len);
        let rlp_bytes_len = bytes_length.len() as u8;
        len_info.push_back(rlp_bytes_len + offset + 55);
        len_info.append(&mut bytes_length);
    }

    len_info
}

pub fn encode_u8(env: &Env, num: u8) -> Bytes {
    let mut bytes = Bytes::new(&env);
    bytes.push_back(num);

    encode(&env, bytes)
}

pub fn encode_u32(env: &Env, num: u32) -> Bytes {
    let bytes = u32_to_bytes(&env, num);
    encode(&env, bytes)
}

pub fn encode_u64(env: &Env, num: u64) -> Bytes {
    let bytes = u64_to_bytes(&env, num);
    encode(&env, bytes)
}

pub fn encode_u128(env: &Env, num: u128) -> Bytes {
    let bytes = u128_to_bytes(&env, num);
    encode(&env, bytes)
}

pub fn encode_string(env: &Env, value: String) -> Bytes {
    let bytes = string_to_bytes(&env, value);
    encode(&env, bytes)
}

pub fn encode_strings(env: &Env, values: Vec<String>) -> Bytes {
    let mut list: Vec<Bytes> = vec![&env];

    for value in values {
        list.push_back(encode_string(&env, value));
    }

    encode_list(&env, list, false)
}
