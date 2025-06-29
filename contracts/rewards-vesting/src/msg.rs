use crate::vesting::Schedule;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VestingContractResponse)]
    VestingContract {
        address: String,
    },
    #[returns(GetAllVestingContractsResponse)]
    AllVestingContracts {
        start_after: Option<String>,
        limit: Option<u64>,
    }
}

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

#[cw_serde]
pub struct VestingContractResponse {
    pub found: bool,
    pub address: String,
    pub contract_address: Option<String>
}

#[cw_serde]
pub struct VestingContractRecord {
    pub address: String,
    pub contract_address: String,
}

#[cw_serde]
pub struct GetAllVestingContractsResponse {
    pub contracts: Vec<VestingContractRecord>,
}