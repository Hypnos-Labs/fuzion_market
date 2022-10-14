use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("To Do Error")]
    ToDo {},

    #[error("To Do Error")]
    ToDoTwo {x: String},

    #[error("No Tokens have been sent")]
    NoTokens {},

    #[error("Listing length must be at least 10 minutes")]
    NotLongEnough {},

    #[error("Listing ID already exists. Try another")]
    IdAlreadyExists {},

    #[error("Listing is expired")]
    Expired {},

    #[error("Funds sent in are not the required funds to purchase")]
    FundsSentNotFundsAsked {which: String},

    #[error("Tokens in ask are not in whitelist")]
    NotWhitelist {which: String},

    #[error("Listing not found")]
    NotFound {listing_id: String},

    #[error("Load bucket error")]
    LoadBucketError {},

    #[error("No Listing Found")]
    NoListing {},

    #[error("Error parsing from Utf8")]
    FromUtfError {},

    #[error("Error validt address- mybytes: {}, trash: {}, keep: {}, id_as_bytes: {}, keep_address_bytes: {}, listing_owner_string: {}", mybytes, trash, keep, id_as_bytes, keep_address_bytes, listing_owner_string)]
    AddrValidateError {mybytes: String, trash: String, keep: String, id_as_bytes: String, keep_address_bytes: String, listing_owner_string: String},

    #[error("Splitting Bytes Error")]
    SplitBytesError {},

    #[error("Strip Suffix Error")]
    StripSuffixError {},

}
