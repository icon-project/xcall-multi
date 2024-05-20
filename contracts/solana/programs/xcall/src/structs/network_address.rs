#[derive(Debug, Clone)]
pub struct NetworkAddress {
    pub net: String,
    pub account: String,
}

impl NetworkAddress {
    pub fn new(net: String, account: String) -> Self {
        Self { net, account }
    }

    pub fn split(value: String) -> Self {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_split_network_address() {
        let na = String::from("0x1.icon/hx124324687");
        let parsed = NetworkAddress::split(na);
        assert_eq!(String::from("0x1.icon"), parsed.net);
        assert_eq!(String::from("hx124324687"), parsed.account);
    }
}
