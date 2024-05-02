use soroban_sdk::{bytes, vec, Bytes, Env, String, Vec};

use crate::encoder::*;
use crate::utils::*;

#[test]
fn test_encode_u8() {
    let env = Env::default();

    let encoded = encode_u8(&env, 100);
    assert_eq!(encoded, bytes!(&env, 0x64))
}

#[test]
fn test_encode_u32() {
    let env = Env::default();

    let encoded = encode_u32(&env, 2000022458);
    assert_eq!(encoded, bytes!(&env, 0x847735EBBA));

    let encoded = encode_u128(&env, 128);
    assert_eq!(encoded, bytes!(&env, 0x8180))
}

#[test]
fn test_encode_u64() {
    let env = Env::default();

    let encoded = encode_u64(&env, 1999999999999999999);
    assert_eq!(encoded, bytes!(&env, 0x881BC16D674EC7FFFF));

    let encoded = encode_u64(&env, 199999999);
    assert_eq!(encoded, bytes!(&env, 0x840BEBC1FF))
}

#[test]
fn test_u128_u128() {
    let env = Env::default();

    let encoded = encode_u128(&env, 199999999999999999999999999999999999999);
    assert_eq!(encoded, bytes!(&env, 0x9096769950B50D88F41314447FFFFFFFFF))
}

#[test]
fn test_encode_string_with_smaller_bytes_length() {
    let env = Env::default();

    let str = "soroban-rlp";
    let encoded = encode_string(&env, String::from_str(&env, str));
    let str_bytes_slice = b"soroban-rlp";

    let expected_rlp_byte = 139;
    let mut expected_bytes = Bytes::new(&env);
    expected_bytes.push_back(expected_rlp_byte);
    expected_bytes.extend_from_slice(str_bytes_slice);

    assert_eq!(encoded, expected_bytes)
}

#[test]
fn test_encode_string_with_larger_bytes_length() {
    let env = Env::default();

    let encoded = encode_string(&env, String::from_str(&env, "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s"));

    let expected_rlp_byte = 184;
    let str_bytes_slice = b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s";
    let mut expected_bytes = Bytes::new(&env);
    expected_bytes.push_back(expected_rlp_byte);
    expected_bytes.push_back(0x97);
    expected_bytes.extend_from_slice(str_bytes_slice);

    assert_eq!(encoded, expected_bytes)
}

#[test]
fn test_encode_strings() {
    let env = Env::default();
    let strings = vec![
        &env,
        String::from_str(&env, "alice"),
        String::from_str(&env, "bob"),
    ];

    let encoded = encode_strings(&env, strings);

    let mut expected_encode = Bytes::new(&env);
    expected_encode.push_back(0xc0 + 10);
    expected_encode.push_back(0x85);
    expected_encode.extend_from_slice(b"alice");
    expected_encode.push_back(0x83);
    expected_encode.extend_from_slice(b"bob");

    assert_eq!(encoded, expected_encode);
    assert_eq!(encoded.len(), 11);
}

