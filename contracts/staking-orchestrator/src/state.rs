use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DenomUnit};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct RegisteredContract {
    pub address: String,
    pub token: DenomUnit,
}

pub const STAKING_CONTRACTS: Map<&String, RegisteredContract> = Map::new("staking_contracts");

pub const REWARDS_CONTRACT: Item<Addr> = Item::new("rewards_contract");