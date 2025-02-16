use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use symphony_interfaces::orchestrator::RegisteredContract;

pub const STAKING_CONTRACTS: Map<&String, RegisteredContract> = Map::new("staking_contracts");

pub const REWARDS_CONTRACT: Item<Addr> = Item::new("rewards_contract");