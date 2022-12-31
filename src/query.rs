use crate::state::{listingz, Bucket, Config, Listing, Status, BUCKETS, CONFIG};
use crate::utils::EzTime;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use cosmwasm_std::{Deps, Env, Order, StdResult};
use cw_storage_plus::PrefixBound;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Queries
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Get contract admin
pub fn get_admin(deps: Deps) -> StdResult<AdminResponse> {
    let storage = CONFIG.load(deps.storage)?;
    Ok(AdminResponse {
        admin: storage.admin.into_string(),
    })
}

// Get config
pub fn get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        config,
    })
}

// Get all buckets owned by an address
pub fn get_buckets(deps: Deps, bucket_owner: &str) -> StdResult<GetBucketsResponse> {
    let bucket_ownerx = deps.api.addr_validate(bucket_owner)?;

    let user_bucks: StdResult<Vec<_>> =
        BUCKETS.prefix(bucket_ownerx).range(deps.storage, None, None, Order::Ascending).collect();

    Ok(GetBucketsResponse {
        buckets: user_bucks?,
    })
}

// Get a single listing by a Listing ID
pub fn get_listing_info(deps: Deps, listing_id: String) -> StdResult<ListingInfoResponse> {
    let Some((_pk, listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id)? else {
        return Err(StdError::GenericErr { msg: "Invalid listing ID".to_string() });
    };

    let status = match listing.status {
        Status::BeingPrepared => "Being Prepared".to_string(),
        Status::FinalizedReady => "Ready for purchase".to_string(),
        Status::Closed => "Closed".to_string(),
    };

    // Getting the sale
    let mut the_sale: Vec<(String, u128)> = vec![];

    listing
        .for_sale
        .native
        .iter()
        .for_each(|the_coin| the_sale.push((the_coin.denom.clone(), the_coin.amount.u128())));

    listing
        .for_sale
        .cw20
        .iter()
        .for_each(|the_coin| the_sale.push((the_coin.address.to_string(), the_coin.amount.u128())));

    listing
        .for_sale
        .nfts
        .iter()
        .for_each(|the_nft| the_sale.push((the_nft.contract_address.to_string(), the_nft.token_id.trim().parse::<u128>().unwrap())));

    // Getting the ask
    let mut the_ask: Vec<(String, u128)> = vec![];

    listing
        .ask
        .native
        .iter()
        .for_each(|the_coin| the_ask.push((the_coin.denom.clone(), the_coin.amount.u128())));

    listing
        .ask
        .cw20
        .iter()
        .for_each(|the_coin| the_ask.push((the_coin.address.to_string(), the_coin.amount.u128())));
    
    listing
        .ask
        .nfts
        .iter()
        .for_each(|the_nft| the_ask.push((the_nft.contract_address.to_string(), the_nft.token_id.trim().parse::<u128>().unwrap())));

    if let Some(x) = listing.expiration_time {
        Ok(ListingInfoResponse {
            creator: listing.creator.to_string(),
            status,
            for_sale: the_sale,
            ask: the_ask,
            expiration: x.eztime_string()?,
        })
    } else {
        Ok(ListingInfoResponse {
            creator: listing.creator.to_string(),
            status,
            for_sale: the_sale,
            ask: the_ask,
            expiration: "None".to_string(),
        })
    }
}

// Get all listings owned by an Address
pub fn get_listings_by_owner(deps: Deps, owner: &str) -> StdResult<MultiListingResponse> {
    let owner = deps.api.addr_validate(owner)?;

    let all_listings: StdResult<Vec<_>> =
        listingz().prefix(&owner).range(deps.storage, None, None, Order::Ascending).collect();

    let listing_data: Vec<Listing> = all_listings?.iter().map(|tup| tup.1.clone()).collect();

    Ok(MultiListingResponse {
        listings: listing_data,
    })
}

// Limited to 100
pub fn get_all_listings(deps: Deps) -> StdResult<MultiListingResponse> {
    let all_listings: StdResult<Vec<_>> =
        listingz().range(deps.storage, None, None, Order::Ascending).take(100).collect();

    let listing_data: Vec<Listing> = all_listings?.iter().map(|entry| entry.1.clone()).collect();

    Ok(MultiListingResponse {
        listings: listing_data,
    })
}

// Query w filter & pagination
pub fn get_listings_for_market(
    deps: Deps,
    env: &Env,
    page_num: u8,
) -> StdResult<MultiListingResponse> {
    let current_time = env.block.time.seconds();
    let two_weeks_ago_in_seconds = current_time - 1_209_600;

    let to_skip_usize = usize::from(page_num * 20 - 20);

    let listings_in_range: Vec<_> = listingz()
        .idx
        .finalized_date
        .prefix_range_raw(
            deps.storage,
            Some(PrefixBound::inclusive(two_weeks_ago_in_seconds)),
            None,
            Order::Ascending,
        )
        .collect::<StdResult<Vec<_>>>()
        .unwrap_or_default()
        .iter()
        .skip(to_skip_usize)
        .take(20)
        .map(|entry| entry.1.clone())
        .collect();

    Ok(MultiListingResponse {
        listings: listings_in_range,
    })
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Responses
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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

#[cw_serde]
pub struct ListingInfoResponse {
    pub creator: String,
    pub status: String,
    pub for_sale: Vec<(String, u128)>,
    pub ask: Vec<(String, u128)>,
    pub expiration: String,
}
