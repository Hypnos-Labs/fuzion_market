use crate::state::{listingz, Bucket, Config, Listing, Status, BUCKETS, CONFIG};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
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

    listing.for_sale.nfts.iter().for_each(|the_nft| {
        the_sale.push((
            the_nft.contract_address.to_string(),
            the_nft.token_id.trim().parse::<u128>().expect("Invalid token ID"),
        ));
    });

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

    listing.ask.nfts.iter().for_each(|the_nft| {
        the_ask.push((
            the_nft.contract_address.to_string(),
            the_nft.token_id.trim().parse::<u128>().expect("Invalid token ID"),
        ));
    });

    let whitelisted_accs: Vec<String> = vec![
        listing.whitelisted_buyer_one,
        listing.whitelisted_buyer_two,
        listing.whitelisted_buyer_three,
    ]
    .into_iter()
    .flatten()
    .map(|x: Addr| x.to_string())
    .collect();

    // unwrap, if there are any, then map each Addr to a String
    // let whitelisted_accs =
    //     listing.whitelisted_purchasers.unwrap_or_default().iter().map(|x| x.to_string()).collect();

    let mut res: ListingInfoResponse = ListingInfoResponse {
        creator: listing.creator.to_string(),
        status,
        for_sale: the_sale,
        ask: the_ask,
        expiration: "None".to_string(),
        whitelisted_purchasers: whitelisted_accs,
    };

    if let Some(x) = listing.expiration_time {
        res.expiration = x.seconds().to_string();
    };

    Ok(res)
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

pub fn get_users_whitelisted_listings(deps: Deps, owner: &str) -> StdResult<MultiListingResponse> {
    let search_whitelist_one: Vec<_> = listingz()
        .idx
        .whitelisted_one
        .prefix(owner.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>() // StdResult<Vec<(PK, Listing)>>
        .unwrap_or_default()
        .iter()
        .map(|entry| entry.1.clone())
        .collect();

    let search_whitelist_two: Vec<_> = listingz()
        .idx
        .whitelisted_two
        .prefix(owner.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .unwrap_or_default()
        .iter()
        .map(|entry| entry.1.clone())
        .collect();

    let search_whitelist_three: Vec<_> = listingz()
        .idx
        .whitelisted_three
        .prefix(owner.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .unwrap_or_default()
        .iter()
        .map(|entry| entry.1.clone())
        .collect();

    let all_listings: Vec<_> = search_whitelist_one
        .into_iter()
        .chain(search_whitelist_two.into_iter())
        .chain(search_whitelist_three.into_iter())
        .collect();

    Ok(MultiListingResponse {
        listings: all_listings,
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
    pub whitelisted_purchasers: Vec<String>,
}
