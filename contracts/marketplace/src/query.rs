use crate::{query_imports::*, state::ROYALTY_REGISTRY};

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Queries
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Get the current Fee Denom
pub fn get_fee_denom(deps: Deps) -> StdResult<FeeDenomResponse> {
    let fee_denom: FeeDenom = FEE_DENOM.load(deps.storage)?;

    let (name, next_change) = match fee_denom {
        FeeDenom::JUNO(x) => ("JUNO".to_string(), x),
        FeeDenom::USDC(y) => ("USDC".to_string(), y),
    };

    Ok(FeeDenomResponse {
        name,
        denom: fee_denom.value(),
        next_change,
    })
}

/// Get all buckets owned by an address
/// - Requires pagination to avoid exceeding gas limits
/// - `Page 1: first 20` `Page 2: second 20`...
pub fn get_buckets(deps: Deps, bucket_owner: &str, page_num: u8) -> StdResult<MultiBucketResponse> {
    let valid_owner = deps.api.addr_validate(bucket_owner)?;

    let to_skip_usize = usize::from(page_num * 20 - 20);

    let user_buckets: Vec<_> = BUCKETS
        .prefix(valid_owner)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .skip(to_skip_usize)
        .take(20)
        .collect();

    Ok(MultiBucketResponse {
        buckets: user_buckets,
    })
}

/// Get all listings owned by an Address
/// - Requires pagination to avoid exceeding gas limits
/// - `Page 1: first 20` `Page 2: second 20`...
pub fn get_listings_by_owner(
    deps: Deps,
    owner: &str,
    page_num: u8,
) -> StdResult<MultiListingResponse> {
    let valid_owner = deps.api.addr_validate(owner)?;

    let to_skip_usize = usize::from(page_num * 20 - 20);

    let listing_data: Vec<_> = listingz()
        .prefix(&valid_owner)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .skip(to_skip_usize)
        .take(20)
        .map(|entry| entry.1)
        .collect();

    Ok(MultiListingResponse {
        listings: listing_data,
    })
}

/// Get all listings that `owner` is whitelisted to purchase
/// - Only returns listings that are finalized
/// - Only returns listings that are not expired
/// - Only returns listings that are not closed (sold)
pub fn get_whitelisted(deps: Deps, _env: Env, owner: String) -> StdResult<MultiListingResponse> {
    let valid_owner = deps.api.addr_validate(owner.as_str())?;

    let whitelists = listingz()
        .idx
        .whitelisted_buyer
        .prefix(valid_owner.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .filter_map(|entry| {
            let listing: Listing = entry.1.clone();
            if listing.status != Status::Open {
                None
            } else {
                Some(listing)
            }
        })
        .collect::<Vec<Listing>>();

    Ok(MultiListingResponse {
        listings: whitelists,
    })
}

/// - Does not return listings finalized over 2 weeks prior
/// - Does not return listings that have not been finalized
/// - Does not return listings that are expired
/// - Does not return listings that are Closed (sold)
/// 
//

// Only returns listings that are Status::Open
// Supports pagination
pub fn get_listings_for_market(
    deps: Deps,
    _env: Env,
    page_num: u16,
) -> StdResult<MultiListingResponse> {

    let to_skip = usize::from(page_num * 20 - 20);

    let listings = listingz()
        .idx
        .id
        .range_raw(
            deps.storage,
            None,
            None,
            Order::Ascending
        )
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .skip(to_skip)
        .take(20)
        .filter_map(|entry| {
            let listing: Listing = entry.1.clone();
            if listing.status != Status::Open {
                None
            } else {
                Some(listing)
            }
        })
        .collect::<Vec<Listing>>();

    Ok(MultiListingResponse {
        listings
    })
}

/// Unimplemented
/// Gets a single listing by id
pub fn get_single_listing(deps: Deps, listing_id: u64) -> StdResult<SingleListingResponse> {
    let Some((_pk, listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id)? else {
        return Err(StdError::GenericErr { msg: "Invalid listing ID".to_string() });
    };

    Ok(SingleListingResponse {
        listing,
    })
}


pub fn get_royalty_contract(deps: Deps) -> StdResult<Option<Addr>> {
    let z: Option<Addr> = ROYALTY_REGISTRY.load(deps.storage)?;
    Ok(z)
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Responses
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cw_serde]
pub struct FeeDenomResponse {
    pub name: String,
    pub denom: String,
    /// When FeeCycle can be called next
    pub next_change: u64,
}

#[cw_serde]
pub struct SingleListingResponse {
    pub listing: Listing,
}

#[cw_serde]
pub struct MultiListingResponse {
    pub listings: Vec<Listing>,
}

#[cw_serde]
pub struct MultiBucketResponse {
    pub buckets: Vec<(u64, Bucket)>,
}
