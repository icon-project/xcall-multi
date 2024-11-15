use soroban_sdk::{bytes, Bytes, Env, String};

use crate::types::{Cancel, MessageType, OrderFill, OrderMessage, SwapOrder};

#[test]
fn test_order_fill_decode_1() {
    let env = Env::default();

    let data = OrderFill::new(
        1,
        bytes!(&env, 0x6c449988e2f33302803c93f8287dc1d8cb33848a),
        String::from_str(&env, "0xcb0a6bbccfccde6be9f10ae781b9d9b00d6e63"),
    );
    let exepected = bytes!(&env, 0xf83f01946c449988e2f33302803c93f8287dc1d8cb33848aa830786362306136626263636663636465366265396631306165373831623964396230306436653633);
    assert_eq!(data.encode(&env), exepected)
}

#[test]
fn test_order_fill_decode_2() {
    let env = Env::default();

    let data = OrderFill::new(
        2,
        bytes!(&env, 0xcb0a6bbccfccde6be9f10ae781b9d9b00d6e63),
        String::from_str(&env, "0x6c449988e2f33302803c93f8287dc1d8cb33848a"),
    );
    let exepected = bytes!(&env, 0xf8400293cb0a6bbccfccde6be9f10ae781b9d9b00d6e63aa307836633434393938386532663333333032383033633933663832383764633164386362333338343861);
    assert_eq!(data.encode(&env), exepected)
}

#[test]
fn test_order_cancel_decode() {
    let env = Env::default();

    let data = Cancel::new(bytes!(&env, 0x6c449988e2f33302803c93f8287dc1d8cb33848a));
    let exepected = bytes!(&env, 0xd5946c449988e2f33302803c93f8287dc1d8cb33848a);
    assert_eq!(data.encode(&env), exepected)
}

#[test]
fn test_order_message_decode_1() {
    let env = Env::default();

    let data = OrderMessage::new(
        MessageType::CANCEL,
        bytes!(&env, 0x6c449988e2f33302803c93f8287dc1d8cb33848a),
    );
    let exepected = bytes!(&env, 0xd602946c449988e2f33302803c93f8287dc1d8cb33848a);
    assert_eq!(data.encode(&env), exepected)
}

#[test]
fn test_order_message_decode_2() {
    let env = Env::default();

    let data = OrderMessage::new(
        MessageType::FILL,
        bytes!(&env, 0x6c449988e2f33302803c93f8287dc1d8cb33848a),
    );
    let exepected = bytes!(&env, 0xd601946c449988e2f33302803c93f8287dc1d8cb33848a);
    assert_eq!(data.encode(&env), exepected)
}

#[test]
fn test_swap_order_decode_1() {
    let env = Env::default();

    let swap_order = SwapOrder::new(
        1,
        String::from_str(&env, "0xbe6452d4d6c61cee97d3"),
        String::from_str(&env, "Ethereum"),
        String::from_str(&env, "Polygon"),
        String::from_str(&env, "0x3e36eddd65e239222e7e67"),
        String::from_str(&env, "0xd2c6218b875457a41b6fb7964e"),
        String::from_str(&env, "0x14355340e857912188b7f202d550222487"),
        1000,
        String::from_str(&env, "0x91a4728b517484f0f610de7b"),
        900,
        Bytes::new(&env),
    );
    let expected = bytes!(&env, 0xf900a601963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378203e89a307839316134373238623531373438346630663631306465376282038480);
    assert_eq!(swap_order.encode(&env), expected)
}

#[test]
fn test_swap_order_decode_2() {
    let env = Env::default();

    let swap_order = SwapOrder::new(
        1,
        String::from_str(&env, "0xbe6452d4d6c61cee97d3"),
        String::from_str(&env, "Ethereum"),
        String::from_str(&env, "Polygon"),
        String::from_str(&env, "0x3e36eddd65e239222e7e67"),
        String::from_str(&env, "0xd2c6218b875457a41b6fb7964e"),
        String::from_str(&env, "0x14355340e857912188b7f202d550222487"),
        100000 * 10_u128.pow(22),
        String::from_str(&env, "0x91a4728b517484f0f610de7b"),
        900 * 10_u128.pow(7),
        bytes!(&env, 0x6c449988e2f33302803c93f8287dc1d8cb33848a),
    );
    let expected = bytes!(&env, 0xf900c701963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378c033b2e3c9fd0803ce80000009a3078393161343732386235313734383466306636313064653762850218711a00946c449988e2f33302803c93f8287dc1d8cb33848a);
    assert_eq!(swap_order.encode(&env), expected)
}
