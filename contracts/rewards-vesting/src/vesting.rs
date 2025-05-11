use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub enum Schedule {
    SaturatingLinear,
    PiecewiseLinear(Vec<(u64, Uint128)>),
}

#[cw_serde]
pub struct VestingInstantiateMsg {
    pub owner: Option<String>,
    pub recipient: String,
    pub title: String,
    pub description: Option<String>,
    pub total: Uint128,
    pub denom: String,
    pub schedule: Schedule,
    pub start_time: Timestamp,
    pub vesting_duration_seconds: u64,
    pub unbonding_duration_seconds: u64,
}