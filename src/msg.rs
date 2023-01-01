#[allow(unused_imports)]
use crate::query::{
    AdminResponse, ConfigResponse, GetBucketsResponse, ListingInfoResponse, MultiListingResponse,
};
use crate::state::GenericBalance;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Execute
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub enum ExecuteMsg {
    // Receive Filters
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    // Create Listing
    CreateListing {
        create_msg: CreateListingMsg,
    },
    // Edit Listing
    AddFundsToSaleNative {
        listing_id: String,
    },
    ChangeAsk {
        listing_id: String,
        new_ask: GenericBalance,
    },
    RemoveListing {
        listing_id: String,
    },
    // Makes Listing available for purchase & sets expiration time
    Finalize {
        listing_id: String,
        seconds: u64,
    },
    // Only callable when Listing is expired
    RefundExpired {
        listing_id: String,
    },
    CreateBucket {
        bucket_id: String,
    },
    AddToBucket {
        bucket_id: String,
    },
    RemoveBucket {
        bucket_id: String,
    },
    BuyListing {
        listing_id: String,
        bucket_id: String,
    },
    WithdrawPurchased {
        listing_id: String,
    },
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// cw20 entry point
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub enum ReceiveMsg {
    CreateListingCw20 {
        create_msg: CreateListingMsg,
    },
    AddFundsToSaleCw20 {
        listing_id: String,
    },
    CreateBucketCw20 {
        bucket_id: String,
    },
    AddToBucketCw20 {
        bucket_id: String,
    },
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// cw721 entry point
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub enum ReceiveNftMsg {
    CreateListingCw721 {
        create_msg: CreateListingMsg,
    },
    AddToListingCw721 {
        listing_id: String,
    },
    CreateBucketCw721 {
        bucket_id: String,
    },
    AddToBucketCw721 {
        bucket_id: String,
    },
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    GetAdmin {},
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(MultiListingResponse)]
    GetAllListings {},
    #[returns(ListingInfoResponse)]
    GetListingInfo {
        listing_id: String,
    },
    #[returns(MultiListingResponse)]
    GetListingsByOwner {
        owner: String,
    },
    #[returns(GetBucketsResponse)]
    GetBuckets {
        bucket_owner: String,
    },
    #[returns(MultiListingResponse)]
    GetListingsForMarket {
        page_num: u8,
    },
}

#[cw_serde]
pub struct CreateListingMsg {
    pub id: String,
    pub ask: GenericBalance,
    pub whitelisted_buyer: Option<String>,
}
