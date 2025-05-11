use cw_storage_plus::Map;

pub const VESTING_CONTRACTS: Map<&String, &String> = Map::new("vesting_contracts");