use crate::msg_imports::*;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Cycles the token denomination charged in fees (JUNO <> USDC)
    /// </br>
    /// This can be called by anyone, but can only be called once
    /// every 100,800 blocks
    /// </br> (approx. 1 week assuming 6 sec blocks)
    FeeCycle,
    // Receive Filters
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    /// Create Listing with Native/cw20
    CreateListing {
        create_msg: CreateListingMsg,
    },
    /// Adding native/cw20 tokens to listing
    AddToListing {
        listing_id: u64,
    },
    /// Change ask of listing
    ChangeAsk {
        listing_id: u64,
        new_ask: GenericBalance,
    },
    /// Makes Listing available for purchase & sets expiration time
    Finalize {
        listing_id: u64,
        seconds: u64,
    },
    /// Callable if listing has not been finalized
    /// or is expired
    DeleteListing {
        listing_id: u64,
    },
    /// Create Bucket with native/cw20
    CreateBucket {},
    /// Add Native/cw20 to bucket
    AddToBucket {
        bucket_id: u64,
    },
    /// Withdraw bucket
    RemoveBucket {
        bucket_id: u64,
    },
    /// Buy listing
    BuyListing {
        listing_id: u64,
        bucket_id: u64,
    },
    /// Withdraw purchased listing
    WithdrawPurchased {
        listing_id: u64,
    },
}

#[cw_serde]
pub enum ReceiveMsg {
    CreateListingCw20 {
        create_msg: CreateListingMsg,
    },
    AddToListingCw20 {
        listing_id: u64,
    },
    CreateBucketCw20 {},
    AddToBucketCw20 {
        bucket_id: u64,
    },
}

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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the current Fee Denom
    #[returns(FeeDenomResponse)]
    GetFeeDenom {},
    /// Gets Buckets owned by user
    /// - Requires pagination
    #[returns(MultiBucketResponse)]
    GetBuckets {
        bucket_owner: String,
        page_num: u8
    },
    /// Gets Listings owned by user
    /// - Requires pagination
    #[returns(MultiListingResponse)]
    GetListingsByOwner {
        owner: String,
        page_num: u8
    },
    /// Gets listings user is whitelisted for
    #[returns(MultiListingResponse)]
    GetListingsByWhitelist {
        owner: String
    },
    /// Gets listings for Marketplace
    /// - Does not return non-finalized, expired, or already sold listings
    /// - Requires pagination
    #[returns(MultiListingResponse)]
    GetListingsForMarket {
        page_num: u8,
    },
    // #[returns(ListingInfoResponse)]
    // GetListingInfo {
    //     listing_id: u64,
    // },
}


/// Must be sent along with message when creating a Listing
#[cw_serde]
pub struct CreateListingMsg {
    pub ask: GenericBalance,
    pub whitelisted_buyer: Option<String>,
}
