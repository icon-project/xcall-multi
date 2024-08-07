#![allow(non_snake_case)]

use anchor_lang::prelude::*;

#[event]
pub struct SendMessage {
    pub targetNetwork: String,
    pub connSn: u128,
    pub msg: Vec<u8>,
}
