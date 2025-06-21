use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, Timestamp, Uint128};

#[cw_serde]
pub enum Schedule {
    SaturatingLinear,
    PiecewiseLinear(Vec<(u64, Uint128)>),
}

#[cw_serde]
pub enum UncheckedDenom {
    /// A native (bank module) asset.
    Native(String),
    /// A cw20 asset.
    Cw20(String),
}

#[cw_serde]
pub enum CheckedDenom {
    /// A native (bank module) asset.
    Native(String),
    /// A cw20 asset.
    Cw20(Addr),
}

impl CheckedDenom {
    pub fn to_string(&self) -> String {
        match self {
            CheckedDenom::Native(denom) => denom.clone(),
            CheckedDenom::Cw20(addr) => addr.to_string(),
        }
    }
}

#[cw_serde]
pub struct VestingInstantiateMsg {
    pub owner: Option<String>,
    pub recipient: String,
    pub title: String,
    pub description: Option<String>,
    pub total: Uint128,
    pub denom: UncheckedDenom,
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
#[derive(QueryResponses)]
pub enum VestingQueryMsg {
    #[returns(Vest)]
    Info {},
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
pub struct SaturatingLinear {
    pub min_x: u64,
    pub min_y: Uint128,
    pub max_x: u64,
    pub max_y: Uint128,
}

#[cw_serde]
pub struct PiecewiseLinear {
    pub steps: Vec<(u64, Uint128)>,
}

#[cw_serde]
pub enum Curve {
    Constant {
        y: Uint128,
    },
    SaturatingLinear(SaturatingLinear),
    PiecewiseLinear(PiecewiseLinear),
}

#[cw_serde]
pub struct Vest {
    /// vested(t), where t is seconds since start_time.
    vested: Curve,
    start_time: Timestamp,

    pub status: Status,
    pub recipient: Addr,
    pub denom: CheckedDenom,

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
    let vest = deps.querier.query_wasm_smart(
        address, &VestingQueryMsg::Info {}
    )?;
    Ok(vest)
}