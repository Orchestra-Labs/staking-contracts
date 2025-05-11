use crate::vesting::Schedule;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};
use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>
}

#[cw_serde]
pub enum QueryMsg {}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    BatchVesting {
        configuration: VestingConfiguration,
        records: Vec<VestingRecord>,
    },
    BatchCancel {
        addresses: Vec<String>
    }
}

#[cw_serde]
pub struct VestingConfiguration {
    pub schedule: Schedule,
    pub start_time: Timestamp,
    pub duration: u64,
    pub denom: String,
    pub title_prefix: String,
    pub description_prefix: String,
    pub contract_code_id: u64,
}

#[cw_serde]
pub struct VestingRecord {
    pub address: String,
    pub amount: Uint128,
}