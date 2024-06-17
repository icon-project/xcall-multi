use anchor_lang::prelude::borsh;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

use crate::error::NetworkError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
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
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct NetworkAddress(String);

impl NetworkAddress {
    pub fn new(nid: &str, address: &str) -> Self {
        Self(format!("{}/{}", nid, address))
    }

    pub fn nid(&self) -> NetId {
        NetId(self.get_parts()[0].to_string())
    }

    pub fn account(&self) -> String {
        self.get_parts()[1].to_owned()
    }

    pub fn parse_network_address(&self) -> (NetId, String) {
        let parts = self.get_parts();
        (NetId(parts[0].to_owned()), parts[1].to_owned())
    }

    pub fn get_parts(&self) -> Vec<&str> {
        let parts = self.0.split('/').collect::<Vec<&str>>();
        if parts.len() != 2 {
            panic!("Invalid Network Address");
        }
        parts
    }
}

impl ToString for NetworkAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for NetworkAddress {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(NetworkError::InvalidNetworkAddress);
        }
        let na = format!("{}/{}", parts[0], parts[1]);
        Ok(Self(na))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_network_address() {
        let na = String::from("0x1.icon/hx124324687");
        let parsed = NetworkAddress::from_str(&na).unwrap();
        assert_eq!(String::from("0x1.icon"), parsed.nid().to_string());
        assert_eq!(String::from("hx124324687"), parsed.account());
    }
}
