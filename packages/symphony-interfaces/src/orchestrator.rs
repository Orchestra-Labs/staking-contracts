use crate::staking::{StakedBalanceAtHeightResponse, StakerBalanceResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::DenomUnit;
use cw_ownable::cw_ownable_query;
use std::collections::HashMap;

#[cw_serde]
pub struct RegisteredContract {
    pub address: String,
    pub token: DenomUnit,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StakingContractByDenomResponse)]
    StakingContractByDenom { denom: String },

    #[returns(AllTokensStakedBalanceAtHeightResponse)]
    AllTokensStakedBalanceAtHeight { address: String, height: Option<u64> },

    #[returns(ListStakersByDenomResponse)]
    ListStakersByDenom { denom: String, start_after: Option<String>, limit: Option<u32> },
}

#[cw_serde]
pub struct StakingContractByDenomResponse {
    pub denom: String,
    pub registered_contract: RegisteredContract,
}

#[cw_serde]
pub struct AllTokensStakedBalanceAtHeightResponse {
    pub tokens_staked_balance: HashMap<String, StakedBalanceAtHeightResponse>,
}

#[cw_serde]
pub struct ListStakersByDenomResponse {
    pub denom: String,
    pub stakers: Vec<StakerBalanceResponse>,
}