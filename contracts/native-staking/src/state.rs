use cosmwasm_schema::cw_serde;
use cosmwasm_std::DenomUnit;
use cw_storage_plus::Item;
use cw_utils::Duration;


#[cw_serde]
pub struct Config {
    pub staking_token: DenomUnit,
    pub unstaking_duration: Option<Duration>,
}

pub const CONFIG: Item<Config> = Item::new("config");