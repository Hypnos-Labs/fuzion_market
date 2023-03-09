#[cfg(not(feature = "library"))]
use crate::contract_imports::*;
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use cw721::Cw721QueryMsg;

const CONTRACT_NAME: &str = "crates.io:fuzion_market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WEEK_IN_SECS: u64 = 604800;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    LISTING_COUNT.save(deps.storage, &1)?;

    BUCKET_COUNT.save(deps.storage, &1)?;

    FEE_DENOM.save(deps.storage, &FeeDenom::JUNO(env.block.time.seconds()))?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Execute
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {

    // Automatically cycle fee on any execute call if FeeCycle is ready
    let (_res, deps) = execute_cycle_fee(deps, &env)?;

    // TO-DO: 
    // - Pass response to every execute to add attributes/messages
    // - See if this is worth the extra read/writes

    match msg {
        //ExecuteMsg::FeeCycle => execute_cycle_fee(deps, env),
        ExecuteMsg::FeeCycle => Ok(Response::default()),

        // ~~~~ Receive Wrappers ~~~~ //
        ExecuteMsg::Receive(receive_msg) => execute_receive(deps, &env, &info, &receive_msg),
        ExecuteMsg::ReceiveNft(receive_nft_msg) => execute_receive_nft(deps, info, receive_nft_msg),

        // ~~~~ Listing Executions ~~~~ //
        ExecuteMsg::CreateListing {
            create_msg,
        } => execute_create_listing(deps, &info.sender, &Balance::from(info.funds), create_msg),
        ExecuteMsg::AddToListing {
            listing_id,
        } => execute_add_to_listing(deps, Balance::from(info.funds), &info.sender, listing_id),
        ExecuteMsg::ChangeAsk {
            listing_id,
            new_ask,
        } => execute_change_ask(deps, &info.sender, listing_id, new_ask),
        ExecuteMsg::Finalize {
            listing_id,
            seconds,
        } => execute_finalize(deps, &env, &info.sender, listing_id, seconds),
        ExecuteMsg::DeleteListing {
            listing_id,
        } => execute_delete_listing(deps, &env, info.sender, listing_id),

        // ~~~~ Bucket Executions ~~~~ //
        ExecuteMsg::CreateBucket {} => {
            execute_create_bucket(deps, &Balance::from(info.funds), &info.sender)
        }
        ExecuteMsg::AddToBucket {
            bucket_id,
        } => execute_add_to_bucket(deps, Balance::from(info.funds), &info.sender, bucket_id),
        ExecuteMsg::RemoveBucket {
            bucket_id,
        } => execute_withdraw_bucket(deps, &env, &info.sender, bucket_id),

        // ~~~~ Marketplace Executions ~~~~ //
        ExecuteMsg::BuyListing {
            listing_id,
            bucket_id,
        } => execute_buy_listing(deps, &env, &info.sender, listing_id, bucket_id),
        ExecuteMsg::WithdrawPurchased {
            listing_id,
        } => execute_withdraw_purchased(deps, &env, &info.sender, listing_id),
    }
}

/// Anyone can call this, but it will only take effect
/// if WEEK_IN_SECS has passed since last cycle
pub fn execute_cycle_fee<'a>(deps: DepsMut<'a>, env: &Env) -> Result<(Response, DepsMut<'a>), ContractError> {
    // updatable = "last updated" + 1 week, used to check if it's been 1 week since the last time this was
    //             called successfully
    // new = current time // if the check passes, this is saved as the new "last updated" to check against
    //        the next time this is called
    let (updatable, new) = match FEE_DENOM.load(deps.storage)? {
        FeeDenom::JUNO(last) => {
            (last.saturating_add(WEEK_IN_SECS), FeeDenom::USDC(env.block.time.seconds()))
        }
        FeeDenom::USDC(lastx) => {
            (lastx.saturating_add(WEEK_IN_SECS), FeeDenom::JUNO(env.block.time.seconds()))
        }
    };

    if env.block.time.seconds() <= updatable {
        // Not updatable, return empty response
        return Ok((Response::new(), deps));
    } else {
        // Is updatable, cycle fee
        FEE_DENOM.save(deps.storage, &new)?;
        return Ok((Response::new().add_attribute("Cycled Fee", new.value()), deps));
    }

    // if current block is <= updatable Error (Cycle every week)
    // if env.block.time.seconds() <= updatable {
    //     return Err(ContractError::GenericError("FeeDenom not yet ready to cycle".to_string()));
    // };

    // // Ready to cycle
    // FEE_DENOM.save(deps.storage, &new)?;

    // Ok(Response::new().add_attribute("Cycle", "Fee"))
}

