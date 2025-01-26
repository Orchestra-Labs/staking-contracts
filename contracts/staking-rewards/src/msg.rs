use crate::state::RewardsDistributionByToken;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::DenomUnit;
use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub staking_orchestrator_addr: String,
    pub reward_token: DenomUnit,
    pub rewards_distribution: Vec<RewardsDistributionByToken>,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        staking_orchestrator_addr: Option<String>,
        reward_token: Option<DenomUnit>,
        rewards_distribution: Option<Vec<RewardsDistributionByToken>>,
    },
    UpdateRewardsState,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}