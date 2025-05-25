use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Timestamp, Uint128};
use wynd_utils::Curve;

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

#[cw_serde]
pub enum VestingExecuteMsg {
    Cancel {},
}

#[cw_serde]
pub enum Status {
    Unfunded,
    Funded,
    Canceled {
        /// owner_withdrawable(t). This is monotonically decreasing and
        /// will be zero once the owner has completed withdrawing
        /// their funds.
        owner_withdrawable: Uint128,
    },
}

#[cw_serde]
pub struct Vest {
    /// vested(t), where t is seconds since start_time.
    vested: Curve,
    start_time: Timestamp,

    pub status: Status,
    pub recipient: Addr,
    pub denom: String,

    /// The number of tokens that have been claimed by the vest receiver.
    pub claimed: Uint128,
    /// The number of tokens that have been slashed while staked by
    /// the vest receiver. Slashed tokens count against the number of
    /// tokens the receiver is entitled to.
    pub slashed: Uint128,

    pub title: String,
    pub description: Option<String>,
}

pub fn query_vesting_contract_for_info(deps: Deps, address: String) -> Result<Vest, cosmwasm_std::StdError> {
    let vest: Vest = deps.querier.query_wasm_smart(
        address, &{}
    )?;
    Ok(vest)
}