use cosmwasm_std::{Addr};
use cw_storage_plus::Map;
use royalties::RoyaltyInfo;

pub const REGISTRY: Map<&Addr, RoyaltyInfo> = Map::new("royalty_registry");

