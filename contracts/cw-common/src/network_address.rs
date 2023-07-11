use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use cw_storage_plus::{Key, KeyDeserialize, PrimaryKey};

#[cw_serde]
#[derive(Eq)]
pub struct NetId(String);

impl From<String> for NetId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ToString for NetId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl NetId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for NetId {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl<'a> PrimaryKey<'a> for NetId {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(self.0.as_bytes())]
    }
}

impl KeyDeserialize for NetId {
    type Output = NetId;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        let result = String::from_utf8(value)
            .map_err(StdError::invalid_utf8)
            .unwrap();
        let net_id = NetId::from_str(&result).unwrap();
        Ok(net_id)
    }
}

#[cw_serde]
#[derive(Eq)]
pub struct NetworkAddress(pub String);

impl NetworkAddress {
    pub fn new(nid: &str, address: &str) -> Self {
        Self(format!("{}/{}", nid, address))
    }

    pub fn nid(&self) -> NetId {
        NetId(self.get_parts()[0].to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn account(&self) -> Addr {
        Addr::unchecked(self.get_parts()[1])
    }

    pub fn get_parts(&self) -> Vec<&str> {
        let parts = self.0.split('/').collect::<Vec<&str>>();
        parts
    }

    pub fn parse_parts(&self) -> (NetId, Addr) {
        let parts = self.0.split('/').collect::<Vec<&str>>();
        (NetId(parts[0].to_string()), Addr::unchecked(parts[1]))
    }

    pub fn validate(&self) -> bool {
        let parts = self.get_parts();
        let net_id = parts[0].to_string();
        let address = parts[1];
        match net_id {
            s if s.contains("icon") => validate_icon_address(address),
            _ => false,
        }
    }
}

impl ToString for NetworkAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for NetworkAddress {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(StdError::GenericErr {
                msg: "Invalid Network Address".to_owned(),
            });
        }
        let na = format!("{}/{}", parts[0], parts[1]);
        Ok(Self(na))
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
        NetworkAddress("0x01.icon/cx9876543210fedcba9876543210fedcba98765432".to_string());
    let (network, account) = network_address.parse_parts();
    assert_eq!(network, NetId("0x01.icon".to_string()));
    assert_eq!(
        account,
        Addr::unchecked("cx9876543210fedcba9876543210fedcba98765432")
    );
}

#[test]
fn address_validation_test() {
    let network_address =
        NetworkAddress("0x01.icon/cx9876543210fedcba9876543210fedcba98765432".to_string());
    let res = network_address.validate();
    assert!(res);

    let network_address =
        NetworkAddress("0x01.icon/hx9876543210fedcba9876543210fedcba98765432".to_string());
    let res = network_address.validate();
    assert!(res);

    let network_address =
        NetworkAddress("0x01.bsc/cx9876543210fedcba9876543210fedcba98765432".to_string());
    let res = network_address.validate();
    assert!(!res);

    let network_address =
        NetworkAddress("0x01.icon/wx9876543210fedcba9876543210fedcba98765432".to_string());
    let res = network_address.validate();
    assert!(!res);

    let network_address =
        NetworkAddress("0x01.icon/cx9876543210fedcba9876543210fedcba9876542".to_string());
    let res = network_address.validate();
    assert!(!res);

    let network_address =
        NetworkAddress("0x01.icon/cx9876543210fedcba9876543210fedcba9876543_".to_string());
    let res = network_address.validate();
    assert!(!res);
}
