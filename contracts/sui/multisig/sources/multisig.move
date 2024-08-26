
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
    use sui::{ed25519::ed25519_verify};
    use sui::{ecdsa_k1::secp256k1_verify};
    use sui::{ecdsa_r1::secp256r1_verify};

    /** signature schemes*/
    const FlagED25519 :u8= 0x00;
    const FlagSecp256k1 :u8= 0x01;
    const FlagSecp256r1 :u8= 0x02;
	const FlagMultiSig :u8= 0x03;

    /* hash algorithm*/
    const KECCAK256: u8 = 0x00;
    const SHA256: u8 = 0x01;

    
    public struct Signer has store,drop{
        pub_key:vector<u8>,
        sui_address:address,
        weight:u8
    }

    public fun new_signer(pub_key:vector<u8>,weight:u8):Signer{
       let sui_address = address::from_bytes(hash::blake2b256(&pub_key));
       Signer {
        pub_key,
        sui_address,
        weight
       }
    }

    public struct MultisigWallet has store {
        multisig_address:address,
        signers:vector<Signer>,
        threshold:u16,

    }

    public fun multisig_address(self:&MultisigWallet):address {
        return self.multisig_address
    }

    public struct MultiSignature has drop{
        signatures:vector<vector<u8>>,
        bitmap:u16,
        multi_pubkey:MultiPubKey
      

    }

    public struct MultiPubKey has drop {
        weighted_pubkey:vector<vector<u8>>,
        threshold:u16
    }

    public struct Proposal has store{
        id:u64,
        title:String,
        multisig_address:address,
        tx_data:vector<u8>,
        is_digest:bool,
    }

    public struct Vote has store,drop{
        signature:vector<u8>,
        voter:address,
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
    public fun get_wallets(self:&Storage):&VecMap<address,MultisigWallet>{
        &self.wallets
    }
    public fun get_proposals(self:&Storage):&Table<u64,Proposal>{
        &self.proposals
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
        bytes.push_back(FlagMultiSig);
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

    entry fun register_wallet(storage:&mut Storage,_admin:&AdminCap, pub_keys:vector<String>,weights:vector<u8>,threshold:u16){
        assert!(pub_keys.length()==weights.length());
        assert!(threshold>0);
        let mut pub_keys_bytes:vector<vector<u8>> = vector::empty();
        let mut signers:vector<Signer> = vector::empty();
        let mut i=0;
        while(i < pub_keys.length()){
            let bytes=base64::decode(pub_keys.borrow(i));
            pub_keys_bytes.push_back(bytes);
            let signer_1=new_signer(bytes,*weights.borrow(i));
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
        storage.wallet_proposals.add(multisig_addr, vector::empty<u64>());
    }

    entry fun create_proposal(storage:&mut Storage,title:String,tx_bytes_64:String,multisig_address:address,ctx:&TxContext){
        let tx_bytes=base64::decode(&tx_bytes_64);
        let wallet=storage.wallets.get(&multisig_address);
        assert!(only_member(wallet,ctx.sender())==true);
        let is_digest=tx_bytes.length()==32;
        let proposal_id=get_proposal_id(storage);
        let proposal= Proposal{
            id:proposal_id,
            title:title,
            multisig_address:multisig_address,
            tx_data:tx_bytes,
            is_digest,

        };
        storage.proposals.add(proposal_id,proposal);
        storage.wallet_proposals.borrow_mut(multisig_address).push_back(proposal_id);
       
    }

    entry fun approve_proposal(storage:&mut Storage,proposal_id:u64,raw_signature_64:String,ctx:&TxContext){
        let raw_signature=base64::decode(&raw_signature_64);
        let proposal = storage.proposals.borrow(proposal_id);
        let wallet= storage.wallets.get(&proposal.multisig_address);
        assert!(only_member(wallet,ctx.sender())==true);
        let (index,pubkey)=get_pubkey(wallet,ctx.sender());
        assert!(index!=0);
        assert!(verify_pubkey(&pubkey,&proposal.tx_data,&raw_signature,proposal.is_digest)==true);
        let vote_key=VoteKey{
            proposal_id:proposal_id,
            sui_address:ctx.sender()
        };
        assert!(storage.votes.contains(vote_key)==false);
        storage.votes.add(vote_key, Vote{
             signature:raw_signature,
             voter:ctx.sender()
        });


        
    }

    public fun get_execute_command(storage:&Storage,proposal_id:u64):String{
        let proposal=storage.proposals.borrow(proposal_id);
        let wallet=storage.wallets.get(&proposal.multisig_address);
        let mut signatures:vector<vector<u8>> = vector::empty();

        let mut i=0;
        while( i <wallet.signers.length()){
           
            let signer_1= wallet.signers.borrow(i);
            let key=VoteKey{
               proposal_id:proposal_id,
               sui_address:signer_1.sui_address
            };
            if (storage.votes.contains(key)){
                signatures.push_back(storage.votes.borrow(key).signature)
            };
            i=i+1;

        };

        let multisig= create_multi_signature(&signatures,&wallet.signers,wallet.threshold);
        let multisig_serialized_64= base64::encode(&serialize_multisig(&multisig));
        let mut command=vector::empty<u8>();
        command.append(b"sui client execute-signed-tx --tx-bytes ");
        let tx_data_64= base64::encode(&proposal.tx_data);
        if(proposal.is_digest){
         command.append(b"${ORIGINAL_TX_BYTES}");
        }else {
         command.append(*tx_data_64.as_bytes());
        };
       
        command.append(b" --signatures ");
        command.append(*multisig_serialized_64.as_bytes());

        string::utf8(command)



    }

    fun only_member(wallet:&MultisigWallet,caller:address):bool{
        let mut i=0;
        let mut is_member=false;
        while(i < wallet.signers.length() && is_member==false){
            if (wallet.signers.borrow(i).sui_address==caller){
                is_member=true;
            };
            i=i+1;
        };
        is_member

    }

    fun get_proposal_id(storage:&mut Storage):u64{
        let count=storage.proposal_count+1;
        storage.proposal_count=count;
        count
    }

    public fun create_multi_signature(raw_signatures:&vector<vector<u8>>,signers:&vector<Signer>,threshold:u16):MultiSignature{
        let mut bitmap:u16=0;
        let mut i:u64=0;
        let mut signatures:vector<vector<u8>> = vector::empty();
        let mut weighted_pubkey:vector<vector<u8>> =vector::empty();
        while( i < raw_signatures.length()){
            let (sig,pub,scheme)=split_signature(raw_signatures.borrow(i));
            let mut index= get_pub_key_index(signers,pub);
            assert!(index>0);
            index=index-1;
           
            bitmap =bitmap | (1 << index);
            std::debug::print(&bitmap);
            let mut full_sig:vector<u8> =vector::empty();
            full_sig.push_back(scheme);
            full_sig.append(sig);
            signatures.push_back(full_sig);
            i=i+1;


        };
        i=0;
        while(i < signers.length()){
            let mut pubkey= signers.borrow(i).pub_key;
            pubkey.push_back(signers.borrow(i).weight);
            weighted_pubkey.push_back(pubkey);
            i=i+1;
        };
        let multi_pubkey=MultiPubKey {
            weighted_pubkey,
            threshold
        };
        
        MultiSignature {
            signatures,
            bitmap,
            multi_pubkey
        }




    }

    public fun serialize_multisig(sig:&MultiSignature):vector<u8>{
        let mut serialized:vector<u8> = vector::empty();
        serialized.push_back(FlagMultiSig);
        serialized.push_back(sig.signatures.length() as u8);
        let mut i=0;
        while(i < sig.signatures.length()){
            serialized.append(*sig.signatures.borrow(i));
            i=i+1;
        };
        let bitmap=std::bcs::to_bytes(&sig.bitmap);
        serialized.append(bitmap);
        serialized.push_back(sig.multi_pubkey.weighted_pubkey.length() as u8);
        i=0;
        while( i < sig.multi_pubkey.weighted_pubkey.length()){
            serialized.append(*sig.multi_pubkey.weighted_pubkey.borrow(i));
            i=i+1;
        };
        serialized.append(std::bcs::to_bytes(&sig.multi_pubkey.threshold));
        serialized
    }

    fun get_pub_key_index(signers:&vector<Signer>,pubkey:vector<u8>):u8{
        let mut index:u64=0;
        while(index < signers.length()){
            let signer_pubkey=signers.borrow(index).pub_key;
            let public_key=slice_vector(&signer_pubkey,1,signer_pubkey.length()-1);
            if(public_key == pubkey){
                return ((index+1) as u8)
            };
            index=index+1;
        };
        0u8


    }

    public fun verify_pubkey(key:&vector<u8>,data:&vector<u8>,raw_signature:&vector<u8>,is_digest:bool):bool{
        let flag=*key.borrow(0);
        let public_key=slice_vector(key,1,key.length()-1);
        let (signature,pub,_scheme)= split_signature(raw_signature);
        assert!(public_key==pub);
        let digest= if(is_digest==true){
            data
        }else {
             let intent_msg= get_intent_message(data);
             let digest= hash::blake2b256(&intent_msg);
             &digest
        };
        let verify= if(flag==FlagED25519){

            ed25519_verify(&signature,&public_key,digest)

        }else if(flag==FlagSecp256k1){
            
            secp256k1_verify(&signature,&public_key,digest,SHA256)

        }else if (flag==FlagSecp256r1){

            secp256r1_verify(&signature,&public_key,digest,SHA256)
        }else {
            return false
        };
        verify

    }

    
    fun get_intent_message(msg:&vector<u8>):vector<u8>{
        let mut intent_message:vector<u8> =vector::empty();
        intent_message.push_back(0x00);
        intent_message.push_back(0x00);
        intent_message.push_back(0x00);
        intent_message.append(*msg);
        intent_message

    }

    fun split_signature(raw_signature:&vector<u8>):(vector<u8>,vector<u8>,u8){
        let scheme=*raw_signature.borrow(0);
        let length= if (scheme==FlagED25519){32}else{33};
        let signature=slice_vector(raw_signature,1,64);
        let pubkey=slice_vector(raw_signature,raw_signature.length()-length,length);
        return (signature,pubkey,scheme)
    }

    fun get_pubkey(wallet:&MultisigWallet,caller:address):(u64,vector<u8>){
        let mut i=0;
        while(i <wallet.signers.length()){
            if(wallet.signers.borrow(i).sui_address==caller) return (i+1,wallet.signers.borrow(i).pub_key);
            i=i+1;
        };
        (0,x"00")

    }

    public fun slice_vector(vec: &vector<u8>, start: u64, length: u64): vector<u8> {
        let mut result = vector::empty<u8>();
        let mut i = 0;
        while (i < length) {
            let value = *vector::borrow(vec, start + i);
            vector::push_back(&mut result, value);
            i = i + 1;
        };
        result
    }

    #[test_only] use sui::test_scenario::{Self,Scenario};
    #[test_only]
    public fun init_state(admin:address,mut scenario:Scenario):Scenario{
     init(scenario.ctx());
     scenario.next_tx(admin);
     scenario
    }


}
#[test_only]
module multisig::tests {
    use sui::hash::{Self};
    use multisig::multisig::{create_multisig_address,verify_pubkey,Signer,new_signer,create_multi_signature,serialize_multisig};
    

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

    #[test]
    fun test_verify_pubkey(){
        let data=x"00000200203fab45fb191ca013a74ccfc3b7d5ed27a3ef6dce79adc4d1e39555f01a361bf801001b61730f57e4d64241cecb40ac259c58ced75018bd231acdbe463ddb9ac99176480000000000000020e12acf2025db82d420e8696d3645dfd25c6542382883f6e3762cc655791a007e01010101010001000029a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c20458901c2a6df77449ebce38397d97785f52c2341abafc21c47523e9cfb71d141d5df614800000000000000209e4e3d3f48881797ecc3690b237e0353e16a8be1cd79ac75210657229942e12b29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589e80300000000000078be2d000000000000";


        let mut public_keys:vector<vector<u8>> = vector::empty();
        public_keys.push_back(x"01033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814");
        public_keys.push_back(x"00016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481");
        public_keys.push_back(x"01034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804");
        public_keys.push_back(x"02023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e");
        public_keys.push_back(x"0103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598");
        
        let mut signatures:vector<vector<u8>> = vector::empty();
        signatures.push_back(x"0196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc76033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814");
        signatures.push_back(x"00339e5f6df9aeac4e902767679407b1a6ae0e14db0036c4b19d8b45825036099299bb1948ef907b42df3a250b24395d49d1ff24e75748fadd32892934b22aa70e016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481");
        signatures.push_back(x"019d51371b1fe243fc6bf92b0a6b58feb5cc7bfc0a90ee8900d2f5db8c22f7122f564dda95aff1ea5da752226686ff89e5c7c7e175d752bdc95d42c2bcf71cf160034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804");
        signatures.push_back(x"0222194e51b82f0a13293c4ca9db40ef1a8c4115ace0047466627ece5c0e229766479486a6423f8332b13d8a13f2baba5d72f86c4a5d55c5b8c2c134f2a367299e023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e");
        signatures.push_back(x"016a21ed6ace0e5c28dd6396dc004a8a9321f9a4c446f037275cdbaad81797c544570cd9e470d4f46d11eb512d8b8a2b6f00c16356e9f2505ead5bafd492387e4303b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598");
        let mut i=0;
        while( i < public_keys.length()){
            std::debug::print(public_keys.borrow(i));
            let signature=signatures.borrow(i);
            let verify=verify_pubkey(public_keys.borrow(i),&data,signature,false);
            i=i+1;
            
            assert!(verify==true);
        };

    }

    #[test]
    fun test_create_multisignature(){

        let mut signers:vector<Signer> =vector::empty();
        signers.push_back(new_signer(x"01033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814",1));
        signers.push_back(new_signer(x"00016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481",1));
        signers.push_back(new_signer(x"01034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804",1));
        signers.push_back(new_signer(x"02023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e",1));
        signers.push_back(new_signer(x"0103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598",1));
        let mut raw_signatures:vector<vector<u8>> = vector::empty();
        raw_signatures.push_back(x"0196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc76033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814");
        raw_signatures.push_back(x"00339e5f6df9aeac4e902767679407b1a6ae0e14db0036c4b19d8b45825036099299bb1948ef907b42df3a250b24395d49d1ff24e75748fadd32892934b22aa70e016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481");
        raw_signatures.push_back(x"019d51371b1fe243fc6bf92b0a6b58feb5cc7bfc0a90ee8900d2f5db8c22f7122f564dda95aff1ea5da752226686ff89e5c7c7e175d752bdc95d42c2bcf71cf160034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804");

        let multisignature= create_multi_signature(&raw_signatures,&signers,3_u16);

        std::debug::print(&serialize_multisig(&multisignature));

        let expected=x"03030196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc7600339e5f6df9aeac4e902767679407b1a6ae0e14db0036c4b19d8b45825036099299bb1948ef907b42df3a250b24395d49d1ff24e75748fadd32892934b22aa70e019d51371b1fe243fc6bf92b0a6b58feb5cc7bfc0a90ee8900d2f5db8c22f7122f564dda95aff1ea5da752226686ff89e5c7c7e175d752bdc95d42c2bcf71cf16007000501033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba907261268140100016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f4810101034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d638040102023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e010103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598010300";
        assert!(expected==serialize_multisig(&multisignature));



    }

    #[test]
    fun test_create_multisignature_2(){

        let mut signers:vector<Signer> =vector::empty();
        signers.push_back(new_signer(x"01033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814",1));
        signers.push_back(new_signer(x"00016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f481",1));
        signers.push_back(new_signer(x"01034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804",1));
        signers.push_back(new_signer(x"02023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e",1));
        signers.push_back(new_signer(x"0103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598",1));
        let mut raw_signatures:vector<vector<u8>> = vector::empty();
        raw_signatures.push_back(x"0196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc76033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814");
        raw_signatures.push_back(x"019d51371b1fe243fc6bf92b0a6b58feb5cc7bfc0a90ee8900d2f5db8c22f7122f564dda95aff1ea5da752226686ff89e5c7c7e175d752bdc95d42c2bcf71cf160034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d63804");
        raw_signatures.push_back(x"016a21ed6ace0e5c28dd6396dc004a8a9321f9a4c446f037275cdbaad81797c544570cd9e470d4f46d11eb512d8b8a2b6f00c16356e9f2505ead5bafd492387e4303b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598");

        let multisignature= create_multi_signature(&raw_signatures,&signers,3_u16);

        std::debug::print(&serialize_multisig(&multisignature));

        let expected=x"03030196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc76019d51371b1fe243fc6bf92b0a6b58feb5cc7bfc0a90ee8900d2f5db8c22f7122f564dda95aff1ea5da752226686ff89e5c7c7e175d752bdc95d42c2bcf71cf160016a21ed6ace0e5c28dd6396dc004a8a9321f9a4c446f037275cdbaad81797c544570cd9e470d4f46d11eb512d8b8a2b6f00c16356e9f2505ead5bafd492387e4315000501033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba907261268140100016e02b7a72826d951789791f3c053f92af9bc28b27a8e2c60fc474ca536f4810101034195b5a61eeebee6fd2b959ef6e23f7393b2b0717b6458eebe6ff72778d638040102023ecb8ff6cf8eb4748ef6fb062eb52f862b093eb3a42d629d89e43d9645108f9e010103b13036d4a4adf7b9c36c5cd165613c82734e7541e3a47864fb5b8727e7920598010300";
        assert!(expected==serialize_multisig(&multisignature));



    }

   







}


