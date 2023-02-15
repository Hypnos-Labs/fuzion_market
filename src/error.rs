use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Error Message: {0}")]
    Std(#[from] StdError),

    #[error("Error Message: {0}")]
    GenericError(String),

    #[error("Error Message: Unauthorized")]
    Unauthorized {},

    #[error("Error Message: Not whitelisted to purchase")]
    NotWhitelisted {},

    #[error("Error Message: One or more invalid address formats")]
    InvalidAddressFormat,

    #[error("Error Message: To Do Error")]
    ToDo {},

    #[error("Error Message: Error Adding: {0}")]
    ErrorAdding(String),

    #[error("Error Message: No Tokens have been sent")]
    NoTokens {},

    #[error("Error Message: Listing already finalized")]
    AlreadyFinalized {},

    #[error("Error Message: ID already taken")]
    IdAlreadyExists {},

    #[error("Error Message: Listing is expired")]
    Expired {},

    #[error("Error Message: Funds sent in are not the required funds to purchase")]
    FundsSentNotFundsAsked {
        which: String,
    },

    #[error("Error Message: Tokens in ask are not in whitelist")]
    NotWhitelist {
        which: String,
    },

    #[error("Error Message: {typ} {id} not found")]
    NotFound {
        typ: String,
        id: String,
    },

    #[error("Error Message: Load bucket error")]
    LoadBucketError {},

    #[error("Error Message: Invalid Expiration")]
    InvalidExpiration {},

    #[error("Error Message: Listing not expired | Expiration: {x}")]
    NotExpired {
        x: String,
    },

    #[error("Error Message: Listing not purchasable")]
    NotPurchasable {},

    #[error("Error Message: Missing Instantiate Option {0}")]
    MissingInit(String),

    #[error("Error Message: Invalid address passed in Instantiate Message")]
    InitInvalidAddr,

    #[error("Error Message: Generic Invalid")]
    GenericInvalid,

    #[error("Error Message: Fee calculation error")]
    FeeCalc,
}
