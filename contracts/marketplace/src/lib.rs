pub mod contract;
pub mod error;
pub use crate::error::ContractError;
pub mod execute;

#[cfg(test)]
pub mod integration_tests;

pub mod msg;
pub mod query;
pub mod state;
pub mod utils;

pub const MAX_NUM_ASSETS: u32 = 25u32;

mod contract_imports {
    pub use cosmwasm_std::{
        entry_point, from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo,
        Response, StdResult,
    };
    pub use cw2::set_contract_version;
    pub use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg, Cw20QueryMsg, TokenInfoResponse};
    pub use cw721::{Cw721ReceiveMsg, Cw721QueryMsg};

    pub use crate::error::ContractError;
    pub use crate::execute::{
        execute_add_to_bucket, execute_add_to_bucket_cw721, execute_add_to_listing,
        execute_add_to_listing_cw721, execute_buy_listing, execute_change_ask,
        execute_create_bucket, execute_create_bucket_cw721, execute_create_listing,
        execute_create_listing_cw721, execute_delete_listing,
        execute_withdraw_bucket, execute_withdraw_purchased,
    };
    pub use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, ReceiveNftMsg};
    pub use crate::query::*;
    pub use crate::state::{FeeDenom, Nft, FEE_DENOM, BUCKET_ID_USED, LISTING_ID_USED};
    pub use royalties::msg::InstantiateMsg as RoyaltyInstantiateMsg;
}

mod execute_imports {
    pub use crate::error::ContractError;
    pub use crate::msg::{CreateListingMsg, GenericBalanceUnvalidated};
    pub use crate::state::{
        genbal_cmp,
        listingz,
        BalanceUtil,
        Bucket,
        FeeDenom,
        GenericBalance,
        Listing,
        Nft,
        Status,
        BUCKETS,
        BUCKET_ID_USED,
        FEE_DENOM,
        LISTING_ID_USED, //BUCKET_COUNT, LISTING_COUNT
        ROYALTY_REGISTRY
    };
    pub use crate::utils::{calc_fee_coin, max, send_tokens_cosmos};
    pub use cosmwasm_std::{Addr, DepsMut, Env, Response, StdError, CosmosMsg};
    pub use cw20::Balance;
    pub use std::collections::BTreeSet;
    
    pub use royalties::{
        RoyaltyInfo,
        msg::QueryMsg as RoyaltyQueryMsg,
    };

    pub use super::MAX_NUM_ASSETS;
}

mod integration_tests_imports {
    pub use anyhow::ensure;
    pub use core::fmt::Display;

    pub use crate::{msg::*, state::*, utils::MAX_SAFE_INT};
    pub use cosmwasm_std::{coins, to_binary, Addr, Coin, Empty, Uint128};
    pub use cw20::{Cw20Coin, Cw20CoinVerified, Cw20Contract};
}

mod msg_imports {
    pub use std::collections::BTreeSet;
    pub use crate::query::*;
    pub use crate::state::GenericBalance;
    pub use cosmwasm_schema::{cw_serde, QueryResponses};
    pub use cw20::Cw20ReceiveMsg;
    pub use cw721::Cw721ReceiveMsg;
    pub use cosmwasm_std::{Uint128, Coin, Addr};
    pub use super::MAX_NUM_ASSETS;
}

mod query_imports {
    pub use crate::state::{
        listingz,
        Bucket,
        FeeDenom,
        Listing,
        Status,
        BUCKETS,
        FEE_DENOM, //LISTING_COUNT, BUCKET_COUNT
    };
    pub use cosmwasm_schema::cw_serde;
    pub use cosmwasm_std::{Addr, Deps, Env, Order, StdError, StdResult};
    pub use cw_storage_plus::PrefixBound;
}

mod state_imports {
    pub use std::collections::BTreeSet;
    pub use crate::error::ContractError;
    pub use crate::utils::send_tokens_cosmos;
    pub use cosmwasm_schema::cw_serde;
    pub use cosmwasm_std::{
        to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, StdResult, Timestamp, Uint128, WasmMsg,
    };
    pub use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg};
    pub use cw721::Cw721ExecuteMsg;
    pub use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};
    pub use std::collections::BTreeMap;
    pub use anybuf::Anybuf;
    pub use cosmwasm_std::{Empty, coin, StdError};
    pub use royalties::RoyaltyInfo;
    pub use super::MAX_NUM_ASSETS;
}

mod utils_imports {
    pub use crate::error::ContractError;
    pub use crate::state::{FeeDenom, GenericBalance, Listing};
    pub use cosmwasm_std::{
        coin, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, StdError,
        StdResult, WasmMsg,
    };
    pub use cw20::Cw20ExecuteMsg;
    pub use cw721::Cw721ExecuteMsg;
    pub use std::collections::BTreeMap;
}