// CW20 Filter
pub fn execute_receive(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    wrapper: &Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // Ensure this did not contain anything
    if !info.funds.is_empty() {
        return Err(ContractError::GenericError("Invalid cw20 receive".to_string()));
    }

    // This doesn't guarantee that sender a cw20, but aids in verification
    let _x: TokenInfoResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone(), &Cw20QueryMsg::TokenInfo {})
        .map_err(|_e| ContractError::GenericError("Invalid CW20 Spec".to_string()))?;

    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let user_wallet = deps.api.addr_validate(&wrapper.sender)?;

    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: wrapper.amount,
    });

    match msg {
        ReceiveMsg::CreateListingCw20 {
            create_msg,
        } => execute_create_listing(deps, &user_wallet, &balance, create_msg),
        ReceiveMsg::AddToListingCw20 {
            listing_id,
        } => execute_add_to_listing(deps, balance, &user_wallet, listing_id),
        ReceiveMsg::CreateBucketCw20 {} => execute_create_bucket(deps, &balance, &user_wallet),
        ReceiveMsg::AddToBucketCw20 {
            bucket_id,
        } => execute_add_to_bucket(deps, balance, &user_wallet, bucket_id),
    }
}

// CW721 filter
pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    // Ensure this did not contain anything
    if !info.funds.is_empty() {
        return Err(ContractError::GenericError("Invalid cw721 receive".to_string()));
    }

    // This doesn't guarantee that it's a cw721, but aids in verification
    let _x: cw721::ContractInfoResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone(), &Cw721QueryMsg::ContractInfo {})
        .map_err(|_e| ContractError::GenericError("Invalid CW721 Spec".to_string()))?;

    let msg: ReceiveNftMsg = from_binary(&wrapper.msg)?;
    let user_wallet = deps.api.addr_validate(&wrapper.sender)?;

    let incoming_nft: Nft = Nft {
        contract_address: info.sender,
        token_id: wrapper.token_id,
    };

    match msg {
        ReceiveNftMsg::CreateListingCw721 {
            create_msg,
        } => execute_create_listing_cw721(deps, &user_wallet, incoming_nft, create_msg),
        ReceiveNftMsg::AddToListingCw721 {
            listing_id,
        } => execute_add_to_listing_cw721(deps, &user_wallet, incoming_nft, listing_id),
        ReceiveNftMsg::CreateBucketCw721 {} => {
            execute_create_bucket_cw721(deps, &user_wallet, incoming_nft)
        }
        ReceiveNftMsg::AddToBucketCw721 {
            bucket_id,
        } => execute_add_to_bucket_cw721(deps, &user_wallet, incoming_nft, bucket_id),
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFeeDenom {} => to_binary(&get_fee_denom(deps)?),
        QueryMsg::GetListingsByOwner {
            owner,
            page_num,
        } => to_binary(&get_listings_by_owner(deps, owner.as_str(), page_num)?),
        QueryMsg::GetListingsByWhitelist {
            owner,
        } => to_binary(&get_whitelisted(deps, env, owner)?),
        QueryMsg::GetBuckets {
            bucket_owner,
            page_num,
        } => to_binary(&get_buckets(deps, bucket_owner.as_str(), page_num)?),
        QueryMsg::GetListingsForMarket {
            page_num,
        } => to_binary(&get_listings_for_market(deps, env, page_num)?),
        // QueryMsg::GetListingInfo {
        //     listing_id,
        // } => to_binary(&get_single_listing(deps, listing_id)?),
    }
}
