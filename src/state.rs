use cosmwasm_std::{Addr, Coin, Timestamp};
use cw20::{Balance, Cw20CoinVerified};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};

use cosmwasm_schema::cw_serde;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Config
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub const CONFIG: Item<Config> = Item::new("junovaults_config");

#[cw_serde]
pub struct Config {
    // Admin of contract
    pub admin: Addr,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings IndexedMap
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct ListingIndexes<'a> {
    pub id: UniqueIndex<'a, String, Listing, (&'a Addr, String)>,
    pub finalized_date: MultiIndex<'a, u64, Listing, (&'a Addr, String)>,
}

impl IndexList<Listing> for ListingIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Listing>> + '_> {
        let v: Vec<&dyn Index<Listing>> = vec![&self.id, &self.finalized_date];
        Box::new(v.into_iter())
    }
}

#[must_use]
pub fn listingz<'a>() -> IndexedMap<'a, (&'a Addr, String), Listing, ListingIndexes<'a>> {
    let indexes = ListingIndexes {
        id: UniqueIndex::new(|a_listing| a_listing.id.clone(), "listing__id"),
        finalized_date: MultiIndex::new(
            |_pk, a_listing| a_listing.finalized_time.map_or(0_u64, |x| x.seconds()),
            "listings_im",
            "listing__finalized__date",
        ),
    };

    IndexedMap::new("listings_im", indexes)
}

pub const BUCKETS: Map<(Addr, &str), Bucket> = Map::new("buckets");

#[cw_serde]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
    pub nfts: Vec<Nft>,
}

#[cw_serde]
pub struct Listing {
    pub creator: Addr,
    pub id: String,
    pub finalized_time: Option<Timestamp>,
    pub expiration_time: Option<Timestamp>,
    pub status: Status,
    pub claimant: Option<Addr>,
    pub whitelisted_purchasers: Option<Vec<Addr>>,

    pub for_sale: GenericBalance,

    pub ask: GenericBalance,
}

#[cw_serde]
pub struct Nft {
    pub contract_address: Addr,
    pub token_id: String,
}

#[cw_serde]
pub struct Bucket {
    pub funds: GenericBalance,
    pub owner: Addr,
}

#[cw_serde]
pub enum Status {
    BeingPrepared,
    FinalizedReady,
    Closed,
}

pub trait GenericBalanceUtil {
    fn add_tokens(&mut self, add: Balance);
    fn add_nft(&mut self, nft: Nft);
}

impl GenericBalanceUtil for GenericBalance {
    fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }

    fn add_nft(&mut self, nft: Nft) {
        self.nfts.push(nft);
    }
}

#[must_use]
pub fn genbal_from_nft(nft: Nft) -> GenericBalance {
    GenericBalance {
        native: Vec::new(),
        cw20: Vec::new(),
        nfts: vec![nft],
    }
}

pub trait ToGenericBalance {
    fn to_generic(&self) -> GenericBalance;
}

impl ToGenericBalance for Balance {
    fn to_generic(&self) -> GenericBalance {
        match self {
            Balance::Native(balance) => GenericBalance {
                native: balance.clone().into_vec(),
                cw20: Vec::new(),
                nfts: Vec::new(),
            },
            Balance::Cw20(token) => GenericBalance {
                native: Vec::new(),
                cw20: vec![token.clone()],
                nfts: Vec::new(),
            },
        }
    }
}
