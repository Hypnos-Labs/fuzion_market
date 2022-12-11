#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg};
use cw721::Cw721ReceiveMsg;

use crate::error::ContractError;
use crate::execute::*;
use crate::msg::*;
use crate::query::*;
use crate::state::*;
use crate::utils::*;
use std::str;

const CONTRACT_NAME: &str = "crates.io:cyberswap_nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    //let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());

    //let validated = deps.api.addr_validate(&admin)?;

    // let validated = match msg.admin {
    //     Some(a) => deps.api.addr_validate(&a)?,
    //     None => info.sender
    // };

    let validated = info.sender;

    //let native_whitelist: Vec<(String, String)> = vec![("JUNO".to_string(), "ujunox".to_string())];
    let Some(native_whitelist): Option<Vec<(String, String)>> = msg.native_whitelist else {
        return Err(ContractError::MissingInit("Native Whitelist Missing".to_string()));
    };

    //let msg_cw20_whitelist = msg.cw20_whitelist;
    let Some(cw20_wl) = msg.cw20_whitelist else {
        return Err(ContractError::MissingInit("CW20 Whitelist Missing".to_string()));
    };

    let Some(nft_wl) = msg.nft_whitelist else {
        return Err(ContractError::MissingInit("NFT Whitelist Missing".to_string()));
    };

    let cw20_whitelist = cw20_wl
        .iter()
        .map(|cw20| (cw20.0.clone(), Addr::unchecked(cw20.1.clone())))
        .collect();

    let nft_whitelist = nft_wl
        .iter()
        .map(|nft| (nft.0.clone(), Addr::unchecked(nft.1.clone())))
        .collect();

    CONFIG.save(
        deps.storage,
        &Config {
            admin: validated,
            whitelist_native: native_whitelist,
            whitelist_cw20: cw20_whitelist,
            whitelist_nft: nft_whitelist,
        },
    )?;

    Ok(Response::default())
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
    match msg {

        ExecuteMsg::AddToWhitelist { type_adding, to_add } => add_to_whitelist(deps, info.sender, type_adding, to_add),

        ExecuteMsg::Receive(receive_msg) => execute_receive(deps, env, info, receive_msg),
        ExecuteMsg::ReceiveNft(receive_nft_msg) => execute_receive_nft(deps, info, receive_nft_msg),

        ExecuteMsg::CreateListing { create_msg } => {
            execute_create_listing(deps, &info.sender, Balance::from(info.funds), create_msg)
        }

        // Adding native tokens to a sale
        ExecuteMsg::AddFundsToSaleNative { listing_id } => {
            execute_add_funds_to_sale(deps, Balance::from(info.funds), &info.sender, listing_id)
        }
        // Changing price of a sale
        ExecuteMsg::ChangeAsk {
            listing_id,
            new_ask,
        } => execute_change_ask(deps, &info.sender, listing_id, new_ask),
        // Can only remove if listing has not yet been finalized
        ExecuteMsg::RemoveListing { listing_id } => {
            execute_remove_listing(deps, &info.sender, listing_id)
        }
        // Finalizes listing for sale w/ expiration
        ExecuteMsg::Finalize {
            listing_id,
            seconds,
        } => execute_finalize(deps, env, &info.sender, listing_id, seconds),
        // Refunds unpurchased listing if expired
        ExecuteMsg::RefundExpired { listing_id } => {
            execute_refund(deps, env, &info.sender, listing_id)
        }

        // Create a bucket
        ExecuteMsg::CreateBucket { bucket_id } => {
            execute_create_bucket(deps, Balance::from(info.funds), &info.sender, bucket_id)
        }
        // Add funds to bucket
        ExecuteMsg::AddToBucket { bucket_id } => {
            execute_add_to_bucket(deps, Balance::from(info.funds), &info.sender, bucket_id)
        }
        // Remove and delete a bucket
        ExecuteMsg::RemoveBucket { bucket_id } => {
            execute_withdraw_bucket(deps, &info.sender, bucket_id)
        }

        ExecuteMsg::BuyListing {
            listing_id,
            bucket_id,
        } => execute_buy_listing(deps, env, &info.sender, listing_id, bucket_id),
        ExecuteMsg::WithdrawPurchased { listing_id } => {
            execute_withdraw_purchased(deps, &info.sender, listing_id)
        }
    }
}

// cw20 filter
pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;

    let user_wallet = &deps.api.addr_validate(&wrapper.sender)?;

    let balance = Balance::Cw20(Cw20CoinVerified {
        //cw20 contract this message was sent from
        address: info.sender.clone(),
        amount: wrapper.amount,
    });

    // is_balance_whitelisted check in each message individually
    match msg {
        // Create listing with Cw20's initially
        ReceiveMsg::CreateListingCw20 { create_msg } => {
            execute_create_listing_cw20(deps, user_wallet, &info.sender, balance, create_msg)
        }
        // Add Cw20's to sale
        ReceiveMsg::AddFundsToSaleCw20 { listing_id } => {
            execute_add_funds_to_sale(deps, balance, user_wallet, listing_id)
        }
        // Create Bucket with Cw20's initially
        ReceiveMsg::CreateBucketCw20 { bucket_id } => {
            execute_create_bucket(deps, balance, user_wallet, bucket_id)
        }
        // Add Cw20's to bucket
        ReceiveMsg::AddToBucketCw20 { bucket_id } => {
            execute_add_to_bucket(deps, balance, user_wallet, bucket_id)
        }
    }
}

// cw721 filter
pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    // Whitelist check
    let config = CONFIG.load(deps.storage)?;
    is_nft_whitelisted(&info.sender, &config)?;

    // Pull message and wallet to avoid clone in NFT construction
    let msg: ReceiveNftMsg = from_binary(&wrapper.msg)?;
    let user_wallet = &deps.api.addr_validate(&wrapper.sender)?;

    // Construct NFT
    let incoming_nft: Nft = Nft {
        contract_address: info.sender,
        token_id: wrapper.token_id,
    };

    match msg {
        ReceiveNftMsg::CreateListingCw721 { create_msg } => {
            execute_create_listing_cw721(deps, user_wallet, incoming_nft, create_msg)
        }

        ReceiveNftMsg::AddToListingCw721 { listing_id } => {
            execute_add_to_sale_cw721(deps, user_wallet, incoming_nft, listing_id)
        }

        ReceiveNftMsg::CreateBucketCw721 { bucket_id } => {
            execute_create_bucket_cw721(deps, user_wallet, incoming_nft, bucket_id)
        }

        ReceiveNftMsg::AddToBucketCw721 { bucket_id } => {
            execute_add_to_bucket_cw721(deps, user_wallet, incoming_nft, bucket_id)
        }
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Get admin<shocking>
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps)?),
        // Get Config
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
        // Get a specific listing info
        QueryMsg::GetListingInfo { listing_id } => to_binary(&get_listing_info(deps, listing_id)?),
        // Get listings by owner
        QueryMsg::GetListingsByOwner { owner } => to_binary(&get_listings_by_owner(deps, owner)?),
        // Get all listings (take 100)
        QueryMsg::GetAllListings {} => to_binary(&get_all_listings(deps)?),
        // Get buckets owned by 1 address
        QueryMsg::GetBuckets { bucket_owner } => to_binary(&get_buckets(deps, bucket_owner)?),
        // Get listings finalized within 2 weeks & paginate for page
        QueryMsg::GetListingsForMarket { page_num } => {
            to_binary(&get_listings_for_market(deps, env, page_num)?)
        }
    }
}