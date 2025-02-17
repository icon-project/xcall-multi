use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Xcall,
    Relayer,
    Admin,
    UpgradeAuthority,
    Xlm,
    ConnSn,
    NetworkFee(String),
    Receipts(String, u128),
    Validators,
    ValidatorThreshold
}

#[contracttype]
pub struct InitializeMsg {
    pub relayer: Address,
    pub admin: Address,
    pub native_token: Address,
    pub xcall_address: Address,
    pub upgrade_authority: Address,
}

#[contracttype]
pub struct NetworkFee {
    pub message_fee: u128,
    pub response_fee: u128,
}

impl NetworkFee {
    pub fn default() -> Self {
        Self {
            message_fee: 0,
            response_fee: 0,
        }
    }
}
