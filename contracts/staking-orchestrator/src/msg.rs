use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{DenomUnit, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}


#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    CreateStakingContract {
        code_id: u64,
        denom_unit: DenomUnit,
        unbonding_period: Option<Duration>,
        owner: Option<String>,
    }
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}