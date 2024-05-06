use soroban_sdk::{
    xdr::{FromXdr, ToXdr},
    Bytes, Env, String,
};

pub fn u32_to_bytes(env: &Env, number: u32) -> Bytes {
    let mut bytes = Bytes::new(&env);
    let mut i = 3;
    while i >= 0 {
        let val = (number >> (i * 8) & 0xff) as u8;
        if val > 0 {
            bytes.push_back(val);
        }

        i -= 1;
    }
    bytes
}

pub fn bytes_to_u32(bytes: Bytes) -> u32 {
    let mut num = 0;
    for byte in bytes.iter() {
        num = (num << 8) | byte as u32;
    }
    num
}

pub fn u64_to_bytes(env: &Env, number: u64) -> Bytes {
    let mut bytes = Bytes::new(&env);
    let mut i = 7;
    while i >= 0 {
        let val = (number >> (i * 8) & 0xff) as u8;
        if val > 0 {
            bytes.push_back(val);
        }

        i -= 1;
    }
    bytes
}

pub fn bytes_to_u64(bytes: Bytes) -> u64 {
    let mut num = 0;
    for byte in bytes.iter() {
        num = (num << 8) | byte as u64;
    }
    num
}

pub fn u128_to_bytes(env: &Env, number: u128) -> Bytes {
    let mut bytes = Bytes::new(&env);
    let mut i = 15;
    while i >= 0 {
        let val = (number >> (i * 8) & 0xff) as u8;
        if val > 0 {
            bytes.push_back(val);
        }

        i -= 1;
    }

    bytes
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
