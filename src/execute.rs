use crate::error::ContractError;
use crate::msg::CreateListingMsg;
use crate::state::{
    genbal_from_nft, listingz, Bucket, GenericBalance, GenericBalanceUtil, Listing, Nft, Status,
    ToGenericBalance, BUCKETS,
};
use crate::utils::{calc_fee, normalize_ask_error_on_dup, send_tokens_cosmos};
use cosmwasm_std::{Addr, DepsMut, Env, Response};
use cw20::Balance;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_create_bucket(
    deps: DepsMut,
    funds: &Balance,
    creator: &Addr,
    bucket_id: &String,
) -> Result<Response, ContractError> {
    // Can't create an empty Bucket
    if funds.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (creator.clone(), bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Save bucket
    BUCKETS.save(
        deps.storage,
        (creator.clone(), bucket_id),
        &Bucket {
            funds: funds.to_generic(),
            owner: creator.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_bucket")
        .add_attribute("bucket_id", bucket_id))
}

pub fn execute_create_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: &str,
) -> Result<Response, ContractError> {
    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (user_wallet.clone(), bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // NFT validation checks are handled in receiver wrapper
    // Save bucket
    BUCKETS.save(
        deps.storage,
        (user_wallet.clone(), bucket_id),
        &Bucket {
            funds: genbal_from_nft(nft),
            owner: user_wallet.clone(),
        },
    )?;

    Ok(Response::default())
}

fn get_bucket_if_sender_is_owner(
    deps: &DepsMut,
    sender: &Addr,
    bucket_id: &str,
) -> Result<Bucket, ContractError> {
    let Some(the_bucket) = BUCKETS.may_load(deps.storage, (sender.clone(), bucket_id))? else {
        return Err(ContractError::NotFound { typ: "Bucket".to_string(), id: bucket_id.to_string() })
    };

    // Authorized check
    if sender != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(the_bucket)
}

pub fn execute_add_to_bucket(
    deps: DepsMut,
    funds: Balance,
    sender: &Addr,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Error if no funds sent
    if funds.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    let the_bucket = get_bucket_if_sender_is_owner(&deps, sender, &bucket_id)?;

    // Add tokens
    let new_bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_tokens(funds);
        if old_funds == new_bucket.funds {
            Err(ContractError::ErrorAdding("Tokens to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Save the updated bucket
    BUCKETS.save(deps.storage, (sender.clone(), &bucket_id), &new_bucket)?;

    Ok(Response::new()
        .add_attribute("action", "add_funds_to_bucket")
        .add_attribute("bucket_id", &bucket_id))
}

pub fn execute_add_to_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: String,
) -> Result<Response, ContractError> {
    let the_bucket = get_bucket_if_sender_is_owner(&deps, user_wallet, &bucket_id)?;

    // Create updated bucket
    let new_bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_nft(nft);
        if old_funds == new_bucket.funds {
            Err(ContractError::ErrorAdding("NFT to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Save updated bucket
    BUCKETS.update(deps.storage, (user_wallet.clone(), &bucket_id), {
        |o| match o {
            Some(_) => Ok(new_bucket),
            None => Err(ContractError::ToDo {}),
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "execute_add_to_bucket_cw721")
        .add_attribute("bucket_id", bucket_id))
}

pub fn execute_withdraw_bucket(
    deps: DepsMut,
    user_wallet: &Addr,
    bucket_id: &str,
) -> Result<Response, ContractError> {
    let the_bucket = get_bucket_if_sender_is_owner(&deps, user_wallet, bucket_id)?;

    // Create Send Msgs
    let msgs = send_tokens_cosmos(user_wallet, &the_bucket.funds)?;

    // Remove Bucket
    BUCKETS.remove(deps.storage, (user_wallet.clone(), bucket_id));

    Ok(Response::new()
        .add_attribute("action", "empty_bucket")
        .add_attribute("bucket_id", bucket_id)
        .add_messages(msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

fn validate_basic_new_listing(
    deps: &DepsMut,
    listing_id: &str,
    bal: GenericBalance,
    whitelisted_buyer: Option<String>,
) -> Result<(GenericBalance, Option<Addr>), ContractError> {
    // Check ID isn't taken
    if (listingz().idx.id.item(deps.storage, listing_id.to_string())?).is_some() {
        return Err(ContractError::IdAlreadyExists {});
    }

    // normalize the tokens sent in
    let ask_tokens = normalize_ask_error_on_dup(bal)?;

    let whitelist: Option<Addr> = whitelisted_buyer.map(|w| deps.api.addr_validate(&w)).transpose()?;
    Ok((ask_tokens, whitelist))
}

pub fn execute_create_listing(
    deps: DepsMut,
    user_address: &Addr,
    funds_sent: &Balance,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Check that some tokens were sent with message
    if funds_sent.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    let (ask_tokens, whitelisted_buyer) = validate_basic_new_listing(
        &deps,
        &createlistingmsg.id,
        createlistingmsg.ask,
        createlistingmsg.whitelisted_buyer,
    )?;

    // Save listing
    listingz().save(
        deps.storage,
        (user_address, createlistingmsg.id.clone()),
        &Listing {
            creator: user_address.clone(),
            id: createlistingmsg.id.clone(),
            finalized_time: None,
            expiration_time: None,
            status: Status::BeingPrepared,
            for_sale: funds_sent.to_generic(),
            ask: ask_tokens,
            claimant: None,
            whitelisted_buyer,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_native_listing")
        .add_attribute("listing_id", &createlistingmsg.id))
}

pub fn execute_create_listing_cw20(
    deps: DepsMut,
    user_address: &Addr,
    _contract_address: &Addr,
    funds_sent: &Balance,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Check that some tokens were sent with message
    if funds_sent.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    let (ask_tokens, whitelisted_buyer) = validate_basic_new_listing(
        &deps,
        &createlistingmsg.id,
        createlistingmsg.ask,
        createlistingmsg.whitelisted_buyer,
    )?;

    listingz().save(
        deps.storage,
        (user_address, createlistingmsg.id.clone()),
        &Listing {
            creator: user_address.clone(),
            id: createlistingmsg.id.clone(),
            finalized_time: None,
            expiration_time: None,
            status: Status::BeingPrepared,
            for_sale: funds_sent.to_generic(),
            ask: ask_tokens,
            claimant: None,
            whitelisted_buyer,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_cw20_listing")
        .add_attribute("listing_id", &createlistingmsg.id)
        .add_attribute("creator", user_address.to_string()))
}

pub fn execute_create_listing_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    let (ask_tokens, whitelisted_buyer) = validate_basic_new_listing(
        &deps,
        &createlistingmsg.id,
        createlistingmsg.ask,
        createlistingmsg.whitelisted_buyer,
    )?;

    listingz().save(
        deps.storage,
        (user_wallet, createlistingmsg.id.clone()),
        &Listing {
            creator: user_wallet.clone(),
            id: createlistingmsg.id.clone(),
            finalized_time: None,
            expiration_time: None,
            status: Status::BeingPrepared,
            for_sale: genbal_from_nft(nft),
            ask: ask_tokens,
            claimant: None,
            whitelisted_buyer,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_cw721_listing")
        .add_attribute("listing_id", &createlistingmsg.id)
        .add_attribute("creator", user_wallet.to_string()))
}

/// Validate basic listing info
/// - Ensure listing exists, sender is owner, & get listing
/// - Ensure sender is owner
/// - Ensure no finalized time
/// - Ensure being prepared
/// If all of these checks pass, return the listing.
fn validate_basic_listings(
    deps: &DepsMut,
    user_sender: &Addr,
    listing_id: &str,
    is_refund: bool, // only for execute_refund
) -> Result<Listing, ContractError> {
    // Ensure listing exists, sender is owner, & get listing
    let Some(listing) = listingz().may_load(deps.storage, (user_sender, listing_id.to_string()))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string()
        });
    };

    // Ensure sender is owner
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no finalized time (unless we want to refund, in which case ignore)
    if !is_refund && listing.finalized_time.is_some() {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure being prepared (unless it is a refund)
    if !is_refund && listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no Claimant
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    Ok(listing)
}

pub fn execute_change_ask(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: String,
    new_ask: GenericBalance,
) -> Result<Response, ContractError> {
    let listing = validate_basic_listings(&deps, user_sender, &listing_id, false)?;

    let new_ask_tokens = normalize_ask_error_on_dup(new_ask)?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id.clone()),
        Some(&Listing {
            ask: new_ask_tokens,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("attribute", "change_listing_ask")
        .add_attribute("listing_id", &listing_id))
}

pub fn execute_modify_whitelisted_buyer(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: String,
    new_whitelisted_buyer: Option<String>,
) -> Result<Response, ContractError> {
    let listing = validate_basic_listings(&deps, user_sender, &listing_id, false)?;

    let whitelisted_buyer: Option<Addr> = if let Some(new_whitelisted_buyer) = new_whitelisted_buyer
    {
        let addr = deps.api.addr_validate(&new_whitelisted_buyer)?;
        Some(addr)
    } else {
        None
    };

    listingz().replace(
        deps.storage,
        (user_sender, listing_id.clone()),
        Some(&Listing {
            whitelisted_buyer,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("attribute", "execute_modify_whitelisted_buyer")
        .add_attribute("listing_id", &listing_id))
}

pub fn execute_add_funds_to_sale(
    deps: DepsMut,
    balance: Balance,
    user_sender: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    // Error if no funds sent
    if balance.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    let listing = validate_basic_listings(&deps, user_sender, &listing_id, false)?;

    // Update old listing by adding tokens
    let new_listing = {
        let old_listing = listing.for_sale.clone();
        let mut x = listing.clone();
        x.for_sale.add_tokens(balance);
        if old_listing == x.for_sale {
            Err(ContractError::ErrorAdding("Tokens to Listing".to_string()))
        } else {
            Ok(x)
        }
    }?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id.clone()),
        Some(&new_listing),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("action", "add_funds_to_listing")
        .add_attribute("listing", &listing_id))
}

pub fn execute_add_to_sale_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    listing_id: String,
) -> Result<Response, ContractError> {
    let old_listing = validate_basic_listings(&deps, user_wallet, &listing_id, false)?;

    // Create updated listing
    let new_listing = {
        let old = old_listing.for_sale.clone();
        let mut x = old_listing.clone();
        x.for_sale.add_nft(nft);
        if old == x.for_sale {
            Err(ContractError::ToDo {})
        } else {
            Ok(x)
        }
    }?;

    // Replace old listing with new listing
    listingz().replace(
        deps.storage,
        (user_wallet, listing_id),
        Some(&new_listing),
        Some(&old_listing),
    )?;

    Ok(Response::default())
}

pub fn execute_remove_listing(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    let listing = validate_basic_listings(&deps, user_sender, &listing_id, false)?;

    // Delete listing & send funds back to user
    let msgs = send_tokens_cosmos(&listing.creator, &listing.for_sale)?;

    listingz().remove(deps.storage, (user_sender, listing_id))?;

    Ok(Response::new().add_attribute("action", "remove_listing").add_messages(msgs))
}

pub fn execute_finalize(
    deps: DepsMut,
    env: &Env,
    user_sender: &Addr,
    listing_id: String,
    seconds: u64,
) -> Result<Response, ContractError> {
    let listing = validate_basic_listings(&deps, user_sender, &listing_id, false)?;

    // max expiration is 1209600 seconds <14 days>
    // min expiration is 600 seconds <10 minutes>
    if !(600..=1_209_600).contains(&seconds) {
        return Err(ContractError::InvalidExpiration {});
    }

    let finalized_at = env.block.time;
    let expiration = env.block.time.plus_seconds(seconds);

    listingz().replace(
        deps.storage,
        (user_sender, listing_id.clone()),
        Some(&Listing {
            finalized_time: Some(finalized_at),
            expiration_time: Some(expiration),
            status: Status::FinalizedReady,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("action", "finalize")
        .add_attribute("listing_id", &listing_id)
        .add_attribute("expiration_seconds", expiration.to_string()))
}

pub fn execute_refund(
    deps: DepsMut,
    env: &Env,
    user_sender: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    let listing = validate_basic_listings(&deps, user_sender, &listing_id, true)?;

    // Check if listing is expired
    match listing.expiration_time {
        None => {
            return Err(ContractError::Unauthorized {});
        }
        Some(timestamp) => {
            if env.block.time < timestamp {
                return Err(ContractError::NotExpired {
                    x: timestamp.seconds().to_string(),
                });
            }
        }
    };

    // Checks pass, send refund & delete listing
    let refundee = listing.creator;
    let funds = listing.for_sale;
    let send_msgs = send_tokens_cosmos(&refundee, &funds)?;

    // Delete Listing
    listingz().remove(deps.storage, (user_sender, listing_id))?;

    Ok(Response::new().add_attribute("action", "refund").add_messages(send_msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Purchasing
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_buy_listing(
    deps: DepsMut,
    env: &Env,
    buyer: &Addr,
    listing_id: String,
    bucket_id: &str,
) -> Result<Response, ContractError> {
    // Get bucket (will error if no bucket found)
    let the_bucket = match BUCKETS.load(deps.storage, (buyer.clone(), bucket_id)) {
        Ok(buck) => buck,
        Err(_) => return Err(ContractError::LoadBucketError {}),
    };

    // Check listing exists & get the_listing
    let Some((_pk, the_listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id.clone())? else {
        return Err(ContractError::NotFound { typ: "Listing".to_string(), id: listing_id });
    };

    // Check that sender is bucket owner (redundant check)
    if buyer != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }
    // Check that bucket contains required purchase price
    if the_bucket.funds != the_listing.ask {
        return Err(ContractError::FundsSentNotFundsAsked {
            which: format!("Bucket ID: {bucket_id}"),
        });
    }
    // Check that listing is ready for purchase
    if the_listing.status != Status::FinalizedReady {
        return Err(ContractError::NotPurchasable {});
    }
    // Check that the user buying is whitelisted
    if let Some(whitelist) = the_listing.whitelisted_buyer.clone() {
        if whitelist != *buyer {
            return Err(ContractError::NotWhitelisted {});
        }
    }

    // Check that there's no existing claimant on listing
    if the_listing.claimant.is_some() {
        return Err(ContractError::NotPurchasable {});
    }
    // Check that listing isn't expired
    if let Some(exp) = the_listing.expiration_time {
        if env.block.time > exp {
            return Err(ContractError::Expired {});
        }
    }

    // Delete Old Listing -> Save new listing with listing_buyer in key & creator
    listingz().remove(deps.storage, (&the_listing.creator, listing_id.clone()))?;
    listingz().save(
        deps.storage,
        (buyer, listing_id.clone()),
        &Listing {
            creator: buyer.clone(),
            claimant: Some(buyer.clone()),
            status: Status::Closed,
            ..the_listing
        },
    )?;

    // Delete Old Bucket -> Save new Bucket with listing_seller in key & owner
    BUCKETS.remove(deps.storage, (buyer.clone(), bucket_id));
    BUCKETS.save(
        deps.storage,
        (the_listing.creator.clone(), bucket_id),
        &Bucket {
            owner: the_listing.creator,
            ..the_bucket
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "buy_listing")
        .add_attribute("bucket_used", bucket_id)
        .add_attribute("listing_purchased:", &listing_id))
}

// TODO: merge this in with buy_listing function above
pub fn execute_withdraw_purchased(
    deps: DepsMut,
    withdrawer: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    // Get listing
    let Some((_pk, the_listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id.clone())? else {
        return Err(ContractError::NotFound { typ: "Listing".to_string(), id: listing_id });
    };

    // Check and pull out claimant
    let listing_claimer = the_listing.claimant.ok_or(ContractError::Unauthorized {})?;

    // Check that withdrawer is the claimant
    if withdrawer != &listing_claimer {
        return Err(ContractError::Unauthorized {});
    };

    // Check that status is Closed
    if the_listing.status != Status::Closed {
        return Err(ContractError::Unauthorized {});
    };

    // Delete Listing
    listingz().remove(deps.storage, (&listing_claimer, listing_id.clone()))?;

    // default listing response
    let res: Response = Response::new()
        .add_attribute("action", "withdraw_purchased")
        .add_attribute("listing_id", listing_id);

    if let Some((fee_msg, gbal)) =
        calc_fee(&the_listing.for_sale).map_err(|_foo| ContractError::FeeCalc)?
    {
        let user_msgs = send_tokens_cosmos(&listing_claimer, &gbal)?;
        Ok(res.add_message(fee_msg).add_messages(user_msgs))
    } else {
        let user_msgs = send_tokens_cosmos(&listing_claimer, &the_listing.for_sale)?;
        Ok(res.add_messages(user_msgs))
    }
}
