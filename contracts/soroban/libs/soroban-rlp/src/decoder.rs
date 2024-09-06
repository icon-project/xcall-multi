use super::utils::*;
use soroban_sdk::{vec, Bytes, Env, String, Vec};

pub fn decode(env: &Env, bytes: Bytes) -> Bytes {
    assert!(bytes.len() > 0);

    let rlp_byte = bytes.get(0).unwrap();

    #[allow(unused_comparisons)]
    let decoded = if rlp_byte == 0x80 || rlp_byte == 0xc0 {
        Bytes::new(&env)
    } else if rlp_byte < 0x80 {
        bytes
    } else if rlp_byte < 0xb8 {
        let data_len = rlp_byte - 0x80;
        slice_vector(&env, bytes, 1, data_len as u64)
    } else if rlp_byte > 0xb7 && rlp_byte < 0xc0 {
        let data_bytes_len = rlp_byte - 0xb7;
        let len_bytes = slice_vector(&env, bytes.clone(), 1, data_bytes_len as u64);

        let data_len = bytes_to_u64(len_bytes.clone());
        let data_start = len_bytes.len() + 1;

        slice_vector(&env, bytes, data_start as u64, data_len)
    } else if rlp_byte > 0xc0 && rlp_byte <= 0xf7 {
        let data_len = rlp_byte - 0xc0;
        slice_vector(&env, bytes, 1, data_len as u64)
    } else if rlp_byte > 0xf7 && rlp_byte <= 0xff {
        let data_bytes_len = rlp_byte - 0xf7;
        let len_bytes = slice_vector(&env, bytes.clone(), 1, data_bytes_len as u64);

        let data_len = bytes_to_u64(len_bytes.clone());
        let data_start = len_bytes.len() + 1;

        slice_vector(&env, bytes, data_start as u64, data_len)
    } else {
        panic!("invalid rlp byte length")
    };

    decoded
}

pub fn decode_list(env: &Env, list: Bytes) -> Vec<Bytes> {
    let data_len = decode_length(&env, list.clone(), 0xc0);
    let start = list.len() as u64 - data_len;
    let encoded = slice_vector(&env, list, start, data_len);

    let mut decoded: Vec<Bytes> = Vec::new(&env);
    let mut i = 0;
    while i < encoded.len() {
        let byte = encoded.get(i).unwrap();

        #[allow(unused_comparisons)]
        if byte == 0x80 || byte == 0xc0 {
            decoded.push_back(Bytes::new(&env));
            i = i + 1;
        } else if byte < 0x80 {
            let mut singleton = Bytes::new(&env);
            singleton.push_back(byte);
            decoded.push_back(singleton);
            i = i + 1;
        } else if byte > 0x80 && byte < 0xB8 {
            let len = (byte - 0x80) as u64;
            decoded.push_back(slice_vector(&env, encoded.clone(), i as u64 + 1, len));
            i = i + (len as u32 + 1);
        } else if byte > 0xc0 && byte < 0xf7 {
            let len = (byte - 0xc0) as u64;
            decoded.push_back(slice_vector(&env, encoded.clone(), i as u64, len + 1));
            i = i + (len as u32 + 1)
        } else if byte > 0xb7 && byte < 0xc0 {
            let data_bytes_len = (byte - 0xb7) as u64;
            let len_bytes = slice_vector(&env, encoded.clone(), i as u64 + 1, data_bytes_len);
            let len = bytes_to_u64(len_bytes);
            decoded.push_back(slice_vector(
                &env,
                encoded.clone(),
                i as u64 + data_bytes_len + 1,
                len,
            ));
            i = i + (data_bytes_len + len + 1) as u32
        } else if byte > 0xf7 && byte <= 0xff {
            let data_bytes_len = (byte - 0xf7) as u64;
            let len_bytes = slice_vector(&env, encoded.clone(), i as u64 + 1, data_bytes_len);
            let len = bytes_to_u64(len_bytes);
            if byte == 0xf8 && len == 0 {
                decoded.push_back(Bytes::new(&env));
            } else {
                decoded.push_back(slice_vector(
                    &env,
                    encoded.clone(),
                    i as u64,
                    data_bytes_len + len + 1,
                ));
            }
            i = i + (data_bytes_len + len + 1) as u32
        } else {
            panic!("invalid rlp byte length")
        }
    }
    decoded
}

pub fn decode_length(env: &Env, bytes: Bytes, offset: u8) -> u64 {
    let bytes_len = bytes.len();

    let len = if bytes_len == 0 {
        0
    } else if bytes_len < 56 {
        (bytes.get(0).unwrap() - offset) as u64
    } else {
        let len = bytes.get(0).unwrap() - offset - 55;
        let len_bytes = slice_vector(env, bytes, 1, len as u64);
        bytes_to_u64(len_bytes)
    };

    len
}

pub fn decode_u8(env: &Env, bytes: Bytes) -> u8 {
    decode(&env, bytes).get(0).unwrap_or(0)
}

pub fn decode_u32(env: &Env, bytes: Bytes) -> u32 {
    let decoded = decode(&env, bytes);
    bytes_to_u32(decoded)
}

pub fn decode_u64(env: &Env, bytes: Bytes) -> u64 {
    let decoded = decode(&env, bytes);
    bytes_to_u64(decoded)
}

pub fn decode_u128(env: &Env, bytes: Bytes) -> u128 {
    let decoded = decode(&env, bytes);
    bytes_to_u128(decoded)
}

pub fn decode_string(env: &Env, bytes: Bytes) -> String {
    let decoded = decode(&env, bytes);
    bytes_to_string(&env, decoded)
}

pub fn decode_strings(env: &Env, bytes: Bytes) -> Vec<String> {
    let list = decode_list(&env, bytes);

    let mut strings: Vec<String> = vec![&env];
    for byte in list {
        strings.push_back(bytes_to_string(&env, byte))
    }

    strings
}
