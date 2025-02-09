use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DenomUnit, Uint128, Uint64};
use cw_storage_plus::{Item, SnapshotMap, Strategy};
use std::collections::HashMap;

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

#[cw_serde]
pub struct PoolState {
    pub denom: DenomUnit,
    pub total_rewards: Uint128,
    pub block_height: Uint64,
}

#[cw_serde]
pub struct RewardsRecord {
    pub rewards: Uint128,
}

#[cw_serde]
pub struct UserState {
    pub reward_debt: Uint128,
    pub last_claim_block_height: Uint64,
    pub rewards_data: HashMap<String, RewardsRecord>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const POOL_STATE: SnapshotMap<&str, PoolState> = SnapshotMap::new(
    "pool_state",
    "pool_state__checkpoints",
    "pool_state__changelog",
    Strategy::EveryBlock,
);
pub const USER_STATE: SnapshotMap<&Addr, UserState> = SnapshotMap::new(
    "user_state",
    "user_state__checkpoints",
    "user_state__changelog",
    Strategy::EveryBlock,
);