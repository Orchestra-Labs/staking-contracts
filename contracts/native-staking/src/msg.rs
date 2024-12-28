use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{DenomUnit, Uint128};
use cw_controllers::ClaimsResponse;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use cw_utils::Duration;
use symphony_interfaces::staking::StakedBalanceAtHeightResponse;


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
}

#[cw_serde]
pub struct MigrateMsg {}

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