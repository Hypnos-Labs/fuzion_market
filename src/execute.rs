use crate::execute_imports::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_create_bucket(
    deps: DepsMut,
    funds: &Balance,
    creator: &Addr,
) -> Result<Response, ContractError> {
    let count = BUCKET_COUNT.load(deps.storage)?;
    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (creator.clone(), count)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Error if funds contains duplicates or 0 balances
    // - Prefer this over normalize to abort rather than alter balance sent
    funds.normalized_check()?;

    // Save bucket
    BUCKETS.save(
        deps.storage,
        (creator.clone(), count),
        &Bucket {
            owner: creator.clone(),
            funds: GenericBalance::from_balance(funds),
            fee_amount: None,
        },
    )?;

    // Update count
    BUCKET_COUNT
        .update(deps.storage, |old| -> Result<u64, StdError> {
            Ok(old.checked_add(1).unwrap_or(1))
        })
        .map_err(|_| ContractError::GenericError("Error updating Bucket Count".to_string()))?;

    Ok(Response::new()
        .add_attribute("action", "create_bucket")
        .add_attribute("bucket_id", count.to_string()))
}

pub fn execute_create_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
) -> Result<Response, ContractError> {
    let count = BUCKET_COUNT.load(deps.storage)?;

    // Check that bucket_id isn't used
    if BUCKETS.has(deps.storage, (user_wallet.clone(), count)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // NFT validation checks are handled in receiver wrapper
    // Save bucket
    BUCKETS.save(
        deps.storage,
        (user_wallet.clone(), count),
        &Bucket {
            owner: user_wallet.clone(),
            funds: GenericBalance::from_nft(nft),
            fee_amount: None,
        },
    )?;

    // Update count
    BUCKET_COUNT
        .update(deps.storage, |old| -> Result<u64, StdError> {
            Ok(old.checked_add(1).unwrap_or(1))
        })
        .map_err(|_| ContractError::GenericError("Error updating Bucket Count".to_string()))?;

    Ok(Response::new()
        .add_attribute("action", "create_bucket")
        .add_attribute("bucket_id", count.to_string()))
}

pub fn execute_add_to_bucket(
    deps: DepsMut,
    funds: Balance,
    sender: &Addr,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Error if funds contains duplicates or 0 balances
    // - Prefer this over normalize to abort rather than alter balance sent
    funds.normalized_check()?;

    // Ensure bucket exists & Sender is owner
    let Some(the_bucket) = BUCKETS.may_load(deps.storage, (sender.clone(), bucket_id))? else {
        return Err(ContractError::NotFound { typ: "Bucket".to_string(), id: bucket_id.to_string() })
    };

    // Authorized check
    if sender != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Add tokens
    let new_bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_tokens(funds);
        if genbal_cmp(&old_funds, &new_bucket.funds).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Save the updated bucket
    //BUCKETS.save(deps.storage, (sender.clone(), &bucket_id), &new_bucket)?;
    BUCKETS.update(deps.storage, (sender.clone(), bucket_id), {
        |o| match o {
            Some(_) => Ok(new_bucket),
            None => Err(ContractError::GenericError("Error during storage update".to_string())),
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "add_funds_to_bucket")
        .add_attribute("bucket_id", bucket_id.to_string()))
}

pub fn execute_add_to_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Ensure bucket exists & Sender is owner
    let Some(the_bucket) = BUCKETS.may_load(deps.storage, (user_wallet.clone(), bucket_id))? else {
        return Err(ContractError::NotFound { typ: "Bucket".to_string(), id: bucket_id.to_string() })
    };

    // Authorized check
    if user_wallet != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Create updated bucket
    let new_bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_nft(nft);
        if genbal_cmp(&old_funds, &new_bucket.funds).is_ok() {
            Err(ContractError::ErrorAdding("NFT to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Save updated bucket
    BUCKETS.update(deps.storage, (user_wallet.clone(), bucket_id), {
        |o| match o {
            Some(_) => Ok(new_bucket),
            None => Err(ContractError::GenericError("Error during storage update".to_string())),
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "execute_add_to_bucket_cw721")
        .add_attribute("bucket_id", bucket_id.to_string()))
}

pub fn execute_withdraw_bucket(
    deps: DepsMut,
    user: &Addr,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Get Bucket
    let the_bucket: Bucket = BUCKETS.load(deps.storage, (user.clone(), bucket_id))?;

    // Check sender is owner redundant
    if &the_bucket.owner != user {
        return Err(ContractError::Unauthorized {});
    }

    // Create Send Msgs
    // (fee_amount is added when Bucket is used to buy a Listing)
    let msgs = the_bucket.withdraw_msgs()?;

    // Remove Bucket
    BUCKETS.remove(deps.storage, (user.clone(), bucket_id));

    Ok(Response::new()
        .add_attribute("action", "empty_bucket")
        .add_attribute("bucket_id", bucket_id.to_string())
        .add_messages(msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub fn execute_create_listing(
    deps: DepsMut,
    user_address: &Addr,
    funds_sent: &Balance,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Error if funds_sent contains duplicates or 0 balances
    // - Prefer this over normalize to abort rather than alter balance sent
    funds_sent.normalized_check()?;

    // Pull incrementor ID
    let count = LISTING_COUNT.load(deps.storage)?;

    // Check edge case that incrementor ID is taken
    if (listingz().idx.id.item(deps.storage, count)?).is_some() {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Get whitelisted buyer | Errors if invalid address
    // let wl_buyer = createlistingmsg
    //     .whitelisted_buyer
    //     .and_then(|address| Some(deps.api.addr_validate(&address)))
    //     .transpose()
    //     .map_err(|_| ContractError::GenericError("Invalid whitelisted buyer".to_string()))?;

    let wl_buyer = createlistingmsg
        .whitelisted_buyer
        .map(|address| deps.api.addr_validate(&address))
        .transpose()
        .map_err(|_e| ContractError::GenericError("Invalid whitelisted buyer".to_string()))?;

    // Error if whitelisted buyer is listing creator?
    // Is there ever a situation where someone might want to do this?
    // if let Some(wlbuyer) = &wl_buyer {
    //     if wlbuyer.eq(user_address) {
    //         return Err(ContractError::GenericError("Whitelisted buyer should not be the same as Listing Creator".to_string()));
    //     }
    // }

    // Check the asking price, errors if invalid
    //check_valid_genbal(&createlistingmsg.ask)?;
    createlistingmsg.ask.check_valid()?;

    // Save listing
    listingz().save(
        deps.storage,
        (user_address, count),
        &Listing {
            creator: user_address.clone(),
            id: count,
            finalized_time: None,
            expiration_time: None,
            status: Status::BeingPrepared,
            claimant: None,
            whitelisted_buyer: wl_buyer,
            for_sale: GenericBalance::from_balance(funds_sent),
            ask: createlistingmsg.ask,
            fee_amount: None,
        },
    )?;

    // Update count
    // Edge case:
    // If (18,446,744,073,709,551,615 - 2) Listings are created before a Listing has been removed,
    // The following Listing Creation call will fail
    // For this to occur, assuming 7 bil human on earth, every human would need to create 2,635,249,153 Listings
    // Assuming 0.001 JUNO gas per tx, that would generate 18,446,744,073,709,551 JUNO in network fees
    // Should handle this case anyway because safety
    LISTING_COUNT
        .update(deps.storage, |old| -> Result<u64, StdError> {
            Ok(old.checked_add(1).unwrap_or(1))
        })
        .map_err(|_| ContractError::GenericError("Error updating Listing Count".to_string()))?;

    Ok(Response::new()
        .add_attribute("action", "create_listing")
        .add_attribute("listing_id", count.to_string())
        .add_attribute("creator", user_address.to_string()))
}

pub fn execute_create_listing_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    createlistingmsg: CreateListingMsg,
) -> Result<Response, ContractError> {
    // Pull ID incrementor
    let count = LISTING_COUNT.load(deps.storage)?;

    // Edge case check that ID isn't taken
    if (listingz().idx.id.item(deps.storage, count)?).is_some() {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Get whitelisted buyer | Errors if invalid address
    let wl_buyer = createlistingmsg
        .whitelisted_buyer
        .map(|address| deps.api.addr_validate(&address))
        .transpose()
        .map_err(|_| ContractError::GenericError("Invalid whitelisted buyer".to_string()))?;

    // Check the asking price, errors if invalid
    createlistingmsg.ask.check_valid()?;

    listingz().save(
        deps.storage,
        (user_wallet, count),
        &Listing {
            creator: user_wallet.clone(),
            id: count,
            finalized_time: None,
            expiration_time: None,
            status: Status::BeingPrepared,
            claimant: None,
            whitelisted_buyer: wl_buyer,
            for_sale: GenericBalance::from_nft(nft),
            ask: createlistingmsg.ask,
            fee_amount: None,
        },
    )?;

    LISTING_COUNT
        .update(deps.storage, |old| -> Result<u64, StdError> {
            Ok(old.checked_add(1).unwrap_or(1))
        })
        .map_err(|_| ContractError::GenericError("Error updating Listing Count".to_string()))?;

    Ok(Response::new()
        .add_attribute("action", "create_cw721_listing")
        .add_attribute("listing_id", count.to_string())
        .add_attribute("creator", user_wallet.to_string()))
}

pub fn execute_change_ask(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: u64,
    new_ask: GenericBalance,
) -> Result<Response, ContractError> {
    // Ensure listing exists, sender is owner, & get listing
    let Some(listing) = listingz().may_load(deps.storage, (user_sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string()
        });
    };

    // Ensure sender is creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no finalized time
    if listing.finalized_time.is_some() {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure being prepared
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no Claimant
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Check the asking price, errors if invalid
    new_ask.check_valid()?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id),
        Some(&Listing {
            ask: new_ask,
            ..listing.clone()
        }),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("attribute", "change_listing_ask")
        .add_attribute("listing_id", listing_id.to_string()))
}

pub fn execute_add_to_listing(
    deps: DepsMut,
    balance: Balance,
    user_sender: &Addr,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Error on dupes / 0 amounts
    balance.normalized_check()?;

    // Ensure listing exists, sender is owner, & get listing
    let Some(listing) = listingz().may_load(deps.storage, (user_sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string()
        });
    };

    // Ensure sender is Creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure status is InPreperation
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no claimant <not already purchased>
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Update old listing by adding tokens
    let new_listing = {
        let old_listing = listing.for_sale.clone();
        let mut new = listing.clone();
        new.for_sale.add_tokens(balance);
        //if old_listing == x.for_sale {
        if genbal_cmp(&old_listing, &new.for_sale).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to Listing".to_string()))
        } else {
            Ok(new)
        }
    }?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id),
        Some(&new_listing),
        Some(&listing),
    )?;

    Ok(Response::new()
        .add_attribute("action", "add_funds_to_listing")
        .add_attribute("listing", listing_id.to_string()))
}

pub fn execute_add_to_listing_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Ensure listing exists, sender is owner, & get listing
    let Some(old_listing) = listingz().may_load(deps.storage, (user_wallet, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string(),
        });
    };

    // Ensure sender is Creator
    if user_wallet != &old_listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure status is InPreperation
    if old_listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no claimant <not already purchased>
    if old_listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Create updated listing
    let new_listing = {
        let old = old_listing.for_sale.clone();
        let mut new = old_listing.clone();
        new.for_sale.add_nft(nft);
        if genbal_cmp(&old, &new.for_sale).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to Listing".to_string()))
        } else {
            Ok(new)
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

pub fn execute_finalize(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    listing_id: u64,
    seconds: u64,
) -> Result<Response, ContractError> {
    // Ensure listing exists, Sender is owner & get listing
    let Some(listing) = listingz().may_load(deps.storage, (sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string(),
        });
    };

    // Ensure sender is creator
    if sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no finalized time
    if listing.finalized_time.is_some() {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure being prepared
    if listing.status != Status::BeingPrepared {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no Claimant
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // max expiration is 1209600 seconds <14 days>
    // min expiration is 600 seconds <10 minutes>
    if !(600..=1_209_600).contains(&seconds) {
        return Err(ContractError::InvalidExpiration {});
    }

    let finalized_at = env.block.time;
    let expiration = env.block.time.plus_seconds(seconds);

    listingz().replace(
        deps.storage,
        (sender, listing_id),
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
        .add_attribute("listing_id", listing_id.to_string())
        .add_attribute("expiration_seconds", expiration.to_string()))
}

/// Deletes a Listing that is either **BeingPrepared** or **Expired**,
/// and sends funds back to creator
pub fn execute_delete_listing(
    deps: DepsMut,
    env: &Env,
    sender: Addr,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Check listing exists, sender is owner & get listing
    let Some(listing) = listingz().may_load(deps.storage, (&sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string(),
        });
    };

    // Check that sender is listing creator
    if sender != listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // If listing.claimant.is_some() then listing already purchased
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Listing can only be removed if there is no expiration (meaning it's not finalized), or it's expired
    if let Some(exp) = listing.expiration_time {
        if env.block.time < exp {
            return Err(ContractError::NotExpired {
                x: exp.seconds().to_string(),
            });
        }
    }

    // Delete listing & send funds back to user
    let msgs = send_tokens_cosmos(&listing.creator, &listing.for_sale)?;

    listingz().remove(deps.storage, (&sender, listing_id))?;

    Ok(Response::new().add_attribute("Remove listing", listing_id.to_string()).add_messages(msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Purchasing
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_buy_listing(
    deps: DepsMut,
    env: &Env,
    buyer: &Addr,
    listing_id: u64,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Get bucket (will error if no bucket found)
    let the_bucket = match BUCKETS.load(deps.storage, (buyer.clone(), bucket_id)) {
        Ok(buck) => buck,
        Err(_) => return Err(ContractError::LoadBucketError {}),
    };

    // Check listing exists & get the_listing
    let Some((_pk, the_listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id)? else {
        return Err(ContractError::NotFound { typ: "Listing".to_string(), id: listing_id.to_string() });
    };

    // Check that sender is bucket owner (redundant check)
    if buyer != &the_bucket.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Check that bucket contains required purchase price
    genbal_cmp(&the_bucket.funds, &the_listing.ask)?;

    // Check that listing is ready for purchase
    if the_listing.status != Status::FinalizedReady {
        return Err(ContractError::NotPurchasable {});
    }

    // Check that the user buying is whitelisted
    if !the_listing.whitelisted_buyer.clone().map_or(true, |wl| wl == buyer.clone()) {
        return Err(ContractError::Unauthorized {});
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

    // Load current fee denom
    let fee_denom: FeeDenom = FEE_DENOM.load(deps.storage)?;

    // Calculate Fee amount for Listing (paid by Listing Buyer on withdraw)
    let (l_fee_coin, l_balance) = calc_fee_coin(&fee_denom, &the_listing.for_sale)?;

    // Delete Old Listing -> Save new listing with listing_buyer in key / creator && Fee
    listingz().remove(deps.storage, (&the_listing.creator, listing_id))?;
    listingz().save(
        deps.storage,
        (buyer, listing_id),
        &Listing {
            creator: buyer.clone(),
            claimant: Some(buyer.clone()),
            status: Status::Closed,
            fee_amount: l_fee_coin,
            for_sale: l_balance,
            ..the_listing
        },
    )?;

    // Calculate Fee amount for Bucket (paid by Listing Seller on withdraw)
    let (b_fee_coin, b_balance) = calc_fee_coin(&fee_denom, &the_bucket.funds)?;

    // Delete Old Bucket -> Save new Bucket with listing_seller in key / owner && Fee
    BUCKETS.remove(deps.storage, (buyer.clone(), bucket_id));
    BUCKETS.save(
        deps.storage,
        (the_listing.creator.clone(), bucket_id),
        &Bucket {
            owner: the_listing.creator,
            funds: b_balance,
            fee_amount: b_fee_coin.clone(),
        },
    )?;

    let res = Response::new();

    let msgs = if let Some(fee_coin) = b_fee_coin {
        let fee_msg = proto_encode(
            MsgFundCommunityPool {
                amount: vec![SdkCoin {
                    denom: fee_coin.denom,
                    amount: fee_coin.amount.to_string(),
                }],
                depositor: env.contract.address.to_string(),
            },
            "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
        )?;
        vec![fee_msg]
    } else {
        vec![]
    };

    Ok(res
        .add_messages(msgs)
        .add_attribute("action", "buy_listing")
        .add_attribute("bucket_used", bucket_id.to_string())
        .add_attribute("listing_purchased:", listing_id.to_string()))
}

pub fn execute_withdraw_purchased(
    deps: DepsMut,
    withdrawer: &Addr,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Get listing
    let Some((_pk, the_listing)): Option<(_, Listing)> = listingz().idx.id.item(deps.storage, listing_id)? else {
        return Err(ContractError::NotFound { typ: "Listing".to_string(), id: listing_id.to_string() });
    };

    // Check and pull out claimant
    let listing_claimant = the_listing.claimant.clone().ok_or(ContractError::Unauthorized {})?;

    // Check that withdrawer is the claimant
    if withdrawer != &listing_claimant {
        return Err(ContractError::Unauthorized {});
    };

    // Check that status is Closed
    if the_listing.status != Status::Closed {
        return Err(ContractError::Unauthorized {});
    };

    // Delete Listing
    listingz().remove(deps.storage, (&listing_claimant, listing_id))?;

    let withdraw_msgs = the_listing.withdraw_msgs()?;

    Ok(Response::new()
        .add_attribute("Action", "withdraw_purchased")
        .add_attribute("listing_id", listing_id.to_string())
        .add_messages(withdraw_msgs))
}
