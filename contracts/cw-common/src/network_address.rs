use cosmwasm_std::StdResult;

pub struct NetworkAddress;

//TODO: use network address library from ibc-integration/contracts/cosmwasm-vm/cw-common
impl NetworkAddress {
    pub fn parse_network_address(_str: &str) -> StdResult<(&str, &str)> {
        let mut iter = _str.splitn(2, "://");
        let _ = iter.next().unwrap_or("");
        let mut account = iter.next().unwrap_or("").splitn(2, "/");
        let network = account.next().unwrap_or("");
        let address = account.next().unwrap_or("");
        Ok((network, address))
    }

    pub fn parse_protocol_address(_str: &str) -> StdResult<(&str, &str)> {
        let mut iter = _str.splitn(2, "://");
        let protocol = iter.next().unwrap_or("");
        let account = iter.next().unwrap_or("");
        Ok((protocol, account))
    }

    pub fn protocol_address(_str: &str) -> StdResult<&str> {
        let mut iter = _str.splitn(2, "://");
        let _ = iter.next().unwrap_or("");
        let mut address = iter.next().unwrap_or("").splitn(2, "/");
        let network = address.next().unwrap_or("");
        Ok(network)
    }

    pub fn get_network_address(protocol: &str, network: &str, account: &str) -> String {
        format!(
            "{}://{}/{}",
            protocol,
            network,
            account
        )
    }
}

mod tests {
    #[test]
    fn test_parse_btp_address() {
        let btp_address = "btp://0x38.bsc/0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798";
        let (network, account) = super::NetworkAddress::parse_network_address(btp_address).unwrap();
        assert_eq!(network, "0x38.bsc");
        assert_eq!(account, "0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798");
    }

    #[test]
    fn test_parse_network_address() {
        let btp_address = "btp://0x38.bsc/0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798";
        let (network, account) = super::NetworkAddress::parse_protocol_address(btp_address).unwrap();
        assert_eq!(network, "btp");
        assert_eq!(
            account,
            "0x38.bsc/0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798"
        );
    }

    #[test]
    fn test_network_address() {
        let btp_address = "btp://0x38.bsc/0x034AaDE86BF402F023Aa17E5725fABC4ab9E9798";
        let network = super::NetworkAddress::protocol_address(btp_address).unwrap();
        assert_eq!(network, "0x38.bsc");
    }
}