#[test]
fn test_encode_strings_with_longer_bytes() {
    let env = Env::default();
    let strings = vec![
            &env,
            String::from_str(&env, "It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout."),
            String::from_str(&env, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
            String::from_str(&env, "Egestas maecenas pharetra convallis posuere morbi. Velit laoreet id donec ultrices tincidunt arcu non sodales neque.")
        ];

    let encoded = encode_strings(&env, strings);

    let rlp_byte = 0xf7 + 2;
    let mut expected_encode = Bytes::new(&env);

    // rlp byte and data length bytes
    expected_encode.push_back(rlp_byte);
    expected_encode.extend_from_array(&[0x01, 0x71]);

    // strings
    let string_rlp_byte = 0xb7 + 1;
    let string_len_byte = 0x7c;
    expected_encode.extend_from_array(&[string_rlp_byte, string_len_byte]);
    expected_encode.extend_from_slice(b"It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout.");

    let string_len_byte = 0x7b;
    expected_encode.extend_from_array(&[string_rlp_byte, string_len_byte]);
    expected_encode.extend_from_slice(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");

    let string_len_byte = 0x74;
    expected_encode.extend_from_array(&[string_rlp_byte, string_len_byte]);
    expected_encode.extend_from_slice(b"Egestas maecenas pharetra convallis posuere morbi. Velit laoreet id donec ultrices tincidunt arcu non sodales neque.");

    assert_eq!(encoded, expected_encode);
    assert_eq!(encoded.len(), 372);
}

#[test]
fn test_encode_list_empty() {
    let env = Env::default();

    let list: Vec<Bytes> = vec![&env];
    let encoded = encode_list(&env, list, true);

    assert_eq!(encoded, bytes!(&env, 0xc0))
}

#[test]
fn test_encode_list_with_smaller_bytes() {
    let env = Env::default();

    let mut list: Vec<Bytes> = vec![&env];
    let short_str = String::from_str(&env, "soroban-rlp");
    list.push_back(u32_to_bytes(&env, 4294967295));
    list.push_back(string_to_bytes(&env, short_str.clone()));

    let encoded = encode_list(&env, list, true);

    let expected_rlp_byte = 0xc0 + 17;
    let mut expected_bytes = Bytes::new(&env);
    expected_bytes.push_back(expected_rlp_byte);
    expected_bytes.append(&encode_u32(&env, 4294967295));
    expected_bytes.append(&encode_string(&env, short_str));

    assert_eq!(encoded, expected_bytes)
}

#[test]
fn test_encode_list_with_longer_bytes() {
    let env = Env::default();

    let str_1 = String::from_str(&env, "Integer quis auctor elit sed vulputate mi sit.");
    let str_2 = String::from_str(&env, "Tincidunt nunc pulvinar sapien et ligula");
    let str_3 = String::from_str(&env, "Sed adipiscing diam donec adipiscing tristique");

    let mut list: Vec<Bytes> = vec![&env];
    list.push_back(encode_u8(&env, 245));
    list.push_back(encode_u32(&env, 24196199));
    list.push_back(encode_u64(&env, 103921887687475199));
    list.push_back(encode_u128(&env, 180593171625979951495805181356371083263));
    list.push_back(encode_strings(&env, vec![&env, str_1, str_2]));
    list.push_back(encode_string(&env, str_3));

    let encoded = encode_list(&env, list, false);

    let mut expected_bytes = Bytes::new(&env);

    // rlp and data len bytes
    let rlp_byte = 0xf7 + 1;
    let data_len_byte = 0xAA;
    expected_bytes.extend_from_array(&[rlp_byte, data_len_byte]);

    // u8
    expected_bytes.extend_from_array(&[0x81, 0xF5]);

    // u32
    expected_bytes.extend_from_array(&[0x84, 0x01, 0x71, 0x34, 0x67]);

    // u64
    expected_bytes.extend_from_array(&[0x88, 0x01, 0x71, 0x34, 0x67, 0xff, 0xff, 0xff, 0xff]);

    // u128
    expected_bytes.extend_from_array(&[
        0x90, 0x87, 0xdc, 0xfa, 0xcd, 0x87, 0x98, 0x27, 0x36, 0xcd, 0xef, 0xcd, 0xef, 0xff, 0xff,
        0xff, 0xff,
    ]);

    // strings
    let array_rlp_byte = 0xf7 + 1;
    let total_rlp_bytes_in_array = 2;
    let strings_len_byte = 0x56 + total_rlp_bytes_in_array;
    expected_bytes.extend_from_array(&[array_rlp_byte, strings_len_byte]);

    let rlp_byte = 0x80 + 46;
    expected_bytes.push_back(rlp_byte);
    expected_bytes.extend_from_slice(b"Integer quis auctor elit sed vulputate mi sit.");

    let rlp_byte = 0x80 + 40;
    expected_bytes.push_back(rlp_byte);
    expected_bytes.extend_from_slice(b"Tincidunt nunc pulvinar sapien et ligula");

    // string
    let rlp_byte = 0x80 + 46;
    expected_bytes.push_back(rlp_byte);
    expected_bytes.extend_from_slice(b"Sed adipiscing diam donec adipiscing tristique");

    assert_eq!(encoded, expected_bytes);
}
