pub use cosmwasm_std::Addr;
pub use cw_xcall_lib::network_address::{NetId, NetworkAddress};
pub use std::str::FromStr;

pub trait IconAddressValidation {
    fn validate_foreign_addresses(&self) -> bool;
}

impl IconAddressValidation for NetworkAddress {
    fn validate_foreign_addresses(&self) -> bool {
        let parts = self.get_parts();
        let net_id = parts[0].to_string();
        let address = parts[1];
        match net_id {
            s if s.contains("icon") => validate_icon_address(address),
            _ => false,
        }
    }
}

fn validate_icon_address(address: &str) -> bool {
    let lowercase_address = address.to_lowercase();

    if !lowercase_address.starts_with("hx") && !lowercase_address.starts_with("cx") {
        return false;
    }

    lowercase_address.len() == 42 && is_valid_character_set(&lowercase_address[2..])
}

fn is_valid_character_set(address: &str) -> bool {
    address.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

#[test]
fn test_parse_btp_address() {
    let network_address =
        NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba98765432").unwrap();
    assert_eq!(network_address.nid(), NetId::from_str("0x01.icon").unwrap());
    assert_eq!(
        network_address.account(),
        Addr::unchecked("cx9876543210fedcba9876543210fedcba98765432")
    );
}

#[test]
fn address_validation_test() {
    let network_address =
        NetworkAddress::from_str("0x7.icon/cxd06f80e28e989a67e297799ab1fb501cdddc2b4d").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(res);

    let network_address =
        NetworkAddress::from_str("0x01.icon/hx9876543210fedcba9876543210fedcba98765432").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(res);

    let network_address =
        NetworkAddress::from_str("0x01.bsc/cx9876543210fedcba9876543210fedcba98765432").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(!res);

    let network_address =
        NetworkAddress::from_str("0x01.icon/wx9876543210fedcba9876543210fedcba98765432").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(!res);

    let network_address =
        NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba9876542").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(!res);

    let network_address =
        NetworkAddress::from_str("0x01.icon/cx9876543210fedcba9876543210fedcba9876543_").unwrap();
    let res = network_address.validate_foreign_addresses();
    assert!(!res);
}
