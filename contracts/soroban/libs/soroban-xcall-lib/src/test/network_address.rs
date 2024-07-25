use soroban_sdk::{Env, String};

use crate::network_address::NetworkAddress;

#[test]
fn test_network_address_new() {
    let env = Env::default();

    let network_id = String::from_str(&env, "icon");
    let account = String::from_str(&env, "hx9b79391cefc9a64dfda6446312ebb7717230df5b");
    let network_address = NetworkAddress::new(&env, network_id, account);

    assert_eq!(
        network_address,
        NetworkAddress::from_string(String::from_str(
            &env,
            "icon/hx9b79391cefc9a64dfda6446312ebb7717230df5b"
        ))
    )
}

#[test]
fn test_parse_network_address() {
    let env = Env::default();

    let network_address = NetworkAddress::new(
        &env,
        String::from_str(&env, "stellar"),
        String::from_str(
            &env,
            "GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
        ),
    );
    let (network_id, account) = network_address.parse_network_address(&env);

    let expected_nid = String::from_str(&env, "stellar");
    let expected_account = String::from_str(
        &env,
        "GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
    );
    assert_eq!(network_address.nid(&env), expected_nid);
    assert_eq!(network_address.account(&env), expected_account);
    assert_eq!(network_id, expected_nid);
    assert_eq!(account, expected_account);
    assert_eq!(
        network_address.to_string(),
        String::from_str(
            &env,
            "stellar/GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU"
        )
    )
}

#[test]
#[should_panic(expected = "Invalid network address")]
fn test_parse_network_address_fail_if_separator_not_found() {
    let env = Env::default();

    let network_address = NetworkAddress::from_string(String::from_str(
        &env,
        "iconGCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
    ));
    network_address.parse_network_address(&env);
}
