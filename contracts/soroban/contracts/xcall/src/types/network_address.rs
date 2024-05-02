extern crate alloc;

use soroban_sdk::{contracttype, Env, String};

#[contracttype]
#[derive(Clone, PartialEq, PartialOrd)]
pub struct NetId(pub String);

impl From<String> for NetId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<NetId> for String {
    fn from(value: NetId) -> Self {
        value.0
    }
}

#[contracttype]
#[derive(Clone)]
pub struct NetworkAddress(String);

impl NetworkAddress {
    pub fn new(env: &Env, nid: String, account: String) -> Self {
        let n = Self::to_alloc_string(&nid);
        let a = Self::to_alloc_string(&account);
        let n_a = alloc::format!("{}/{}", n, a);

        Self(String::from_str(&env, &n_a))
    }

    pub fn nid(&self, env: &Env) -> NetId {
        NetId(String::from_str(&env, self.get_parts()[0].as_str()))
    }

    pub fn account(&self, env: &Env) -> String {
        String::from_str(&env, self.get_parts()[1].as_str())
    }

    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn parse_network_address(&self, e: &Env) -> (NetId, String) {
        let network_address = self.get_parts();
        let nid = NetId(String::from_str(&e, network_address[0].as_str()));
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
