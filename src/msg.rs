use crate::msg_imports::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub struct InstantiateMsg {}

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
    AddToListing {
        listing_id: u64,
    },
    ChangeAsk {
        listing_id: u64,
        new_ask: GenericBalance,
    },
    // Makes Listing available for purchase & sets expiration time
    Finalize {
        listing_id: u64,
        seconds: u64,
    },
    DeleteListing {
        listing_id: u64,
    },
    CreateBucket {
        //bucket_id: String,
    },
    AddToBucket {
        bucket_id: u64,
    },
    RemoveBucket {
        bucket_id: u64,
    },
    BuyListing {
        listing_id: u64,
        bucket_id: u64,
    },
    WithdrawPurchased {
        listing_id: u64,
    },
    // RemoveListing {
    //     listing_id: u64,
    // },
    // // Only callable when Listing is expired
    // RefundExpired {
    //     listing_id: u64,
    // },
}

// cw20 entry point
#[cw_serde]
pub enum ReceiveMsg {
    CreateListingCw20 {
        create_msg: CreateListingMsg,
    },
    // AddFundsToSaleCw20 {
    //     listing_id: u64,
    // },
    AddToListingCw20 {
        listing_id: u64,
    },
    CreateBucketCw20 {
        //bucket_id: String,
    },
    AddToBucketCw20 {
        bucket_id: u64,
    },
}

// cw721 entry point
#[cw_serde]
pub enum ReceiveNftMsg {
    CreateListingCw721 {
        create_msg: CreateListingMsg,
    },
    AddToListingCw721 {
        listing_id: u64,
    },
    CreateBucketCw721 {},
    AddToBucketCw721 {
        bucket_id: u64,
    },
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CountResponse)]
    GetCounts {},
    #[returns(FeeDenomResponse)]
    GetFeeDenom {},
    #[returns(MultiListingResponse)]
    GetAllListings {},
    // #[returns(ListingInfoResponse)]
    // GetListingInfo {
    //     listing_id: u64,
    // },
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

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query Helpers
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cw_serde]
pub struct CreateListingMsg {
    //pub id: u64,
    pub ask: GenericBalance,
    pub whitelisted_buyer: Option<String>,
}
