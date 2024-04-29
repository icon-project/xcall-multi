module xcall::centralized_state {
    use std::string::{Self, String};
    use sui::vec_map::{Self, VecMap};
     use xcall::xcall_utils::{Self as utils};
      use sui::coin::{Self,Coin};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use xcall::xcall_state::{ConnCap};
    use sui::bag::{Bag, Self};

    const PackageId:vector<u8> =b"centralized";
     const ENotAdmin:u64=1;

    public fun package_id_str():String {
        string::utf8(PackageId)
    }

     public fun get_state(states:&mut Bag):&mut State {
      let package_id= package_id_str();
      let state:&mut State=bag::borrow_mut(states,package_id);
      state
    }
    

    public struct ReceiptKey has copy, drop, store {
        conn_sn: u128,
        nid: String,
    }

    public struct State has store{ 
        message_fee: VecMap<String, u64>,
        response_fee: VecMap<String, u64>,
        receipts: VecMap<ReceiptKey, bool>,
        xcall: String,
        admin: address,
        conn_sn: u128,
        balance: Balance<SUI>,
        cap:ConnCap,
    }

    public fun create(cap:ConnCap,admin:address): State {
        State {
            message_fee: vec_map::empty<String, u64>(),
            response_fee: vec_map::empty<String, u64>(),
            conn_sn: 0,
            receipts: vec_map::empty(),
            xcall: string::utf8(b""), 
            admin: admin,
            balance:balance::zero(),
            cap
        }
    }

    public(package) fun get_next_conn_sn(self:&mut State):u128 {
        let sn=self.conn_sn+1;
        self.conn_sn=sn;
        sn
    }

    public fun get_fee(self: &State, netId: &String, response: bool): u64 {
        let fee: u64 = if (response == true) {
            utils::get_or_default(&self.message_fee, netId, 0)
                + utils::get_or_default(&self.response_fee, netId, 0)
        } else {
            utils::get_or_default(&self.message_fee, netId, 0)
        };
        fee
    }

    public(package) fun set_fee(self: &mut State, net_id: String, message_fee: u64, response_fee: u64,caller:address) {
        ensure_admin(self,caller);
        vec_map::insert(&mut self.message_fee, net_id, message_fee);
        vec_map::insert(&mut self.response_fee, net_id, response_fee);
    }

    public(package) fun check_save_receipt(self: &mut State, net_id: String, sn: u128) {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        assert!(!vec_map::contains(&self.receipts, &receipt_key), 100);
        vec_map::insert(&mut self.receipts, receipt_key, true);
    }

    public(package) fun get_receipt(self: &State, net_id: String, sn: u128): bool {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        vec_map::contains(&self.receipts, &receipt_key)
    }

    public(package) fun deposit(self:&mut State,balance:Balance<SUI>){
        balance::join(&mut self.balance,balance);

    }

    public(package) fun conn_cap(self:&State):&ConnCap{
                &self.cap
    }

    public(package) fun set_admin(self:&mut State,caller:address,admin:address){
        ensure_admin(self,caller);
        self.admin=admin;
    }

    public(package) fun claim_fees(self:&mut State,caller:address,ctx:&mut TxContext){
        ensure_admin(self,caller);
        let total= self.balance.withdraw_all();
        let coin= coin::from_balance(total,ctx);
        transfer::public_transfer(coin,caller);

    }

    public(package) fun ensure_admin(self:&State,caller:address){
        assert!(self.admin==caller,ENotAdmin);
    }
}