use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{DenomUnit, Uint128};
use cw_controllers::ClaimsResponse;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom_unit: DenomUnit,
    pub unbonding_period: Option<Duration>,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(StakedBalanceAtHeightResponse)]
    StakedBalanceAtHeight {
        address: String,
        height: Option<u64>,
    },

    #[returns(TotalStakedAtHeightResponse)]
    TotalStakedAtHeight {
        height: Option<u64>,
    },

    #[returns(ClaimsResponse)]
    Claims { address: String },

    #[returns(ListStakersResponse)]
    ListStakers { start_after: Option<String>, limit: Option<u32> },
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        unbonding_period: Option<Duration>,
    },
    Stake {},
    Unstake { amount: Uint128 },
    Claim {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub staking_token: DenomUnit,
    pub unstaking_duration: Option<Duration>,
}

#[cw_serde]
pub struct TotalStakedAtHeightResponse {
    pub total: Uint128,
    pub height: u64,
}

#[cw_serde]
pub struct ListStakersResponse {
    pub stakers: Vec<StakerBalanceResponse>,
}

#[cw_serde]
pub struct StakedBalanceAtHeightResponse {
    pub balance: Uint128,
    pub height: u64,
}

#[cw_serde]
pub struct StakerBalanceResponse {
    pub address: String,
    pub balance: Uint128,
}