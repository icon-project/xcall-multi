use soroban_sdk::{bytes, vec, Bytes, Env, String, Vec};

use crate::decoder::*;
use crate::encoder;

#[test]
fn test_decode_u32() {
    let env = Env::default();

    let bytes = bytes!(&env, 0x843548668C);
    let decoded = decode_u32(&env, bytes);
    assert_eq!(decoded, 893937292);

    let bytes = bytes!(&env, 0x830DA3F1);
    let decoded = decode_u32(&env, bytes);
    assert_eq!(decoded, 893937)
}

#[test]
fn test_decode_u64() {
    let env = Env::default();

    let bytes = bytes!(&env, 0x88FFFFFFFFFFFFFFFF);
    let decoded = decode_u64(&env, bytes);
    assert_eq!(decoded, 18446744073709551615)
}

#[test]
fn test_decode_u128() {
    let env = Env::default();

    let bytes = bytes!(&env, 0x90FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let decoded = decode_u128(&env, bytes);
    assert_eq!(decoded, 340282366920938463463374607431768211455)
}

#[test]
fn test_decode_string_with_less_bytes() {
    let env = Env::default();

    let str = String::from_str(&env, "soroban-rlp");
    let encoded = encoder::encode_string(&env, str.clone());
    let decoded = decode_string(&env, encoded);
    assert_eq!(str, decoded)
}

#[test]
fn test_decode_string_with_longer_bytes() {
    let env = Env::default();

    let str = String::from_str(
        &env,
        "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
    );
    let encoded = encoder::encode_string(&env, str.clone());
    let decoded = decode_string(&env, encoded);
    assert_eq!(str, decoded)
}

#[test]
fn test_decode_strings() {
    let env = Env::default();
    let string_list = vec![
        &env,
        String::from_str(
            &env,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
        ),
        String::from_str(
            &env,
            "sed do eiusmod tempor incididunt ut labore et dolore magna aliqua ",
        ),
    ];

    let encoded = encoder::encode_strings(&env, string_list.clone());
    let decoded = decode_strings(&env, encoded);

    assert_eq!(string_list, decoded)
}

#[test]
fn test_decode_list() {
    let env = Env::default();

    let str_1 = String::from_str(&env, "Integer quis auctor elit sed vulputate mi sit.");
    let str_2 = String::from_str(&env, "Tincidunt nunc pulvinar sapien et ligula");
    let str_3 = String::from_str(&env, "Sed adipiscing diam donec adipiscing tristique");

    let mut list: Vec<Bytes> = vec![&env];
    list.push_back(encoder::encode_u8(&env, 245));
    list.push_back(encoder::encode_u128(
        &env,
        180593171625979951495805181356371083263,
    ));
    list.push_back(encoder::encode_u32(&env, 24196199));
    list.push_back(encoder::encode_u64(&env, 103921887687475199));
    list.push_back(encoder::encode_strings(&env, vec![&env, str_1, str_2]));
    list.push_back(encoder::encode_string(&env, str_3));

    let encoded = encoder::encode_list(&env, list.clone(), true);
    let decoded = decode_list(&env, encoded);

    assert_eq!(list, decoded)
}
