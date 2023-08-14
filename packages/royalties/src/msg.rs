use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::{
    //DenomType, 
    RoyaltyInfo
};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Register {
        nft_contract: String,
        payout_addr: String,
        bps: u64,
    },
    Update {
        nft_contract: String,
        new_payout_addr: Option<String>,
        new_bps: Option<u64>,
    },
    Remove {
        nft_contract: String
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<RoyaltyInfo>)]
    RoyaltyInfoSingle {
        nft_contract: String
    },
    #[returns(Vec<Option<RoyaltyInfo>>)]
    RoyaltyInfoMulti {
        nft_contracts: Vec<String>
    }
}