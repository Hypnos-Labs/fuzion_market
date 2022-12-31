#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg};
use cw721::Cw721ReceiveMsg;

use crate::error::ContractError;
use crate::execute::{
    add_to_whitelist, execute_add_funds_to_sale, execute_add_to_bucket,
    execute_add_to_bucket_cw721, execute_add_to_sale_cw721, execute_buy_listing,
    execute_change_ask, execute_create_bucket, execute_create_bucket_cw721, execute_create_listing,
    execute_create_listing_cw20, execute_create_listing_cw721, execute_finalize, execute_refund,
    execute_remove_listing, execute_withdraw_bucket, execute_withdraw_purchased,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, ReceiveNftMsg};
use crate::query::{
    get_admin, get_all_listings, get_buckets, get_config, get_listing_info, get_listings_by_owner,
    get_listings_for_market,
};
use crate::state::{Config, Nft, CONFIG, WHITELIST_CW20, WHITELIST_NATIVE, WHITELIST_NFT};
use crate::utils::is_nft_whitelisted;
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

    let validated_admin = match msg.admin {
        Some(adm) => deps.api.addr_validate(&adm)?,
        None => info.sender,
    };

    CONFIG
        .save(
            deps.storage,
            &Config {
                admin: validated_admin,
            },
        )
        .map_err(|_e| ContractError::InitInvalidAddr)?;

    for native in msg.native_whitelist {
        WHITELIST_NATIVE
            .save(deps.storage, native.clone(), &true)
            .map_err(|_e| ContractError::InitInvalidAddr)?;
    }

    for cw20 in msg.cw20_whitelist {
        let Ok(address) = deps.api.addr_validate(&cw20) else {
            return Err(ContractError::InitInvalidAddr);
        };

        WHITELIST_CW20
            .save(deps.storage, address, &true)
            .map_err(|_e| ContractError::InitInvalidAddr)?;
    }

    for nft in msg.nft_whitelist {
        let Ok(address) = deps.api.addr_validate(&nft) else {
            return Err(ContractError::InitInvalidAddr);
        };

        WHITELIST_NFT
            .save(deps.storage, address, &true)
            .map_err(|_e| ContractError::InitInvalidAddr)?;
    }

    Ok(Response::new().add_attribute("Called", "Instantiate"))
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
        // ~~~~
        // Receive Wrappers
        ExecuteMsg::Receive(receive_msg) => execute_receive(deps, &env, &info, &receive_msg),
        ExecuteMsg::ReceiveNft(receive_nft_msg) => execute_receive_nft(deps, info, receive_nft_msg),
        // ~~~~
        // Admin only
        ExecuteMsg::AddToWhitelist {
            type_adding,
            to_add,
        } => add_to_whitelist(deps, &info.sender, type_adding, to_add),
        // ~~~~
        // Listing Executions
        ExecuteMsg::CreateListing {
            create_msg,
        } => execute_create_listing(deps, &info.sender, &Balance::from(info.funds), create_msg),
        ExecuteMsg::AddFundsToSaleNative {
            listing_id,
        } => execute_add_funds_to_sale(deps, Balance::from(info.funds), &info.sender, listing_id),
        ExecuteMsg::ChangeAsk {
            listing_id,
            new_ask,
        } => execute_change_ask(deps, &info.sender, listing_id, new_ask),
        ExecuteMsg::RemoveListing {
            listing_id,
        } => execute_remove_listing(deps, &info.sender, listing_id),
        ExecuteMsg::Finalize {
            listing_id,
            seconds,
        } => execute_finalize(deps, &env, &info.sender, listing_id, seconds),
        ExecuteMsg::RefundExpired {
            listing_id,
        } => execute_refund(deps, &env, &info.sender, listing_id),
        // ~~~~
        // Bucket Executions <purchasing>
        ExecuteMsg::CreateBucket {
            bucket_id,
        } => execute_create_bucket(deps, &Balance::from(info.funds), &info.sender, &bucket_id),
        ExecuteMsg::AddToBucket {
            bucket_id,
        } => execute_add_to_bucket(deps, Balance::from(info.funds), &info.sender, bucket_id),
        ExecuteMsg::RemoveBucket {
            bucket_id,
        } => execute_withdraw_bucket(deps, &info.sender, &bucket_id),
        // ~~~~
        // Marketplace Executions
        ExecuteMsg::BuyListing {
            listing_id,
            bucket_id,
        } => execute_buy_listing(deps, &env, &info.sender, listing_id, &bucket_id),
        ExecuteMsg::WithdrawPurchased {
            listing_id,
        } => execute_withdraw_purchased(deps, &info.sender, listing_id),
    }
}

// CW20 Filter
pub fn execute_receive(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    wrapper: &Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let user_wallet = deps.api.addr_validate(&wrapper.sender)?;

    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: wrapper.amount,
    });

    match msg {
        ReceiveMsg::CreateListingCw20 {
            create_msg,
        } => execute_create_listing_cw20(deps, &user_wallet, &info.sender, &balance, create_msg),
        ReceiveMsg::AddFundsToSaleCw20 {
            listing_id,
        } => execute_add_funds_to_sale(deps, balance, &user_wallet, listing_id),
        ReceiveMsg::CreateBucketCw20 {
            bucket_id,
        } => execute_create_bucket(deps, &balance, &user_wallet, &bucket_id),
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
    //let config = CONFIG.load(deps.storage)?;
    is_nft_whitelisted(&info.sender, &deps)?;

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
        } => execute_add_to_sale_cw721(deps, &user_wallet, incoming_nft, listing_id),
        ReceiveNftMsg::CreateBucketCw721 {
            bucket_id,
        } => execute_create_bucket_cw721(deps, &user_wallet, incoming_nft, &bucket_id),
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
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps)?),
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
        QueryMsg::GetListingInfo {
            listing_id,
        } => to_binary(&get_listing_info(deps, listing_id)?),
        QueryMsg::GetListingsByOwner {
            owner,
        } => to_binary(&get_listings_by_owner(deps, &owner)?),
        QueryMsg::GetAllListings {} => to_binary(&get_all_listings(deps)?),
        QueryMsg::GetBuckets {
            bucket_owner,
        } => to_binary(&get_buckets(deps, &bucket_owner)?),
        QueryMsg::GetListingsForMarket {
            page_num,
        } => to_binary(&get_listings_for_market(deps, &env, page_num)?),
    }
}