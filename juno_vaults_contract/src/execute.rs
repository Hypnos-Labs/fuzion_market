use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;
use crate::utils::*;
use cosmwasm_std::{Addr, DepsMut, Env, Response};
use cw20::Balance;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_create_bucket(
    deps: DepsMut,
    funds: Balance,
    creator: &Addr,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Can't create an empty Bucket
    if funds.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    // Check that balance sent in is on whitelist
    let config = CONFIG.load(deps.storage)?;
    is_balance_whitelisted(&funds, &config)?;

    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (creator.clone(), &bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Save bucket
    BUCKETS.save(
        deps.storage,
        (creator.clone(), &bucket_id),
        &Bucket {
            funds: funds.to_generic(),
            owner: creator.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "create_bucket")
        .add_attribute("bucket_id", &bucket_id))
}

pub fn execute_create_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (user_wallet.clone(), &bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // all NFT validation checks are handled in receiver wrapper
    // Save bucket
    BUCKETS.save(
        deps.storage,
        (user_wallet.clone(), &bucket_id),
        &Bucket {
            funds: genbal_from_nft(nft),
            owner: user_wallet.clone(),
        },
    )?;

    Ok(Response::default())
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

    // Ensure bucket exists & Sender is owner
    if let None = BUCKETS.may_load(deps.storage, (sender.clone(), &bucket_id))? {
        return Err(ContractError::NotFound {
            typ: "Bucket".to_string(),
            id: bucket_id,
        });
    }

    // Ensure coins sent in are in whitelist
    let config = CONFIG.load(deps.storage)?;
    is_balance_whitelisted(&funds, &config)?;

    // Get Bucket
    let the_bucket = BUCKETS.load(deps.storage, (sender.clone(), &bucket_id))?;

    // Authorized check
    if sender != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Add tokens to bucket
    // old is current GenericBalance in bucket
    let old = the_bucket.funds.clone();
    // upd is current bucket
    let mut upd = the_bucket;
    // Mutate upd by adding sent in funds to GenericBalance
    upd.funds.add_tokens(funds);
    // Check that upd.funds have changed
    if old == upd.funds {
        // Balance not updated, can have cleaner logic here
        return Err(ContractError::ToDo {});
    }

    // Save the updated bucket
    BUCKETS.save(deps.storage, (sender.clone(), &bucket_id), &upd)?;

    Ok(Response::new()
        .add_attribute("method", "add_funds_to_bucket")
        .add_attribute("listing", &bucket_id))
}

pub fn execute_add_to_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Ensure bucket exists & Sender is owner
    if let None = BUCKETS.may_load(deps.storage, (user_wallet.clone(), &bucket_id))? {
        return Err(ContractError::NotFound {
            typ: "Bucket".to_string(),
            id: bucket_id,
        });
    }

    // Get Bucket
    let the_bucket = BUCKETS.load(deps.storage, (user_wallet.clone(), &bucket_id))?;

    // Authorized check
    if user_wallet != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Create updated bucket
    let new_bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket.clone();
        new_bucket.funds.add_nft(nft);
        if old_funds == new_bucket.funds {
            Err(ContractError::ToDo {})
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

    Ok(Response::new().add_attribute("Add NFT to bucket", format!("Bucket ID: {}", bucket_id)))
}

pub fn execute_withdraw_bucket(
    deps: DepsMut,
    user: &Addr,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Get Bucket
    let the_bucket = BUCKETS.load(deps.storage, (user.clone(), &bucket_id))?;

    // Check sender is owner
    if &the_bucket.owner != user {
        return Err(ContractError::Unauthorized {});
    }

    // Create Send Msgs
    let msgs = send_tokens_cosmos(user, &the_bucket.funds)?;

    // Remove Bucket
    BUCKETS.remove(deps.storage, (user.clone(), &bucket_id));
    // Possibly need to add replyon & verify funds sent successfully before removing bucket

    Ok(Response::new()
        .add_attribute("method", "empty_bucket")
        .add_attribute("bucket_id", &bucket_id)
        .add_messages(msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Combine with execute_create_listing_cw20 for better code quality
pub fn execute_create_listing(
    deps: DepsMut,
    user_address: &Addr,
    funds_sent: Balance,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Check that some tokens were sent with message
    if funds_sent.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    // Only 6 native in whitelist
    if createlistingmsg.ask.native.len() > 6 {
        return Err(ContractError::ToDo {});
    }

    // Only 3 cw20 in whitelist
    if createlistingmsg.ask.cw20.len() > 3 {
        return Err(ContractError::ToDo {});
    }

    // Check that funds sent are whitelisted
    let config = CONFIG.load(deps.storage)?;

    is_balance_whitelisted(&funds_sent, &config)?;

    // Check that funds in ask are whitelisted
    is_genericbalance_whitelisted(&createlistingmsg.ask, &config)?;

    // Check ID isn't taken
    if let Some(_) = listingz()
        .idx
        .id
        .item(deps.storage, createlistingmsg.id.clone())?
    {
        return Err(ContractError::IdAlreadyExists {});
    }

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
            ask: createlistingmsg.ask.clone(),
            claimant: None,
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "create native listing")
        .add_attribute("listing id", &createlistingmsg.id))
}

// Combine with execute_create_listing for better code quality
pub fn execute_create_listing_cw20(
    deps: DepsMut,
    user_address: &Addr,
    _contract_address: &Addr,
    funds_sent: Balance,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Check that some tokens were sent with message
    if funds_sent.is_empty() {
        return Err(ContractError::NoTokens {});
    }

    // Only 3 cw20 in whitelist
    if createlistingmsg.ask.cw20.len() > 3 {
        return Err(ContractError::ToDo {});
    };

    // Check that funds sent are whitelisted
    let config = CONFIG.load(deps.storage)?;
    is_balance_whitelisted(&funds_sent, &config)?;

    // Check that funds in ask are whitelisted
    is_genericbalance_whitelisted(&createlistingmsg.ask, &config)?;

    // Checking to make sure that listing_id isn't taken
    if let Some(_) = listingz()
        .idx
        .id
        .item(deps.storage, createlistingmsg.id.clone())?
    {
        return Err(ContractError::IdAlreadyExists {});
    }

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
            ask: createlistingmsg.ask,
            claimant: None,
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "create cw20 listing")
        .add_attribute("listing id", &createlistingmsg.id)
        .add_attribute("creator", &user_address.to_string()))
}

