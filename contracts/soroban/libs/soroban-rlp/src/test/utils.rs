use crate::utils::*;
use soroban_sdk::{bytes, Bytes, Env};

#[test]
fn test_u32_to_bytes() {
    let env = Env::default();

    let num = 0x12345678;
    let bytes = u32_to_bytes(&env, num);
    let expected_num = bytes_to_u32(bytes);

    assert_eq!(num, expected_num)
}

#[test]
fn test_u64_to_bytes() {
    let env = Env::default();

    let num: u64 = 18446744073709551615;
    let bytes = u64_to_bytes(&env, num);
    let expected_num = bytes_to_u64(bytes);

    assert_eq!(num, expected_num)
}

#[test]
fn test_u128_to_bytes() {
    let env = Env::default();

    let num: u128 = 340282366920938463463374607431768211455;
    let bytes = u128_to_bytes(&env, num);
    let expected_num = bytes_to_u128(bytes);

    assert_eq!(num, expected_num)
}

#[test]
fn test_remove_leading_zero() {
    let env = Env::default();

    let bytes = u32_to_bytes(&env, 12);
    assert_eq!(bytes, bytes!(&env, 0x000000C));

    let without_zero = remove_leading_zero(&env, bytes);
    let mut expected_bytes = Bytes::new(&env);
    expected_bytes.push_back(12);

    assert_eq!(without_zero, expected_bytes)
}

#[test]
fn test_slice_vector() {
    let env = Env::default();

    let bytes = u128_to_bytes(&env, 1844674407);
    let slice = slice_vector(&env, bytes.clone(), 12, 4);

    assert_eq!(slice, bytes!(&env, 0x6DF37F67));
}
