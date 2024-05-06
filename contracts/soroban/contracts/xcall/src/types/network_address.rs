use soroban_sdk::{contracttype, Env, String};

extern crate alloc;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkAddress(String);

impl NetworkAddress {
    pub fn new(env: &Env, nid: String, account: String) -> Self {
        let network_id = Self::to_alloc_string(&nid);
        let account = Self::to_alloc_string(&account);
        let network_address = alloc::format!("{}/{}", network_id, account);

        Self(String::from_str(&env, &network_address))
    }

    pub fn nid(&self, env: &Env) -> String {
        String::from_str(&env, self.get_parts()[0].as_str())
    }

    pub fn account(&self, env: &Env) -> String {
        String::from_str(&env, self.get_parts()[1].as_str())
    }

    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn to_network_address(address: String) -> Self {
        NetworkAddress(address)
    }

    pub fn parse_network_address(&self, e: &Env) -> (String, String) {
        let network_address = self.get_parts();
        let nid = String::from_str(&e, network_address[0].as_str());
        let account = String::from_str(&e, network_address[1].as_str());

        (nid, account)
    }

    pub fn get_parts(&self) -> alloc::vec::Vec<alloc::string::String> {
        let network_address = Self::to_alloc_string(&self.0);

        let parts = network_address
            .split("/")
            .map(alloc::string::ToString::to_string)
            .collect::<alloc::vec::Vec<alloc::string::String>>();

        if parts.len() != 2 {
            panic!("Invalid network address");
        };

        parts
    }

    pub fn to_alloc_string(value: &String) -> alloc::string::String {
        let len = value.len() as usize;
        let mut slice = alloc::vec![0u8; len];
        value.copy_into_slice(&mut slice);

        let network_address = alloc::string::String::from_utf8(slice).unwrap();

        network_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::setup::*;

    #[test]
    fn test_network_address_new() {
        let env = Env::default();

        let network_id = String::from_str(&env, "icon");
        let account = String::from_str(&env, "hx9b79391cefc9a64dfda6446312ebb7717230df5b");
        let network_address = NetworkAddress::new(&env, network_id, account);

        assert_eq!(
            network_address,
            NetworkAddress(String::from_str(
                &env,
                "icon/hx9b79391cefc9a64dfda6446312ebb7717230df5b"
            ))
        )
    }

    #[test]
    fn test_parse_network_address() {
        let env = Env::default();

        let network_address = get_dummy_network_address(&env);
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
            network_address.as_string(),
            &String::from_str(
                &env,
                "stellar/GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU"
            )
        )
    }

    #[test]
    #[should_panic(expected = "Invalid network address")]
    fn test_parse_network_address_fail() {
        let env = Env::default();

        let network_address = NetworkAddress(String::from_str(
            &env,
            "icon/hx9b79391cefc9a64dfda6446312ebb7717230df5b/",
        ));
        network_address.parse_network_address(&env);
    }
}
