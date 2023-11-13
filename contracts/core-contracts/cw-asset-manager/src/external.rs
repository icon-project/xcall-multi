use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub enum StakingQueryMsg {
    ConfigInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Should be multi-sig, is able to update the config
    pub admin: String,
    /// Should be single-sig, is able to pause the contract
    pub pause_admin: String,
    /// This is the denomination we can stake (and only one we accept for payments)
    pub bond_denom: String,
    /// Liquid token address
    pub liquid_token_addr: String,
    /// Swap contract address
    pub swap_contract_addr: String,
    /// Liquid Treasury contract address
    pub treasury_contract_addr: String,
    /// Team wallet address
    pub team_wallet_addr: String,
    /// percentage of commission taken off of staking rewards
    pub commission_percentage: u16,
    /// percentage of rewards for the team
    pub team_percentage: u16,
    /// percentage of rewards for the liquidity providers
    pub liquidity_percentage: u16,
    /// Delegations preferences for a whitelist of validators, each validator has a delegation percentage
    pub delegations: Vec<DelegationPercentage>,
    /// contract state (active/paused)
    pub contract_state: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DelegationPercentage {
    pub validator: String,
    pub percentage: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Sell a given amount of asset
    Swap {
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
}
