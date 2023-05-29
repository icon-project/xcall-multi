pub mod types {
    pub struct CrossTransfer {
        pub from: String,
        pub to: String,
        pub value: u128,
        pub data: Vec<u8>,
    }

    pub struct CrossTransferRevert {
        pub from: String,
        pub value: u128,
    }
}
