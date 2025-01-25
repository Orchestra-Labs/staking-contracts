use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DenomUnit, Uint64};
use cw_storage_plus::Item;

#[cw_serde]
pub struct RewardsDistributionByToken {
    pub denom: DenomUnit,
    pub weight: Uint64,
}

#[cw_serde]
pub struct Config {
    pub staking_orchestrator_addr: Addr,
    pub reward_token: DenomUnit,
    pub rewards_distribution: Vec<RewardsDistributionByToken>,
}

pub const CONFIG: Item<Config> = Item::new("config");