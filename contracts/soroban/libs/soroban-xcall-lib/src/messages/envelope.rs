use soroban_sdk::{contracttype, String, Vec};

use super::AnyMessage;

#[contracttype]
pub struct Envelope {
    pub message: AnyMessage,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}
