use crate::execute_imports::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_create_bucket(
    deps: DepsMut,
    funds: &Balance,
    creator: &Addr,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Check ID value
    max(bucket_id)?;

    // If this bucket_id has been used before, it cannot be used again
    if BUCKET_ID_USED.has(deps.storage, bucket_id) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Check that bucket_id isn't used (edge case)
    if BUCKETS.has(deps.storage, (creator.clone(), bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Error if funds contains duplicates or 0 balances
    // - Prefer this over normalize to abort rather than alter balance sent
    funds.normalized_check()?;

    // Save bucket
    BUCKETS.save(
        deps.storage,
        (creator.clone(), bucket_id),
        &Bucket {
            owner: creator.clone(),
            funds: GenericBalance::from_balance(funds),
            fee_amount: None,
        },
    )?;

    // Save bucket_id as used
    BUCKET_ID_USED.save(deps.storage, bucket_id, &true)?;

    Ok(Response::new()
        .add_attribute("action", "create_bucket")
        .add_attribute("bucket_id", bucket_id.to_string()))
}

pub fn execute_create_bucket_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Check ID value
    max(bucket_id)?;

    // If this bucket_id has been used before, it cannot be used again
    if BUCKET_ID_USED.has(deps.storage, bucket_id) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Check that bucket_id isn't used (edge case)
    if BUCKETS.has(deps.storage, (user_wallet.clone(), bucket_id)) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // NFT validation checks are handled in receiver wrapper
    BUCKETS.save(
        deps.storage,
        (user_wallet.clone(), bucket_id),
        &Bucket {
            owner: user_wallet.clone(),
            funds: GenericBalance::from_nft(nft),
            fee_amount: None,
        },
    )?;

    // Save bucket_id as used
    BUCKET_ID_USED.save(deps.storage, bucket_id, &true)?;

    Ok(Response::new()
        .add_attribute("action", "create_bucket")
        .add_attribute("bucket_id", bucket_id.to_string()))
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
    let new_bucket: Bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_tokens(funds);
        if genbal_cmp(&old_funds, &new_bucket.funds).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Check that bucket funds are valid (specifically not over 25)
    new_bucket.funds.check_valid()?;

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
    let new_bucket: Bucket = {
        let old_funds = the_bucket.funds.clone();
        let mut new_bucket = the_bucket;
        new_bucket.funds.add_nft(nft);
        if genbal_cmp(&old_funds, &new_bucket.funds).is_ok() {
            Err(ContractError::ErrorAdding("NFT to bucket".to_string()))
        } else {
            Ok(new_bucket)
        }
    }?;

    // Check that bucket funds are valid (specifically not over 35)
    new_bucket.funds.check_valid()?;

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
    env: &Env,
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
    let msgs = the_bucket.withdraw_msgs(env.contract.address.clone())?;

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
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Check ID value
    max(listing_id)?;

    // Error if funds_sent contains duplicates or 0 balances
    // - Prefer this over normalize to abort rather than alter balance sent
    funds_sent.normalized_check()?;

    // If listing_id has been used, it cannot be used again
    if LISTING_ID_USED.has(deps.storage, listing_id) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Check edge case that ID is taken
    if (listingz().idx.id.item(deps.storage, listing_id)?).is_some() {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Get whitelisted buyer | Errors if invalid address
    let wl_buyer = createlistingmsg
        .whitelisted_buyer
        .map(|address| deps.api.addr_validate(&address))
        .transpose()
        .map_err(|_e| ContractError::GenericError("Invalid whitelisted buyer".to_string()))?;

    // Error if whitelisted buyer is listing creator
    // Is there ever a situation where someone might want to do this?
    if let Some(wlbuyer) = &wl_buyer {
        if wlbuyer.eq(user_address) {
            return Err(ContractError::GenericError(
                "Whitelisted buyer should not be the same as Listing Creator".to_string(),
            ));
        }
    }

    // Validate ask
    let valid_ask: GenericBalance = createlistingmsg.ask.validate(&deps)?;

    // Save listing
    listingz().save(
        deps.storage,
        (user_address, listing_id),
        &Listing {
            creator: user_address.clone(),
            id: listing_id,
            status: Status::Open,
            claimant: None,
            whitelisted_buyer: wl_buyer,
            for_sale: GenericBalance::from_balance(funds_sent),
            ask: valid_ask,
            fee_amount: None,
        },
    )?;

    // Mark this ID as used
    LISTING_ID_USED.save(deps.storage, listing_id, &true)?;

    Ok(Response::new()
        .add_attribute("action", "create_listing")
        .add_attribute("listing_id", listing_id.to_string())
        .add_attribute("creator", user_address.to_string()))
}

pub fn execute_create_listing_cw721(
    deps: DepsMut,
    user_wallet: &Addr,
    nft: Nft,
    createlistingmsg: CreateListingMsg,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Check ID value
    max(listing_id)?;

    // If this listing_id has been used, it cannot be used again
    if LISTING_ID_USED.has(deps.storage, listing_id) {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Edge case check that ID isn't taken
    if (listingz().idx.id.item(deps.storage, listing_id)?).is_some() {
        return Err(ContractError::IdAlreadyExists {});
    }

    // Get whitelisted buyer | Errors if invalid address
    let wl_buyer = createlistingmsg
        .whitelisted_buyer
        .map(|address| deps.api.addr_validate(&address))
        .transpose()
        .map_err(|_| ContractError::GenericError("Invalid whitelisted buyer".to_string()))?;

    // Error if whitelisted buyer is listing creator
    // Is there ever a situation where someone might want to do this?
    if let Some(wlbuyer) = &wl_buyer {
        if wlbuyer.eq(user_wallet) {
            return Err(ContractError::GenericError(
                "Whitelisted buyer should not be the same as Listing Creator".to_string(),
            ));
        }
    }

    // Validate ask
    let valid_ask: GenericBalance = createlistingmsg.ask.validate(&deps)?;

    listingz().save(
        deps.storage,
        (user_wallet, listing_id),
        &Listing {
            creator: user_wallet.clone(),
            id: listing_id,
            status: Status::Open,
            claimant: None,
            whitelisted_buyer: wl_buyer,
            for_sale: GenericBalance::from_nft(nft),
            ask: valid_ask,
            fee_amount: None,
        },
    )?;

    // Mark this listing_id as used
    LISTING_ID_USED.save(deps.storage, listing_id, &true)?;

    Ok(Response::new()
        .add_attribute("action", "create_cw721_listing")
        .add_attribute("listing_id", listing_id.to_string())
        .add_attribute("creator", user_wallet.to_string()))
}

pub fn execute_change_ask(
    deps: DepsMut,
    user_sender: &Addr,
    listing_id: u64,
    new_ask: GenericBalanceUnvalidated,
) -> Result<Response, ContractError> {
    // Ensure listing exists, sender is owner, & get listing
    let Some(listing): Option<Listing> = listingz().may_load(deps.storage, (user_sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string()
        });
    };

    // Ensure sender is creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure listing is still open (has not been purchased)
    if listing.status != Status::Open {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no Claimant (listing has been purchased)
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Validate ask
    let valid_ask: GenericBalance = new_ask.validate(&deps)?;

    listingz().replace(
        deps.storage,
        (user_sender, listing_id),
        Some(&Listing {
            ask: valid_ask,
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
    let Some(listing): Option<Listing> = listingz().may_load(deps.storage, (user_sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string()
        });
    };

    // Ensure sender is creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure listing is still open (has not been purchased)
    if listing.status != Status::Open {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure no Claimant (listing has been purchased)
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Update old listing by adding tokens
    let new_listing: Listing = {
        let old_listing = listing.for_sale.clone();
        let mut new = listing.clone();
        new.for_sale.add_tokens(balance);
        // Error if tokens not added
        if genbal_cmp(&old_listing, &new.for_sale).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to Listing".to_string()))
        } else {
            Ok(new)
        }
    }?;

    // Check that new listing doesn't have over MAX_NUM_ASSETS
    let _ = new_listing
        .for_sale
        .native.len()
        .checked_add(new_listing.for_sale.cw20.len())
        .and_then(|v| v.checked_add(new_listing.for_sale.nfts.len()))
        .ok_or_else(|| ContractError::GenericError(format!("Listing cannot contain over {} items", MAX_NUM_ASSETS)))
        .and_then(|v| {
            if v as u32 > MAX_NUM_ASSETS {
                return Err(ContractError::GenericError(format!("Listing cannot contain over {} items", MAX_NUM_ASSETS)));
            }
            Ok(())
        })?;

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
    user_sender: &Addr,
    nft: Nft,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Ensure listing exists, sender is owner, & get listing
    let Some(listing): Option<Listing> = listingz().may_load(deps.storage, (user_sender, listing_id))? else {
        return Err(ContractError::NotFound {
            typ: "Listing".to_string(),
            id: listing_id.to_string(),
        });
    };

    // Ensure sender is Creator
    if user_sender != &listing.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure listing is still open (has not been purchased)
    if listing.status != Status::Open {
        return Err(ContractError::AlreadyFinalized {});
    }

    // Ensure no claimant (listing has already been purchased)
    if listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Create updated listing
    let new_listing: Listing = {
        let old = listing.for_sale.clone();
        let mut new = listing.clone();
        new.for_sale.add_nft(nft);
        if genbal_cmp(&old, &new.for_sale).is_ok() {
            Err(ContractError::ErrorAdding("Tokens to Listing".to_string()))
        } else {
            Ok(new)
        }
    }?;

    // Check that new_listings for_sale is valid (over MAX_NUM_ASSETS or duplicate NFTs)
    new_listing.for_sale.check_valid()?;

    // Replace old listing with new listing
    listingz().replace(
        deps.storage,
        (user_sender, listing_id),
        Some(&new_listing),
        Some(&listing),
    )?;

    Ok(Response::new().add_attribute("Added NFT to listing", listing_id.to_string()))
}


/// Deletes a Listing that is `Status::Open` and sends funds back to creator
pub fn execute_delete_listing(
    deps: DepsMut,
    _env: &Env,
    sender: Addr,
    listing_id: u64,
) -> Result<Response, ContractError> {
    // Check listing exists, sender is owner & get listing
    let Some(listing): Option<Listing> = listingz().may_load(deps.storage, (&sender, listing_id))? else {
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

    // If status is Closed then the listing has already been purchased
    if listing.status == Status::Closed {
        return Err(ContractError::Unauthorized {});
    }

    // Delete listing & send funds back to user
    // Can use send_tokens_cosmos instead of withdraw_msgs because listing has not
    // been purchased (so there won't be a fee)
    let msgs = send_tokens_cosmos(&listing.creator, &listing.for_sale)?;

    listingz().remove(deps.storage, (&sender, listing_id))?;

    Ok(Response::new().add_attribute("Remove listing", listing_id.to_string()).add_messages(msgs))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Purchasing
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn execute_buy_listing(
    deps: DepsMut,
    _env: &Env,
    buyer: &Addr,
    listing_id: u64,
    bucket_id: u64,
) -> Result<Response, ContractError> {
    // Get bucket (will error if no bucket found)
    let the_bucket: Bucket = match BUCKETS.load(deps.storage, (buyer.clone(), bucket_id)) {
        Ok(buck) => Ok(buck),
        Err(_) => Err(ContractError::LoadBucketError {}),
    }?;

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

    // Check that the user buying is whitelisted (or there is no whitelisted buyer)
    if !the_listing.whitelisted_buyer.clone().map_or(true, |wl| wl == buyer.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    // Check that the listing has not already been purchased
    if the_listing.status == Status::Closed || the_listing.claimant.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    // Load current fee denom
    let fee_denom: FeeDenom = FEE_DENOM.load(deps.storage)?;

    // Calculate Fee amount for Listing (paid by Listing Buyer on withdraw)
    let (l_fee_coin, mut l_balance) = calc_fee_coin(&fee_denom, &the_listing.for_sale)?;

    // Calculate Fee amount for Bucket (paid by Listing Seller on withdraw)
    let (b_fee_coin, mut b_balance) = calc_fee_coin(&fee_denom, &the_bucket.funds)?;

    // On the NFTs that the seller is selling, the Seller should pay royalties
    // out of the proceeds they get from the sale

    // On the NFTs that the buyer is paying with, the Buyer should pay royalties
    // on the assets they're purchasing

    // NFT contracts that seller is selling (duplicates removed)
    let seller_nft_contracts = the_listing.for_sale.nfts
        .iter()
        .map(|nft| nft.contract_address.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<String>>();

    // NFT contracts that buyer is paying with (duplicates removed)
    let buyer_nft_contracts = the_bucket.funds.nfts
        .iter()
        .map(|nft| nft.contract_address.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<String>>();

    // Mutable response
    let mut res = Response::<cosmwasm_std::Empty>::new();

    // Royalty Registry address
    let Some(royalty_reg): Option<Addr> = ROYALTY_REGISTRY.load(deps.storage)? else {
        return Err(ContractError::GenericError("No royalty registry".to_string()));
    };

    // Listing Seller is getting the Bucket, therefore the Listing Seller
    // should pay royalties out of their proceeds which is the Bucket
    // (NFTs being sold are paid royalties from the purchase price)
    let final_bucket_balance = match seller_nft_contracts.len() {
        0 => b_balance,
        _ => {
            // Query contract to get royalty responses
            let royalty_responses: Vec<Option<RoyaltyInfo>> = deps.querier.query_wasm_smart(
                royalty_reg.clone(), &RoyaltyQueryMsg::RoyaltyInfoMulti { nft_contracts: seller_nft_contracts }
            )?;

            // Get Royalty Cosmos Msgs & subtract royalty amounts from bucket
            let (royalty_msgs, bips_paid) = b_balance.royalties(royalty_responses)?;

            res = res.add_attribute("Total bips paid by seller from sale proceeds", bips_paid.to_string());

            res = res.add_messages(royalty_msgs);

            // Return updated GenericBalance (the Listing Seller's proceeds)
            b_balance
        }
    };

    // Listing Buyer is getting the Listing, therefore the Listing Buyer
    // should pay royalties out of their purchase which is the Listing
    // (NFTs used to purchase are paid royalties from the assets they're used to purchase)
    let final_listing_balance = match buyer_nft_contracts.len() {
        0 => l_balance,
        _ => {
            let royalty_responses: Vec<Option<RoyaltyInfo>> = deps.querier.query_wasm_smart(
                royalty_reg, &RoyaltyQueryMsg::RoyaltyInfoMulti { nft_contracts: buyer_nft_contracts }
            )?;

            // Get Royalty CosmosMSgs & subtract royalty amounts from Listing
            let (royalty_msgs, bips_paid) = l_balance.royalties(royalty_responses)?;

            res = res.add_attribute("Total bips paid by buyer from purchased assets", bips_paid.to_string());

            res = res.add_messages(royalty_msgs);

            // Updated GenericBalance (the Buyer's purchased Listing)
            l_balance
        }
    };

    // Delete Old Listing
    listingz().remove(deps.storage, (&the_listing.creator, listing_id))?;
    // Save new Listing with
    // - Listing Buyer in key & creator & claimant
    // - Community Pool fee added
    // - Any NFT royalty payments removed
    listingz().save(
        deps.storage,
        (buyer, listing_id),
        &Listing {
            creator: buyer.clone(),
            claimant: Some(buyer.clone()),
            status: Status::Closed,
            fee_amount: l_fee_coin,
            for_sale: final_listing_balance,
            ..the_listing
        },
    )?;

    // Delete Old Bucket
    BUCKETS.remove(deps.storage, (buyer.clone(), bucket_id));
    // Save new Bucket with
    // - Listing Seller in key & owner
    // - Community Pool fee added
    // - Any NFT royalty payments removed
    BUCKETS.save(
        deps.storage,
        (the_listing.creator.clone(), bucket_id),
        &Bucket {
            owner: the_listing.creator,
            funds: final_bucket_balance,
            fee_amount: b_fee_coin,
        },
    )?;

    res = res
        .add_attribute("action", "buy_listing")
        .add_attribute("bucket_used", bucket_id.to_string())
        .add_attribute("listing_purchased:", listing_id.to_string());

    Ok(res)
}

pub fn execute_withdraw_purchased(
    deps: DepsMut,
    env: &Env,
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

    let withdraw_msgs = the_listing.withdraw_msgs(env.contract.address.clone())?;

    Ok(Response::new()
        .add_attribute("Action", "withdraw_purchased")
        .add_attribute("listing_id", listing_id.to_string())
        .add_messages(withdraw_msgs))
}
