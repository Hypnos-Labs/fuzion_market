use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Error Message: {0}")]
    GenericError(String),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not whitelisted to purchase")]
    NotWhitelisted {},

    #[error("One or more invalid address formats")]
    InvalidAddressFormat,

    #[error("To Do Error")]
    ToDo {},

    #[error("Error Adding: {0}")]
    ErrorAdding(String),

    #[error("No Tokens have been sent")]
    NoTokens {},

    #[error("Listing already finalized")]
    AlreadyFinalized {},

    #[error("ID already taken")]
    IdAlreadyExists {},

    #[error("Listing is expired")]
    Expired {},

    #[error("Funds sent in are not the required funds to purchase")]
    FundsSentNotFundsAsked {
        which: String,
    },

    #[error("Tokens in ask are not in whitelist")]
    NotWhitelist {
        which: String,
    },

    #[error("{typ} {id} not found")]
    NotFound {
        typ: String,
        id: String,
    },

    #[error("Load bucket error")]
    LoadBucketError {},

    #[error("Invalid Expiration")]
    InvalidExpiration {},

    #[error("Listing not expired | Expiration: {x}")]
    NotExpired {
        x: String,
    },

    #[error("Listing not purchasable")]
    NotPurchasable {},

    #[error("Missing Instantiate Option {0}")]
    MissingInit(String),

    #[error("Invalid address passed in Instantiate Message")]
    InitInvalidAddr,

    #[error("Generic Invalid")]
    GenericInvalid,

    #[error("Fee calculation error")]
    FeeCalc,
}
