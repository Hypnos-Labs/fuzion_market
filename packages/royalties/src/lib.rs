pub use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
pub mod msg;

#[cw_serde]
pub struct RoyaltyInfo {
    pub last_updated: u64,
    pub bps: u64,
    pub payout_addr: Addr,
}