pub fn execute_create_listing_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Checking to make sure that listing_id isn't taken
    if let Some(_) = listingz()
        .idx
        .id
        .item(deps.storage, createlistingmsg.id.clone())?
    {
        return Err(ContractError::IdAlreadyExists {});
    }

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
            ask: createlistingmsg.ask,
            claimant: None,
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "create cw721 listing")
        .add_attribute("listing id", &createlistingmsg.id)
        .add_attribute("creator", &user_wallet.to_string()))
}

pub fn execute_change_ask(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: String,
    new_ask: GenericBalance,
) -> Result<Response, ContractError> {
    // Ensure listing exists
    if let None = listingz().may_load(deps.storage, (user_sender, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let listing = listingz().load(deps.storage, (user_sender, listing_id.clone()))?;

    // Ensure sender is owner
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no finalized time
    if listing.finalized_time != None {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure being prepared
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no Claimant
    if listing.claimant != None {
        return Err(ContractError::Unauthorized {});
    }

    // Check that new_ask is whitelisted
    let config = CONFIG.load(deps.storage)?;
    is_genericbalance_whitelisted(&new_ask, &config)?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id.clone()),
        Some(&Listing {
            ask: new_ask,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("Change Ask", "Change listing ask")
        .add_attribute("listing ID", &listing_id))
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

    // Ensure listing exists
    if let None = listingz().may_load(deps.storage, (user_sender, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let listing = listingz().load(deps.storage, (user_sender, listing_id.clone()))?;

    // Ensure sender is Creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure status is InPreperation
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no claimant <not already purchased>
    if listing.claimant != None {
        return Err(ContractError::Unauthorized {}); // better error name
    }

    // Ensure coins sent in are in whitelist
    let config = CONFIG.load(deps.storage)?;
    is_balance_whitelisted(&balance, &config)?;

    // Update old listing by adding tokens
    let new_listing = {
        let old_listing = listing.for_sale.clone();
        let mut x = listing.clone();
        x.for_sale.add_tokens(balance);
        if old_listing == x.for_sale {
            Err(ContractError::ToDo {})
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
        .add_attribute("method", "add funds to listing")
        .add_attribute("listing", &listing_id))
}

pub fn execute_add_to_sale_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    listing_id: String,
) -> Result<Response, ContractError> {
    // Ensure listing exists
    if let None = listingz().may_load(deps.storage, (user_wallet, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let old_listing = listingz().load(deps.storage, (user_wallet, listing_id.clone()))?;

    // Ensure sender is Creator
    if user_wallet != &old_listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure status is InPreperation
    if old_listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no claimant <not already purchased>
    if old_listing.claimant != None {
        return Err(ContractError::Unauthorized {}); // better error name
    }

    // Whitelist check handled in cw721 wrapper
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
        (user_wallet, listing_id.clone()),
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
    // Check listing exists
    if let None = listingz().may_load(deps.storage, (user_sender, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let listing = listingz().load(deps.storage, (user_sender, listing_id.clone()))?;

    // Only listing creator can remove listing
    if &listing.creator != user_sender {
        return Err(ContractError::Unauthorized {});
    }

    // Only can be removed if status is BeingPrepared
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Only can be removed if has not yet been purchased
    if listing.claimant != None {
        return Err(ContractError::Unauthorized {});
    }

    // Delete listing & send funds back to user
    let msgs = send_tokens_cosmos(&listing.creator, &listing.for_sale)?;

    listingz().remove(deps.storage, (user_sender, listing_id.clone()))?;

    Ok(Response::new()
        .add_attribute("method", "remove_listing")
        .add_messages(msgs))
}

pub fn execute_finalize(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    listing_id: String,
    seconds: u64,
) -> Result<Response, ContractError> {
    // Ensure listing exists
    if let None = listingz().may_load(deps.storage, (sender, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let listing = listingz().load(deps.storage, (sender, listing_id.clone()))?;

    // Ensure sender is owner
    if sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no finalized time
    if listing.finalized_time != None {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure being prepared
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no Claimant
    if listing.claimant != None {
        return Err(ContractError::Unauthorized {});
    }

    // max expiration is 1209600 seconds <14 days>
    // min expiration is 600 seconds <10 minutes>
    if !(600..=1209600).contains(&seconds) {
        return Err(ContractError::InvalidExpiration {});
    }

    let finalized_at = env.block.time;
    let expiration = env.block.time.plus_seconds(seconds);

    listingz().replace(
        deps.storage,
        (sender, listing_id.clone()),
        Some(&Listing {
            finalized_time: Some(finalized_at),
            expiration_time: Some(expiration),
            status: Status::FinalizedReady,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("method", "finalize")
        .add_attribute("listing ID", &listing_id)
        .add_attribute("expiration time", &expiration.to_string()))
}

pub fn execute_refund(
    deps: DepsMut,
    env: Env,
    user_sender: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    // Check listing exists
    if let None = listingz().may_load(deps.storage, (user_sender, listing_id.clone()))? {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id,
        });
    }

    // Get listing
    let listing = listingz().load(deps.storage, (user_sender, listing_id.clone()))?;

    // Check sender is creator
    if user_sender.clone() != listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // If listing.claimant != None then listing already purchased
    if listing.claimant != None {
        return Err(ContractError::Unauthorized {});
    }

    // Is listing.status == BeingPrepared
    if listing.status == Status::BeingPrepared {
        return Err(ContractError::Unauthorized {});
        // "Please use close function instead", or just process Refund anyway?
    }

    // Check if listing is expired
    match listing.expiration_time {
        None => {
            // Means status = BeingPrepared
            return Err(ContractError::Unauthorized {});
            // "Please use close function instead".. or just process anyway?
        }

        Some(timestamp) => {
            if env.block.time < timestamp {
                return Err(ContractError::NotExpired {
                    x: timestamp.eztime_string()?,
                });
            }
        }
    };

    // Checks pass, send refund & delete listing
    let refundee = listing.creator;
    let funds = listing.for_sale;
    let send_msgs = send_tokens_cosmos(&refundee, &funds)?;

    // Delete Listing
    listingz().remove(deps.storage, (user_sender, listing_id.clone()))?;

    Ok(Response::new()
        .add_attribute("method", "refund")
        .add_messages(send_msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Purchasing
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_buy_listing(
    deps: DepsMut,
    env: Env,
    buyer: &Addr,
    listing_id: String,
    bucket_id: String,
) -> Result<Response, ContractError> {
    // Get bucket (will error if no bucket found)
    let the_bucket = match BUCKETS.load(deps.storage, (buyer.clone(), &bucket_id)) {
        Ok(buck) => buck,
        Err(_) => return Err(ContractError::LoadBucketError {}),
    };

    // Check that listing exists
    if listingz().idx.id.item(deps.storage, listing_id.clone())? == None {
        return Err(ContractError::NoListing {});
    }

    // Unwrap should never hit as previous line checked for None
    let (_pk, the_listing) = listingz()
        .idx
        .id
        .item(deps.storage, listing_id.clone())?
        .unwrap();

    let listing_owner = the_listing.creator;

    // Check that sender is bucket owner (redundant check)
    if buyer != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }
    // Check that bucket contains required purchase price
    if the_bucket.funds != the_listing.ask {
        return Err(ContractError::FundsSentNotFundsAsked {
            which: format!("Bucket ID: {}", bucket_id),
        });
    }
    // Check that listing is ready for purchase
    if the_listing.status != Status::FinalizedReady {
        return Err(ContractError::NotPurchasable {});
    }
    // Check that there's no existing claimant on listing
    if the_listing.claimant != None {
        return Err(ContractError::NotPurchasable {});
    }
    // Check that listing isn't expired
    if let Some(exp) = the_listing.expiration_time {
        if env.block.time > exp {
            return Err(ContractError::Expired {});
        }
    }

    // Delete Old Listing -> Save new listing with listing_buyer in key & creator
    listingz().remove(deps.storage, (&listing_owner, listing_id.clone()))?;
    listingz().save(
        deps.storage,
        (&buyer, listing_id.clone()),
        &Listing {
            creator: buyer.clone(),
            claimant: Some(buyer.clone()),
            status: Status::Closed,
            ..the_listing
        },
    )?;

    // Delete Old Bucket -> Save new Bucket with listing_seller in key & owner
    BUCKETS.remove(deps.storage, (buyer.clone(), &bucket_id));
    BUCKETS.save(
        deps.storage,
        (listing_owner.clone(), &bucket_id),
        &Bucket {
            owner: listing_owner,
            ..the_bucket
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "buy listing")
        .add_attribute("Bucket Used:", &bucket_id)
        .add_attribute("Listing Purchased:", listing_id))
}

pub fn execute_withdraw_purchased(
    deps: DepsMut,
    withdrawer: &Addr,
    listing_id: String,
) -> Result<Response, ContractError> {
    // Get listing
    let (_pk, the_listing) = listingz()
        .idx
        .id
        .item(deps.storage, listing_id.clone())?
        .unwrap();

    // Check and pull out claimant
    let listing_claimr = the_listing.claimant.ok_or(ContractError::Unauthorized {})?;

    // Check that withdrawer is the claimant
    if withdrawer != &listing_claimr {
        return Err(ContractError::Unauthorized {});
    };

    // Check that status is Closed
    if the_listing.status != Status::Closed {
        return Err(ContractError::Unauthorized {});
    };

    // Delete Listing
    listingz().remove(deps.storage, (&listing_claimr, listing_id.clone()))?;

    // Send balance to claimant (use directly from listing.claimant not message sender)
    let msgs = send_tokens_cosmos(&listing_claimr, &the_listing.for_sale)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw purchased")
        .add_attribute("listing_id", listing_id)
        .add_messages(msgs))
}
