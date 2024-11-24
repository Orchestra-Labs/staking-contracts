use cosmwasm_schema::cw_serde;
use cosmwasm_std::DenomUnit;
use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom_unit: DenomUnit,
    pub unbonding_period: Option<Duration>,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}