use super::*;
use bech32::ToBase32;
use cosmwasm_std::Addr;

pub fn keccak256(input: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(input);
    let out: [u8; 32] = hasher.finalize().to_vec().try_into().unwrap();
    out
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    sha2::Sha256::digest(&data).to_vec()
}

pub fn ripemd160(data: &[u8]) -> Vec<u8> {
    use ripemd::Digest;
    ripemd::Ripemd160::digest(&data).to_vec()
}

pub fn pubkey_to_address(pubkey: &[u8], prefix: &str) -> Result<Addr, ContractError> {
    use bech32::{encode, Variant};
    let sha256_hash = sha256(pubkey);
    let ripemd160_hash = ripemd160(&sha256_hash);

    let base32_data_slice = ripemd160_hash.as_slice();
    let base32_data = base32_data_slice.to_base32();

    let encoded = encode(prefix, base32_data, Variant::Bech32)
        .map_err(|_| ContractError::InvalidSignature)?;
    Ok(Addr::unchecked(encoded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_pubkey_to_address() {
        // Test case 1: 33-byte compressed public key (Archway prefix)
        let pubkey_hex = "03c414dbe1812580741f0ebe71830226f8304e74daa5a1fad32d1e97da2d719493";
        let pubkey = hex::decode(pubkey_hex).unwrap();
        let address = pubkey_to_address(&pubkey, "archway").unwrap();
        assert_eq!(address, "archway1a06mhyewfajqcf5dujzyqd6ps9a4v9usauxxqw");
    }
}
