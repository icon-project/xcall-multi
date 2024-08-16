
module multisig::multisig {
    use std::vector::{Self};
    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use sui::event;
    use sui::hash::{Self};
    use sui::vec_map::{Self, VecMap};
    use sui::table::{Table,Self};
    use sui::bcs::{Self};
    use sui::address::{Self};
    use multisig::base64::{Self};

    
    public struct Signer has store{
        pub_key:vector<u8>,
        sui_address:address,
        weight:u8
    }

    public struct MultisigWallet has store {
        multisig_address:address,
        signers:vector<Signer>,
        threshold:u16,

    }

    public struct Proposal has store{
        id:u64,
        title:String,
        multisig_address:address,
        tx_data:vector<u8>,
    }

    public struct Vote has store{
        id:u64,
        signature:vector<u8>,
        pub_key:address,
    }

    public struct VoteKey has store,drop,copy{
        proposal_id:u64,
        sui_address:address,
    }


    public struct Storage has key,store{
        id:UID,
        wallets:VecMap<address,MultisigWallet>,
        wallet_proposals:Table<address,vector<u64>>,
        proposals:Table<u64,Proposal>,
        votes:Table<VoteKey,Vote>,
        proposal_count:u64,



    }
    public struct AdminCap has key,store {
        id: UID
    }


    fun init(ctx: &mut TxContext) {
        let admin = AdminCap {
            id: object::new(ctx),
        };
        let storage = Storage {
            id:object::new(ctx),
            wallets:vec_map::empty<address,MultisigWallet>(),
            wallet_proposals: table::new(ctx),
            proposals:table::new(ctx),
            votes: table::new(ctx),
            proposal_count:0u64,
        };
        transfer::transfer(admin, tx_context::sender(ctx));
        transfer::share_object(storage);

    }

    

    public fun create_multisig_address(pubkeys:vector<vector<u8>>,weights:vector<u8>,threshold:u16):address{
        let mut bytes= vector::empty<u8>();
        bytes.push_back(0x03);
        let threshold_bytes=bcs::to_bytes(&threshold);
        bytes.append(threshold_bytes);
        let mut i=0;
        while(i < pubkeys.length()){
           bytes.append(*pubkeys.borrow(i));
           bytes.push_back(*weights.borrow(i));
           i=i+1;
        };

        let address_bytes=hash::blake2b256(&bytes);
        address::from_bytes(address_bytes)

    }

    entry fun register_wallet(storage:&mut Storage,pub_keys:vector<String>,weights:vector<u8>,threshold:u16){
        let mut pub_keys_bytes:vector<vector<u8>> = vector::empty();
        let mut signers:vector<Signer> = vector::empty();
        let mut i=0;
        while(i < pub_keys.length()){
            let bytes=base64::decode(pub_keys.borrow(i));
            pub_keys_bytes.push_back(bytes);
            let sui_address= address::from_bytes(hash::blake2b256(&bytes));
            let signer_1= Signer {
                pub_key:bytes,
                sui_address:sui_address,
                weight:*weights.borrow(i),
            };
            signers.push_back(signer_1);
            i=i+1;
        };
        let multisig_addr= create_multisig_address(pub_keys_bytes,weights,threshold);
        let multisig_wallet= MultisigWallet{
            multisig_address:multisig_addr,
            signers:signers,
            threshold:threshold,
        };
        storage.wallets.insert(multisig_addr,multisig_wallet);
    }

    entry fun create_proposal(storage:&mut Storage,title:String,tx_bytes:vector<u8>,multisig_address:address,ctx:&mut TxContext){
        let wallet=storage.wallets.get(&multisig_address);
        assert!(only_member(wallet,ctx.sender())==true);
        let proposal_id=get_proposal_id(storage);
        let proposal= Proposal{
            id:proposal_id,
            title:title,
            multisig_address:multisig_address,
            tx_data:tx_bytes,

        };
        storage.proposals.add(proposal_id,proposal);
        storage.wallet_proposals.borrow_mut(multisig_address).push_back(proposal_id);
       
    }

    entry fun approve_proposal(storage:&mut Storage,proposal_id:u64,signature:vector<u8>,ctx:&mut TxContext){
        
    }

    fun only_member(wallet:&MultisigWallet,caller:address):bool{
        let mut i=0;
        let mut is_member=false;
        while(i < wallet.signers.length() && is_member==false){
            if (wallet.signers.borrow(i).sui_address==caller){
                is_member=true;
            };
        };
        is_member

    }

    fun get_proposal_id(storage:&mut Storage):u64{
        let count=storage.proposal_count+1;
        storage.proposal_count=count;
        count
    }

}
#[test_only]
module multisig::tests {
    use sui::hash::{Self};
    use multisig::multisig::{create_multisig_address};
    #[test]
    fun test_public__key_address(){
        let pubkey = x"01033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814";
        let sui_address=hash::blake2b256(&pubkey);
        assert!(x"29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589"==sui_address);


    }

    #[test]
    fun test_multisig_address(){
        let mut public_keys:vector<vector<u8>> =vector::empty();
        public_keys.push_back(x"01033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814");
        public_keys.push_back(x"00016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481");
        public_keys.push_back(x"01034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804");
        public_keys.push_back(x"02023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e");
        public_keys.push_back(x"0103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598");
        let mut weights:vector<u8> = vector::empty();
        weights.push_back(1u8);
        weights.push_back(1u8);
        weights.push_back(1u8);
        weights.push_back(1u8);
        weights.push_back(1u8);
        let threshold:u16=3u16;

        let sui_addr= create_multisig_address(public_keys,weights,threshold);
        std::debug::print(&sui_addr);
        let expected= @0x34f45f30d3af0393474ce42fc7a1de48aa8a9ddf03383062d8fcd1842d627a2f;
        assert!(expected==sui_addr);



    }




}


