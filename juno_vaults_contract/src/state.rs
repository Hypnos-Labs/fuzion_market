use cosmwasm_std::{Addr, Coin, Timestamp};
use cw20::{Balance, Cw20CoinVerified};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};

use cosmwasm_schema::cw_serde;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Config
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub const CONFIG: Item<Config> = Item::new("cyberswap_config");

#[cw_serde]
pub struct Config {
    // Admin of contract
    pub admin: Addr,
    // "Osmo", "ibc/4X5Y6Z"
    pub whitelist_native: Vec<(String, String)>,
    // "Shitcoin", "juno1xxx"
    pub whitelist_cw20: Vec<(String, Addr)>,
    // "NeonPeepz", "juno1xxx"
    pub whitelist_nft: Vec<(String, Addr)>,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings IndexedMap
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct ListingIndexes<'a> {
    pub id: UniqueIndex<'a, String, Listing, (&'a Addr, String)>,
    pub finalized_date: MultiIndex<'a, u64, Listing, (&'a Addr, String)>,
    //pub sale_tokens: MultiIndex<'a, Vec<u8>, Listing, (&'a Addr, String)>,
    //pub ask_tokens: MultiIndex<'a, Vec<u8>, Listing, (&'a Addr, String)>,
}

impl IndexList<Listing> for ListingIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Listing>> + '_> {
        let v: Vec<&dyn Index<Listing>> = vec![&self.id, &self.finalized_date]; //, &self.sale_tokens, &self.ask_tokens];
        Box::new(v.into_iter())
    }
}

// Addr is creator, String is the Listing ID aka UniqueIndex
// Note - ListingID is stored as suffix in PK to enable prefix.range on Addr and ensure uniqueness of each PK
pub fn listingz<'a>() -> IndexedMap<'a, (&'a Addr, String), Listing, ListingIndexes<'a>> {
    let indexes = ListingIndexes {
        id: UniqueIndex::new(|a_listing| a_listing.id.clone(), "listing__id"),
        finalized_date: MultiIndex::new(
            |a_listing| match a_listing.finalized_time {
                None => 0 as u64,
                Some(x) => x.seconds() as u64,
            },
            "listings_im",
            "listing__finalized__date",
        ),
        // Unused rn
        //sale_tokens: MultiIndex::new(
        //    |a_listing| {
        //        let mut natives: Vec<_> = a_listing.for_sale.native.iter().map(|native| native.denom.clone()).collect();
        //        let mut cw20s: Vec<_> = a_listing.for_sale.cw20.iter().map(|cw20| cw20.address.clone().into_string()).collect();
        //        if natives.len() > 0 && cw20s.len() > 0 {
        //            natives.append(&mut cw20s);
        //            let strang = natives.join(" ");
        //            return strang.as_bytes().to_vec();
        //        } else if natives.len() > 0 && cw20s.is_empty() {
        //            let strang = natives.join(" ");
        //            return strang.as_bytes().to_vec();
        //        } else {
        //            // Listing will never have 0 in both fields of GenericBalance of for_sale
        //            let strang = cw20s.join(" ");
        //            return strang.as_bytes().to_vec();
        //        };
        //    },
        //    "listings_im",
        //    "listing__sale__tokens",
        //),
        //// Unused rn
        //ask_tokens: MultiIndex::new(
        //    |a_listing| {
        //        let mut natives:Vec<_> = a_listing.ask.native.iter().map(|native| native.denom.clone()).collect();
        //        let mut cw20s: Vec<_> = a_listing.ask.cw20.iter().map(|cw20| cw20.address.clone().into_string()).collect();
        //        if natives.len() > 0 && cw20s.len() > 0 {
        //            natives.append(&mut cw20s);
        //            let strangx = natives.join(" ");
        //            return strangx.as_bytes().to_vec();
        //        } else if natives.len() > 0 && cw20s.is_empty() {
        //            let strangx = natives.join(" ");
        //            return strangx.as_bytes().to_vec();
        //        } else if natives.is_empty() && cw20s.len() > 0 {
        //            let strangx = cw20s.join(" ");
        //            return strangx.as_bytes().to_vec();
        //        } else {
        //            // If both types in ask are empty, return index as Vec[String; 1]
        //            let nonex = "None".to_string();
        //            return nonex.as_bytes().to_vec();
        //        };
        //    },
        //    "listings_im",
        //    "listing__ask__tokens",
        //),
    };

    IndexedMap::new("listings_im", indexes)
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets Map
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Addr = owner, &str = UUID
pub const BUCKETS: Map<(Addr, &str), Bucket> = Map::new("buckets");

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Types
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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

    pub for_sale: GenericBalance,

    pub ask: GenericBalance,
}

#[cw_serde]
pub struct Nft {
    pub contract_address: Addr,
    pub token_id: String,
    // ignore metadata/uri for time being,
    // don't see a scenario where it will be needed
    //pub metadata: Option<Binary>,
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

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Traits - for my types
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub trait GenericBalanceUtil {
    fn add_tokens(&mut self, add: Balance);
    fn add_nft(&mut self, nft: Nft);
}

impl GenericBalanceUtil for GenericBalance {
    // Add a Balance to a GenericBalance
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

pub fn genbal_from_nft(nft: Nft) -> GenericBalance {
    GenericBalance {
        native: vec![],
        cw20: vec![],
        nfts: vec![nft],
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Traits - external types
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub trait ToGenericBalance {
    fn to_generic(&self) -> GenericBalance;
}

impl ToGenericBalance for Balance {
    // Convert a Balance to a GenericBalance
    fn to_generic(&self) -> GenericBalance {
        match self {
            Balance::Native(balance) => GenericBalance {
                native: balance.clone().into_vec(),
                cw20: vec![],
                nfts: vec![],
            },
            Balance::Cw20(token) => GenericBalance {
                native: vec![],
                cw20: vec![token.clone()],
                nfts: vec![],
            },
        }
    }
}
