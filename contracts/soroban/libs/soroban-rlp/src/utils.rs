use soroban_sdk::{
    bytes,
    xdr::{FromXdr, ToXdr},
    Bytes, Env, String,
};

pub fn u32_to_bytes(env: &Env, number: u32) -> Bytes {
    let bytes = Bytes::from_slice(&env, &number.to_be_bytes());
    to_signed_bytes(&env, bytes)
}

pub fn u64_to_bytes(env: &Env, number: u64) -> Bytes {
    let bytes = Bytes::from_slice(&env, &number.to_be_bytes());
    to_signed_bytes(&env, bytes)
}

pub fn u128_to_bytes(env: &Env, number: u128) -> Bytes {
    let bytes = Bytes::from_slice(&env, &number.to_be_bytes());
    to_signed_bytes(&env, bytes)
}

pub fn to_signed_bytes(env: &Env, bytes: Bytes) -> Bytes {
    let truncated = truncate_zeros(&env, bytes);
    let first_byte = truncated.get(0).unwrap_or(0);

    if first_byte >= 128 {
        let mut prefix = bytes!(&env, 0x00);
        prefix.append(&truncated);
        prefix
    } else {
        truncated
    }
}

pub fn truncate_zeros(env: &Env, bytes: Bytes) -> Bytes {
    let mut i = 0;
    let mut started = false;
    let mut result = Bytes::new(&env);

    while i < bytes.len() {
        let val = bytes.get(i).unwrap();
        if val > 0 || started {
            started = true;
            result.push_back(val);
        }
        i = i + 1;
    }

    result
}

pub fn bytes_to_u32(bytes: Bytes) -> u32 {
    let mut num = 0;
    for byte in bytes.iter() {
        num = (num << 8) | byte as u32;
    }
    num
}

pub fn bytes_to_u64(bytes: Bytes) -> u64 {
    let mut num = 0;
    for byte in bytes.iter() {
        num = (num << 8) | byte as u64;
    }
    num
}

pub fn bytes_to_u128(bytes: Bytes) -> u128 {
    let mut num = 0;
    for byte in bytes.iter() {
        num = (num << 8) | byte as u128
    }
    num
}

pub fn slice_vector(env: &Env, arr: Bytes, start: u64, length: u64) -> Bytes {
    let mut sliced = Bytes::new(&env);
    let mut start = start;
    let end = start + length;

    while start < end {
        let item = arr.get(start as u32).unwrap();
        sliced.push_back(item);
        start += 1;
    }
    sliced
}

pub fn string_to_bytes(env: &Env, value: String) -> Bytes {
    let mut start_index = 8;
    let end_index = start_index + value.len();
    let string_xdr = value.to_xdr(&env);

    let mut bytes = Bytes::new(&env);
    while start_index < end_index {
        bytes.push_back(string_xdr.get(start_index).unwrap());
        start_index += 1;
    }
    bytes
}

pub fn bytes_to_string(env: &Env, bytes: Bytes) -> String {
    let mut bytes_xdr = bytes.to_xdr(&env);
    bytes_xdr.set(3, 14);

    String::from_xdr(&env, &bytes_xdr).unwrap()
}
