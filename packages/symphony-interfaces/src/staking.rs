use cosmwasm_schema::cw_serde;
use cosmwasm_std::DenomUnit;
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom_unit: DenomUnit,
    pub unbonding_period: Option<Duration>,
}