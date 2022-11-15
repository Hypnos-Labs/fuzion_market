use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Order}; // coin, Coin, Uint128
use cw_storage_plus::{PrefixBound};
use std::convert::{TryInto};
use std::marker::PhantomData;
use crate::state::*;
use crate::utils::*;
use cosmwasm_schema::{cw_serde};


/////////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/////////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/////////////// Query Abstractions
/////////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////// Internal
////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Get admin of contract
pub fn get_admin(deps: Deps) -> StdResult<Binary> {
    let storage = CONFIG.load(deps.storage)?;
    to_binary(&AdminResponse {admin: storage.admin.into_string()})
}

// Honestly just get entire config
pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&ConfigResponse {config})
}

// Get all buckets owned by an address
pub fn get_buckets(deps: Deps, bucket_owner: String) -> StdResult<Binary> {

    let bucket_ownerx = deps.api.addr_validate(&bucket_owner)?;

    let user_bucks: StdResult<Vec<_>> = BUCKETS
        .prefix(bucket_ownerx)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    
    let rez = GetBucketsResponse {
        buckets: user_bucks?,
    };

    to_binary(&rez)

}

// Get a single listing by a Listing ID
pub fn get_listing_info(deps: Deps, listing_id: String) -> StdResult<Binary> {

    // If listing doesn't exist the unwrap panics which is
    // handled on front end by manually checking error code
    let (_listing_owner, listing) = listingz().idx.id.item(deps.storage, listing_id.clone())?.unwrap();

    let status = match listing.status {
        Status::BeingPrepared => "Being Prepared".to_string(),
        Status::FinalizedReady => "Ready for purchase".to_string(),
        Status::Closed => "Closed".to_string(),
    };

    // Getting the sale
    let mut the_sale: Vec<(String, u128)> = vec![];

    listing.for_sale.native
    .iter()
    .for_each(|the_coin|
        the_sale.push((the_coin.denom.clone(), the_coin.amount.u128()))
    );

    listing.for_sale.cw20
    .iter()
    .for_each(|the_coin|
        the_sale.push((the_coin.address.to_string(), the_coin.amount.u128()))
    );

    // Getting the ask
    let mut the_ask: Vec<(String, u128)> = vec![];

    listing.ask.native
    .iter()
    .for_each(|the_coin|
        the_ask.push((the_coin.denom.clone(), the_coin.amount.u128()))
    );

    listing.ask.cw20
    .iter()
    .for_each(|the_coin|
        the_ask.push((the_coin.address.to_string(), the_coin.amount.u128()))
    );

    if let Some(x) = listing.expiration_time {
        let res = ListingInfoResponse {
            creator: listing.creator.to_string(),
            status: status,
            for_sale: the_sale,
            ask: the_ask,
            expiration: x.eztime_string()?,
        };
        return to_binary(&res);
    } else {
        let ress = ListingInfoResponse {
            creator: listing.creator.to_string(),
            status: status,
            for_sale: the_sale,
            ask: the_ask,
            expiration: "None".to_string(),
        };
        return to_binary(&ress);
    };

}

// Get all listings owned by an Address
pub fn get_listings_by_owner(deps: Deps, owner: String) -> StdResult<Binary> {

    let owner = deps.api.addr_validate(&owner)?;

    let all_listings: StdResult<Vec<_>> = listingz().prefix(&owner).range(
        deps.storage,
        None,
        None,
        Order::Ascending
    ).collect();

    let listing_data: Vec<Listing> = all_listings?
    .iter()
    .map(|tup| tup.1.clone())
    .collect();
    
    to_binary(&MultiListingResponse {listings: listing_data})
}

// Get most recent 100 Listings that exist
pub fn get_all_listings(deps: Deps) -> StdResult<Binary> {

    let all_listings: StdResult<Vec<_>> = listingz().range( 
        deps.storage, 
        None, 
        None, 
        Order::Ascending
    ).take(100).collect();
    // prob limit this in future to .take(x), shouldn't get that high with removal but ynk
    // to-do:  determine how many to take based on gas usage

    let listing_data: Vec<Listing> = all_listings?
    .iter()
    .map(|entry|
        entry.1.clone()
    )
    .collect();

    to_binary(&MultiListingResponse {listings: listing_data})

}

// Query w filter & pagination
pub fn get_listings_for_market(
    deps: Deps,
    env: Env,
    page_num: u8, 
) -> StdResult<Binary> {

    let current_time = env.block.time.seconds();
    let two_weeks_ago_in_seconds = current_time - 1209600;

    let to_skip = page_num * 20 - 20;

    let listings_in_range: StdResult<Vec<_>> = listingz()
        .idx
        .finalized_date
        .prefix_range(
            deps.storage, 
            Some(PrefixBound::Exclusive((two_weeks_ago_in_seconds, PhantomData))), 
            None, 
            Order::Ascending
        )
        .skip(to_skip.try_into().unwrap())
        .take(20)
        .collect();

    let listing_data: Vec<Listing> = listings_in_range?
    .iter()
    .map(|entry|
        entry.1.clone()
    )
    .collect();

    to_binary(&MultiListingResponse {listings: listing_data})
}

////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////// External
////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////// Query Responses
////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}


#[cw_serde]
pub struct GetBucketsResponse {
    pub buckets: Vec<(String, Bucket)>,

}

#[cw_serde]
pub struct MultiListingResponse {
    pub listings: Vec<Listing>,
}

// Unused / needs update
#[cw_serde]
pub struct ListingInfoResponse {
    pub creator: String,
    pub status: String,
    pub for_sale: Vec<(String, u128)>,
    pub ask: Vec<(String, u128)>,
    pub expiration: String,
}