use crate::state::RewardsDistributionByToken;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{DenomUnit, Uint128, Uint64};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

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

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ListPoolStatesResponse)]
    AllPoolStates {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub staking_orchestrator_addr: String,
    pub reward_token: DenomUnit,
    pub rewards_distribution: Vec<RewardsDistributionByToken>,
}

#[cw_serde]
pub struct PoolStateResponse {
    pub denom: DenomUnit,
    pub total_rewards: Uint128,
    pub total_staked: Uint128,
    pub block_height: Uint64,
}

#[cw_serde]
pub struct ListPoolStatesResponse {
    pub pool_states: Vec<PoolStateResponse>,
}