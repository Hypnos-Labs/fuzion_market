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

    // Key = (whitelisted buyer, listing_id)
    pub whitelisted_one: UniqueIndex<'a, (String, String), Listing, (&'a Addr, String)>,
    pub whitelisted_two: UniqueIndex<'a, (String, String), Listing, (&'a Addr, String)>,
    pub whitelisted_three: UniqueIndex<'a, (String, String), Listing, (&'a Addr, String)>,
}

impl IndexList<Listing> for ListingIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Listing>> + '_> {
        let v: Vec<&dyn Index<Listing>> = vec![&self.id, &self.finalized_date, &self.whitelisted_one, &self.whitelisted_two, &self.whitelisted_three];
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
        whitelisted_one: UniqueIndex::new(
            |listing| {
                (
                    listing.whitelisted_buyer_one
                        .clone()
                        .map_or_else(|| "1".to_string(), |addr| addr.to_string()),
                    listing.id.clone()
                )
            },
            "listing__whitelisted__one"
        ),
        whitelisted_two: UniqueIndex::new(
            |listing| {
                (
                    listing.whitelisted_buyer_two
                        .clone()
                        .map_or_else(|| "2".to_string(), |addr| addr.to_string()),
                    listing.id.clone()
                )
            },
            "listing__whitelisted__two"
        ),
        whitelisted_three: UniqueIndex::new(
            |listing| {
                (
                    listing.whitelisted_buyer_three
                        .clone()
                        .map_or_else(|| "3".to_string(), |addr| addr.to_string()),
                    listing.id.clone()
                )
            },
            "listing__whitelisted__three"
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
    //pub whitelisted_purchasers: Option<Vec<Addr>>,
    pub whitelisted_buyer_one: Option<Addr>,
    pub whitelisted_buyer_two: Option<Addr>,
    pub whitelisted_buyer_three: Option<Addr>,

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
        native: vec![],
        cw20: vec![],
        nfts: vec![nft],
    }
}

pub trait ToGenericBalance {
    fn to_generic(&self) -> GenericBalance;
}

impl ToGenericBalance for Balance {
    fn to_generic(&self) -> GenericBalance {
        match self {
            Self::Native(balance) => GenericBalance {
                native: balance.clone().into_vec(),
                cw20: vec![],
                nfts: vec![],
            },
            Self::Cw20(token) => GenericBalance {
                native: vec![],
                cw20: vec![token.clone()],
                nfts: vec![],
            },
        }
    }
}
