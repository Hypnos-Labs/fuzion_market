use cosmwasm_std::DepsMut;
use cw20::Cw20CoinVerified;

use crate::{msg_imports::*, ContractError, state::Nft};

#[cw_serde]
pub struct InstantiateMsg {
    pub royalty_code_id: u64
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Cycles the token denomination charged in fees (JUNO <> USDC)
    /// </br>
    /// This can be called by anyone, but can only be called once
    /// every 100,800 blocks
    /// </br> (approx. 1 week assuming 6 sec blocks)
    FeeCycle {},
    // Receive Filters
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    /// Create Listing with Native/cw20
    CreateListing {
        listing_id: u64,
        create_msg: CreateListingMsg,
    },
    /// Adding native/cw20 tokens to listing
    AddToListing {
        listing_id: u64,
    },
    /// Change ask of listing
    ChangeAsk {
        listing_id: u64,
        new_ask: GenericBalanceUnvalidated,
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
    CreateBucket {
        bucket_id: u64,
    },
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
        listing_id: u64,
        create_msg: CreateListingMsg,
    },
    AddToListingCw20 {
        listing_id: u64,
    },
    CreateBucketCw20 {
        bucket_id: u64,
    },
    AddToBucketCw20 {
        bucket_id: u64,
    },
}

#[cw_serde]
pub enum ReceiveNftMsg {
    CreateListingCw721 {
        listing_id: u64,
        create_msg: CreateListingMsg,
    },
    AddToListingCw721 {
        listing_id: u64,
    },
    CreateBucketCw721 {
        bucket_id: u64,
    },
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
        page_num: u8,
    },
    /// Gets Listings owned by user
    /// - Requires pagination
    #[returns(MultiListingResponse)]
    GetListingsByOwner {
        owner: String,
        page_num: u8,
    },
    /// Gets listings user is whitelisted for
    #[returns(MultiListingResponse)]
    GetListingsByWhitelist {
        owner: String,
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
    #[returns(Option<cosmwasm_std::Addr>)]
    GetRoyaltyAddr {}
}

/// Must be sent along with message when creating a Listing
#[cw_serde]
pub struct CreateListingMsg {
    pub ask: GenericBalanceUnvalidated,
    pub whitelisted_buyer: Option<String>,
}


#[cw_serde]
pub struct GenericBalanceUnvalidated {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinUnverified>,
    pub nfts: Vec<NftUnverified>,
}

impl GenericBalanceUnvalidated {
    /// Validate a `GenericBalanceUnvalidated` to a `GenericBalance`
    /// Errors if any are true:
    /// - Any Native or CW20 token amount is 0
    /// - `deps.api.addr_validate` errors for any cw20 or nft
    /// - Number of Natives, CW20's, and NFTs are over MAX_NUM_ASSETS
    /// - Any duplicate Native Denom, Cw20 contract_addr, or NFT (contract_addr + token_id)
    pub fn validate(self, deps: &DepsMut) -> Result<GenericBalance, ContractError> {

        // Check Natives for 0's
        if self.native.iter().any(|n| n.amount.is_zero()) {
            return Err(ContractError::GenericError("Cannot contain 0 value amounts".to_string()));
        }

        // Validate cw20 addresses and check for 0's
        let validated_cw20s: Vec<Cw20CoinVerified> = self.cw20
            .into_iter()
            .map(|unvalidated| {
                let addr = deps.api
                    .addr_validate(&unvalidated.address)
                    .map_err(|_e| ContractError::GenericError(format!("Invalid CW20 address: {}", unvalidated.address)))?;

                if unvalidated.amount.is_zero() {
                    return Err(ContractError::GenericError(format!("Invalid CW20 0 amount: {}", unvalidated.address)));
                }

                Ok(Cw20CoinVerified {
                    address: addr,
                    amount: unvalidated.amount
                })
            }).collect::<Result<Vec<Cw20CoinVerified>, ContractError>>()?;
        
        // Validate NFT contract addresses
        let validated_nfts: Vec<Nft> = self.nfts
            .into_iter()
            .map(|unvalidated| {
                let addr = deps.api
                    .addr_validate(&unvalidated.contract_address)
                    .map_err(|_e| ContractError::GenericError(format!("Invalid NFT address: {}", unvalidated.contract_address)))?;

                Ok(Nft {
                    contract_address: addr,
                    token_id: unvalidated.token_id
                })
            }).collect::<Result<Vec<Nft>, ContractError>>()?;


        // Validate number of assets (MAX_NUM_ASSETS is to avoid out of gas problems)
        let _ = self
            .native.len()
            .checked_add(validated_cw20s.len())
            .and_then(|v| v.checked_add(validated_nfts.len()))
            .ok_or_else(|| ContractError::GenericError(format!("Listing cannot contain over {} items", MAX_NUM_ASSETS)))
            .and_then(|v| {
                if v == 0 || v as u32 > MAX_NUM_ASSETS {
                    return Err(ContractError::GenericError(format!("Number of items must be between 1 and {}", MAX_NUM_ASSETS)));
                }
                Ok(())
            })?;
    
        // Check Natives for duplicates (same denom)
        let n_dd = self.native.iter().map(|n| n.denom.clone()).collect::<BTreeSet<String>>();
        if n_dd.len() != self.native.len() {
            return Err(ContractError::GenericError(
                "Cannot contain duplicate Native Tokens".to_string(),
            ));
        }

        // Check CW20's for duplicates (same address)
        // Do not use Addr since CHAIN1XYZ and chain1xyz are indeed different
        let cw_dd = validated_cw20s.iter().map(|cw| cw.address.clone().to_string()).collect::<BTreeSet<String>>();
        if cw_dd.len() != validated_cw20s.len() {
            return Err(ContractError::GenericError(
                "Cannot contain duplicate CW20 Tokens".to_string(),
            ));
        }

        // Check NFts for duplicates (same address & same token_id)
        let nft_dd = validated_nfts
            .iter()
            .map(|nft| (nft.contract_address.to_string(), nft.token_id.clone()))
            .collect::<BTreeSet<(String, String)>>();
        if nft_dd.len() != validated_nfts.len() {
            return Err(ContractError::GenericError("Cannot contain duplicate NFTs".to_string()));
        }

        Ok(GenericBalance {
            native: self.native,
            cw20: validated_cw20s,
            nfts: validated_nfts
        })
    }
}


// Use in messages to prevent unvalidated Addr instances
// from being saved directly to state
#[cw_serde]
pub struct Cw20CoinUnverified {
    pub address: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct NftUnverified {
    pub contract_address: String,
    pub token_id: String,
}


