use soroban_sdk::{contracttype, BytesN, String};

#[contracttype]
pub enum StorageKey {
    DepositId,
    Nid,
    NativeToken,
    ProtocolFee,
    FeeHandler,
    Admin,
    UpgradeAuthority,
    Version,
    ConnSn,
    Orders(u128),
    PendingOrderAmount(u128),
    PendingFills(BytesN<32>),
    FinishedOrders(BytesN<32>),
    Receipts(String, u128),
}
