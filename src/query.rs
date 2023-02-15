use crate::query_imports::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Queries
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Current Fee Denom
pub fn get_fee_denom(deps: Deps) -> StdResult<FeeDenomResponse> {
    let fee_denom: FeeDenom = FEE_DENOM.load(deps.storage)?;

    let symbol = match fee_denom {
        FeeDenom::JUNO => "JUNO".to_string(),
        FeeDenom::USDC => "USDC".to_string(),
    };

    Ok(FeeDenomResponse {
        symbol,
        denom: fee_denom.value(),
    })
}

/// Get Counts
pub fn get_counts(deps: Deps) -> StdResult<CountResponse> {
    let listing_count = LISTING_COUNT.load(deps.storage)?;
    let bucket_count = BUCKET_COUNT.load(deps.storage)?;
    Ok(CountResponse {
        listing_count,
        bucket_count,
    })
}

/// Get all buckets owned by an address
pub fn get_buckets(deps: Deps, bucket_owner: &str) -> StdResult<GetBucketsResponse> {
    let bucket_ownerx = deps.api.addr_validate(bucket_owner)?;

    let user_bucks: StdResult<Vec<_>> =
        BUCKETS.prefix(bucket_ownerx).range(deps.storage, None, None, Order::Ascending).collect();

    Ok(GetBucketsResponse {
        buckets: user_bucks?,
    })
}

/// Get all listings owned by an Address
pub fn get_listings_by_owner(deps: Deps, owner: &str) -> StdResult<MultiListingResponse> {
    let owner = deps.api.addr_validate(owner)?;

    let all_listings: StdResult<Vec<_>> =
        listingz().prefix(&owner).range(deps.storage, None, None, Order::Ascending).collect();

    let listing_data: Vec<Listing> = all_listings?.iter().map(|tup| tup.1.clone()).collect();

    Ok(MultiListingResponse {
        listings: listing_data,
    })
}

// XXXXXXXXXXXXXXXXXXXXXXX needs check
/// Finds all listings that `owner` is whitelisted to purchase
pub fn get_users_whitelisted_listings(deps: Deps, owner: &str) -> StdResult<MultiListingResponse> {
    let search_whitelists: Vec<_> = listingz()
        .idx
        .whitelisted_buyer
        .prefix(owner.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>() // StdResult<Vec<(PK, Listing)>>
        .unwrap_or_default()
        .iter()
        .map(|entry| entry.1.clone())
        .collect();

    Ok(MultiListingResponse {
        listings: search_whitelists,
    })
}

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
pub struct CountResponse {
    pub bucket_count: u64,
    pub listing_count: u64,
}

#[cw_serde]
pub struct FeeDenomResponse {
    pub symbol: String,
    pub denom: String,
}

#[cw_serde]
pub struct GetBucketsResponse {
    pub buckets: Vec<(u64, Bucket)>,
}

#[cw_serde]
pub struct MultiListingResponse {
    pub listings: Vec<Listing>,
}
