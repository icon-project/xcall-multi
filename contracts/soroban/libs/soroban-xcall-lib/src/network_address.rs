use soroban_sdk::{
    contracttype,
    xdr::{FromXdr, ToXdr},
    Bytes, Env, String,
};

/** SC TYPES */
const SC_DATA_BEGIN: u32 = 8;
const SC_STRING: u8 = 14;
const SEPERATOR: u8 = 47;

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAddress(String);

impl NetworkAddress {
    pub fn new(env: &Env, nid: String, account: String) -> Self {
        let mut address = Bytes::new(&env);

        let nid_slice = Self::get_bytes_from_string(&env, nid);
        let account_slice = Self::get_bytes_from_string(&env, account);

        address.append(&nid_slice);
        address.push_back(SEPERATOR);
        address.append(&account_slice);

        Self(Self::get_string_from_bytes(&env, address))
    }

    pub fn nid(&self, env: &Env) -> String {
        let (nid, _) = self.get_parts(&env);
        Self::get_string_from_bytes(&env, nid)
    }

    pub fn account(&self, env: &Env) -> String {
        let (_, account) = self.get_parts(&env);
        Self::get_string_from_bytes(&env, account)
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn from_string(value: String) -> Self {
        Self(value)
    }

    pub fn parse_network_address(&self, env: &Env) -> (String, String) {
        let (nid, account) = self.get_parts(&env);
        (
            Self::get_string_from_bytes(&env, nid),
            Self::get_string_from_bytes(&env, account),
        )
    }

    pub fn get_parts(&self, env: &Env) -> (Bytes, Bytes) {
        let mut nid = Bytes::new(&env);
        let mut account = Bytes::new(&env);

        let addr_slice = Self::get_bytes_from_string(&env, self.0.clone());

        let mut has_seperator = false;
        for (index, value) in addr_slice.clone().iter().enumerate() {
            if has_seperator {
                account.append(&addr_slice.slice(index as u32..addr_slice.len()));
                break;
            } else if value == SEPERATOR {
                has_seperator = true;
            } else {
                nid.push_back(value)
            }
        }

        if !has_seperator {
            panic!("Invalid network address")
        }

        (nid, account)
    }

    /// It converts string value to xdr and extract the slice of string bytes
    ///
    /// Note: This function assumes that the string is less than 256. If string is more than 256
    /// bytes, this will panic
    ///
    /// Returns:
    /// a `sequence of bytes`
    pub fn get_bytes_from_string(env: &Env, value: String) -> Bytes {
        let bytes = value.to_xdr(&env);

        if bytes.get(6).unwrap() > 0 {
            panic!("Invalid network address length")
        }

        let value_len = bytes.get(7).unwrap();
        let slice = bytes.slice(SC_DATA_BEGIN..value_len as u32 + SC_DATA_BEGIN);
        slice
    }

    /// It converts sequence of bytes to xdr and convert xdr of type `bytes` to `string`
    ///
    /// Returns:
    /// a `String` value
    pub fn get_string_from_bytes(e: &Env, bytes: Bytes) -> String {
        let mut bytes_xdr = bytes.to_xdr(&e);
        bytes_xdr.set(3, SC_STRING);

        String::from_xdr(&e, &bytes_xdr).unwrap()
    }
}
