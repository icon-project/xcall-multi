#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
mod states;
use states::{is_signer, ContractError, MultisigWallet, Proposal, Signature, StorageKey};
#[contract]
pub struct ProposalContract;

#[contractimpl]
impl ProposalContract {

    pub fn init(env: Env) -> Result<(), ContractError> {
        if states::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        states::set_count(&env, 0);
        Ok(())
    }


    pub fn add_multisig_wallet(env: Env, wallet: Address, signers: Vec<Address>, threshold: u32) -> Result<(), ContractError> {
        let multisig = MultisigWallet {
            signers,
            threshold
        };
        states::set_multisig_wallet(&env, wallet, multisig);
        Ok(())
    }

    /// Create a proposal. This proposal is identified by a proposal_id that is increased with each proposal.
    /// The proposal is associated with a wallet, which is used to verify signers.
    /// The proposal is approved after a number of signatures equal to the threshold of the wallet is reached.
    pub fn create_proposal(env: Env, sender : Address, proposal_data: String, wallet: Address) -> Result<(), ContractError> {
        sender.require_auth();
        if !is_signer(&env, &wallet, sender) {
            return Err(ContractError::NotAValidSigner);
        }
        let proposal_id = states::get_count(&env);
        let proposal = Proposal {
            proposal_id,
            proposal_data,
            approved: false,
            signatures: Vec::new(&env),
            wallet
        };
        states::increase_count(&env);
        states::set_proposal(&env, proposal_id, proposal);
        Ok(())
    }

    
    /// Add a signature to a proposal. 
    /// The proposal is approved after a number of signatures equal to the threshold of the wallet is reached.
    /// The function returns an error if the proposal has expired, or if the sender is not a valid signer, or if the sender has already voted.
    pub fn add_approval_signature(env: Env, proposal_id: u32, sender: Address, signature: String) -> Result<(), ContractError> {   
        sender.require_auth(); 
        let key = states::get_proposal(&env, proposal_id);
        if states::is_proposal_expired(&env, proposal_id) {
            return Err(ContractError::ProposalExpired);
        }
        let mut proposal: Proposal = env.storage().temporary().get(&key).unwrap();
        if !is_signer(&env, &proposal.wallet, sender.clone()) {
            return Err(ContractError::NotAValidSigner);
        }
        let new_signature = Signature {
            address:sender,
            signature
        };

        if proposal.signatures.contains(&new_signature) {
            return Err(ContractError::AlreadyVoted);
        }
        proposal.signatures.push_back(new_signature);
        let threshold = states::get_threshold(&env, proposal.wallet.clone());
        if proposal.signatures.len() >= threshold {
            proposal.approved = true;
        }

        states::set_proposal(&env, proposal_id, proposal);
        Ok(())
    }

    /// Returns all active proposals. A proposal is active if it has not expired.
    pub fn get_active_proposals(env: Env) -> Vec<Proposal> {
        let count = states::get_count(&env);
        let mut proposals = Vec::new(&env);
        for i in 0..count {
            let key = StorageKey::Proposals(i);
            if !env.storage().temporary().has(&key) {
                continue;
            }
            let proposal: Proposal = env.storage().temporary().get(&key).unwrap();
            proposals.push_back(proposal);
        }
        proposals
    }

    pub fn get_multisig_wallet(env: Env, wallet: Address) -> MultisigWallet {
        states::get_multisig_wallet(&env, wallet)
    }

    pub fn extend_instance(e: Env) {
        states::extend_instance(&e);
    }

}