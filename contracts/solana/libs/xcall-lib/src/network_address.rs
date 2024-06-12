// TODO: Net id might not be required
use borsh::{BorshDeserialize, BorshSerialize};

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

#[derive(Clone,BorshDeserialize,BorshSerialize)]
pub struct NetworkAddress {
    pub net: String,
    pub account: String,
}

impl NetworkAddress {
    pub fn new(net: String, account: String) -> Self {
        Self { net, account }
    }

    pub fn from_str(value: &String) -> Self {
        let mut iter = value.split('/');
        NetworkAddress {
            net: iter.next().unwrap_or("").to_string(),
            account: iter.next().unwrap_or("").to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}/{}", &self.net, &self.account)
    }
}

// impl  FromStr for NetworkAddress {
//     type Err;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let mut iter = s.split('/');
//         Ok(NetworkAddress {
//             net: iter.next().unwrap_or("").to_string(),
//             account: iter.next().unwrap_or("").to_string(),
//         })
//     }
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_split_network_address() {
        let na = String::from("0x1.icon/hx124324687");
        let parsed = NetworkAddress::from_str(&na);
        assert_eq!(String::from("0x1.icon"), parsed.net);
        assert_eq!(String::from("hx124324687"), parsed.account);
    }
}

