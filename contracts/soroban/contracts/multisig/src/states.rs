use soroban_sdk::{ Address, Env, String, Vec, contracttype};

#[contracttype]
pub enum StorageKey {
    Proposals(u32),
    Count,
    MultisigWallet(Address),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Signature {
    pub address: Address,
    pub signature: String
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MultisigWallet {
    pub signers: Vec<Address>,
    pub threshold: u32
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub proposal_data: String,
    pub approved: bool,
    pub signatures: Vec<Signature>,
    pub wallet: Address
}

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyVoted = 1,
    NotAValidSigner = 2,
    AlreadyInitialized = 3,
}

const DAY_IN_LEDGERS: u32 = 17280; // assumes 5s a ledger

const LEDGER_THRESHOLD_INSTANCE: u32 = DAY_IN_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + DAY_IN_LEDGERS; // ~ 31 days

const LEDGER_THRESHOLD_REQUEST: u32 = DAY_IN_LEDGERS * 3; // ~ 3 days
const LEDGER_BUMP_REQUEST: u32 = LEDGER_THRESHOLD_REQUEST + DAY_IN_LEDGERS; // ~ 4 days

pub fn is_signer(env: &Env, wallet: &Address, address: Address) -> bool {
    let multisig: MultisigWallet = env.storage().instance().get(&StorageKey::MultisigWallet(wallet.clone())).unwrap();
    multisig.signers.contains(&address)
}

pub fn set_count(env: &Env, count: u32) {
    env.storage().instance().set(&StorageKey::Count, &count);
}

pub fn get_count(env: &Env) -> u32 {
    env.storage().instance().get(&StorageKey::Count).unwrap()
}
pub fn increase_count(env: &Env) {
    let count = get_count(env);
    set_count(env, count+1);
}

pub fn set_proposal(env: &Env, proposal_id: u32, proposal: Proposal) {
    let key = StorageKey::Proposals(proposal_id);
    env.storage().temporary().set(&key, &proposal);
    extend_temporary_request(env, &key);
}

pub fn get_proposal(env: &Env, proposal_id: u32) -> Proposal {
    let key = StorageKey::Proposals(proposal_id);
    env.storage().temporary().get(&key).unwrap()
}

pub fn set_multisig_wallet(env: &Env, wallet: Address, multisig: MultisigWallet) {
    env.storage().persistent().set(&StorageKey::MultisigWallet(wallet), &multisig);
}

pub fn get_multisig_wallet(env: &Env, wallet: Address) -> MultisigWallet {
    let multisig: MultisigWallet = env.storage().persistent().get(&StorageKey::MultisigWallet(wallet)).unwrap();
    multisig
}

pub fn get_threshold(env: &Env, wallet: Address) -> u32 {
    let multisig: MultisigWallet = env.storage().persistent().get(&StorageKey::MultisigWallet(wallet)).unwrap();
    multisig.threshold
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Count)
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn extend_temporary_request(e: &Env, key: &StorageKey) {
    e.storage()
        .temporary()
        .extend_ttl(key, LEDGER_THRESHOLD_REQUEST, LEDGER_BUMP_REQUEST);
}