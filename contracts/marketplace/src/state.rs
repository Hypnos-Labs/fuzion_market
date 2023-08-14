use crate::state_imports::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Core State
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Keeps track of previously used lisitng IDs
pub const LISTING_ID_USED: Map<u64, bool> = Map::new("listing_id_used");

/// Keeps track of previously used bucket IDs
pub const BUCKET_ID_USED: Map<u64, bool> = Map::new("bucket_id_used");

pub const FEE_DENOM: Item<FeeDenom> = Item::new("fee_denom");

pub const ROYALTY_REGISTRY: Item<Option<Addr>> = Item::new("royalty_regsitry");

#[cw_serde]
pub enum FeeDenom {
    JUNO(u64),
    USDC(u64),
}

impl FeeDenom {
    pub fn value(&self) -> String {
        match *self {
            FeeDenom::JUNO(_) => "ujunox".to_string(),
            //FeeDenom::JUNO => "ujuno".to_string(),
            FeeDenom::USDC(_) => "uusdcx".to_string(),
            //FeeDenom::USDC => "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034".to_string()
        }
    }

    // pub fn create(new: String, time: u64) -> Result<Self, ContractError> {
    //     match new.as_str() {
    //         "JUNO" => Ok(FeeDenom::JUNO(time)),
    //         "USDC" => Ok(FeeDenom::USDC(time)),
    //         x => Err(ContractError::GenericError(format!("Invalid Fee Denom: {x}"))),
    //     }
    // }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct ListingIndexes<'a> {
    pub id: UniqueIndex<'a, u64, Listing, (&'a Addr, u64)>,
    pub finalized_date: MultiIndex<'a, u64, Listing, (&'a Addr, u64)>,
    // (whitelisted_buyer/default, listing_id as u64)  |
    pub whitelisted_buyer: UniqueIndex<'a, (String, u64), Listing, (&'a Addr, u64)>,
}

impl IndexList<Listing> for ListingIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Listing>> + '_> {
        let v: Vec<&dyn Index<Listing>> =
            vec![&self.id, &self.finalized_date, &self.whitelisted_buyer];
        Box::new(v.into_iter())
    }
}

#[must_use]
pub fn listingz<'a>() -> IndexedMap<'a, (&'a Addr, u64), Listing, ListingIndexes<'a>> {
    let indexes = ListingIndexes {
        id: UniqueIndex::new(|a_listing| a_listing.id, "listing__id"),
        finalized_date: MultiIndex::new(
            |_pk, a_listing| a_listing.finalized_time.map_or(0_u64, |x| x.seconds()),
            "listings_im",
            "listing__finalized__date",
        ),
        whitelisted_buyer: UniqueIndex::new(
            |listing| {
                (
                    listing
                        .whitelisted_buyer
                        .clone()
                        .map_or_else(|| "1".to_string(), |addr| addr.to_string()),
                    listing.id,
                )
            },
            "listing__whitelisted__buyer",
        ),
    };

    IndexedMap::new("listings_im", indexes)
}

#[cw_serde]
pub struct Listing {
    pub creator: Addr,
    pub id: u64,

    pub finalized_time: Option<Timestamp>,
    pub expiration_time: Option<Timestamp>,
    pub status: Status,

    pub claimant: Option<Addr>,
    pub whitelisted_buyer: Option<Addr>,

    pub for_sale: GenericBalance,
    pub ask: GenericBalance,

    pub fee_amount: Option<Coin>,
}

impl Listing {
    /// **If `Listing.fee_amount.is_some()`**
    /// - Returns `Vec<CosmosMsg>` sending `Listing.fee_amount` to Com. Pool + `Listing.for_sale` to `Listing.claimant`
    ///
    /// **If `Listing.fee_amount.is_none()`**
    /// - Returns `Vec<CosmosMsg>` sending `Listing.for_sale` to `Listing.claimant`
    #[cfg(not(tarpaulin_include))]
    pub fn withdraw_msgs(&self, contract_addr: Addr) -> Result<Vec<CosmosMsg>, ContractError> {
        // Get claimant (This will not called when Listing does not have claimant)
        let user = self.claimant.as_ref().ok_or_else(|| {
            ContractError::GenericError("Listing has not been purchased".to_string())
        })?;

        match &self.fee_amount {
            // No fee amount, send for_sale to user
            None => send_tokens_cosmos(user, &self.for_sale).map_err(|_e| {
                ContractError::GenericError("Error creating withdraw messages".to_string())
            }),
            // Some fee amount, send fee to CP & for_sale to user
            Some(fee) => {
                let mut user_msgs = send_tokens_cosmos(user, &self.for_sale).map_err(|_e| {
                    ContractError::GenericError("Error creating withdraw messages".to_string())
                })?;
                let fee_msg = fee.get_cp_msg(contract_addr)?;
                user_msgs.push(fee_msg);
                Ok(user_msgs)
            }
        }
    }
}

#[cw_serde]
pub enum Status {
    BeingPrepared,
    FinalizedReady,
    Closed,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub const BUCKETS: Map<(Addr, u64), Bucket> = Map::new("buckets");

#[cw_serde]
pub struct Bucket {
    pub owner: Addr,
    pub funds: GenericBalance,
    pub fee_amount: Option<Coin>,
}

impl Bucket {
    /// **If `Bucket.fee_amount.is_some()`**
    /// - Returns `Vec<CosmosMsg>` sending `Bucket.fee_amount` to Com. Pool + `Bucket.funds` to `Bucket.owner`
    ///
    /// **If `Bucket.fee_amount.is_none()`**
    /// - Returns `Vec<CosmosMsg>` sending `Bucket.funds` to `Bucket.owner`
    #[cfg(not(tarpaulin_include))]
    pub fn withdraw_msgs(&self, contract_addr: Addr) -> Result<Vec<CosmosMsg>, ContractError> {
        match &self.fee_amount {
            None => send_tokens_cosmos(&self.owner, &self.funds).map_err(|_e| {
                ContractError::GenericError("Error creating withdraw messages".to_string())
            }),
            Some(fee) => {
                let mut user_msgs = send_tokens_cosmos(&self.owner, &self.funds).map_err(|_e| {
                    ContractError::GenericError("Error creating withdraw messages".to_string())
                })?;
                let fee_msg = fee.get_cp_msg(contract_addr)?;
                user_msgs.push(fee_msg);
                Ok(user_msgs)
            }
        }
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// GenericBalance
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// If we start to see these collections growing to 50+ items (only likely with NFTs)
// Then probably a good time to migrate to BTreeMap instead of Vec
#[cw_serde]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
    pub nfts: Vec<Nft>,
}

#[cw_serde]
pub struct Nft {
    pub contract_address: Addr,
    pub token_id: String,
}

impl GenericBalance {
    /// Generate messages for sending `Cw20Cw721ExecuteMsg::Send` variants
    /// This can be used if the withdrawing contracts wants to invoke some
    /// action on their contract when the cw20/cw721 messages are received
    /// **Currently unused, need to make a way for contracts to specify this
    /// when withdrawing**
    pub fn contract_msgs(&self, to: &Addr) -> StdResult<Vec<CosmosMsg>> {
        let mut msgs: Vec<CosmosMsg> = if self.native.is_empty() {
            vec![]
        } else {
            vec![CosmosMsg::from(BankMsg::Send {
                to_address: to.into(),
                amount: self.native.clone(),
            })]
        };

        let cw20_msgs: StdResult<Vec<_>> = self
            .cw20
            .iter()
            .map(|c| {
                // Contract must implement the cw20 receiver interface, replace x with contract specific message variant
                let msg = Cw20ExecuteMsg::Send {
                    contract: to.into(),
                    amount: c.amount,
                    msg: to_binary("x")?,
                };
                let exec = CosmosMsg::from(WasmMsg::Execute {
                    contract_addr: c.address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                Ok(exec)
            })
            .collect();

        msgs.extend(cw20_msgs?);

        let nft_msgs: StdResult<Vec<CosmosMsg<cosmwasm_std::Empty>>> = self
            .nfts
            .iter()
            .map(|n| {
                // Contract must implement the cw721 receiver interface, replace x with contract specific message variant
                let msg = Cw721ExecuteMsg::SendNft {
                    contract: to.into(),
                    token_id: n.token_id.clone(),
                    msg: to_binary("x")?,
                };
                let exec = CosmosMsg::from(WasmMsg::Execute {
                    contract_addr: n.contract_address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                Ok(exec)
            })
            .collect();

        msgs.extend(nft_msgs?);

        Ok(msgs)
    }

    pub fn wallet_msgs(&self, to: &Addr) -> StdResult<Vec<CosmosMsg>> {
        let mut msgs: Vec<CosmosMsg> = if self.native.is_empty() {
            vec![]
        } else {
            vec![CosmosMsg::from(BankMsg::Send {
                to_address: to.into(),
                amount: self.native.clone(),
            })]
        };

        let cw20_msgs: StdResult<Vec<_>> = self
            .cw20
            .iter()
            .map(|c| {
                // Will send to any type of address, but will NOT
                // execute any actions on "to" if it is a contract
                let msg = Cw20ExecuteMsg::Transfer {
                    recipient: to.into(),
                    amount: c.amount,
                };
                let exec = CosmosMsg::from(WasmMsg::Execute {
                    contract_addr: c.address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                Ok(exec)
            })
            .collect();
        msgs.extend(cw20_msgs?);

        let nft_msgs: StdResult<Vec<CosmosMsg<cosmwasm_std::Empty>>> = self
            .nfts
            .iter()
            .map(|n| {
                // Will send to any type of address, but will NOT
                // execute any actions on "to" if it is a contract
                let msg = Cw721ExecuteMsg::TransferNft {
                    recipient: to.into(),
                    token_id: n.token_id.clone(),
                };
                let exec = CosmosMsg::from(WasmMsg::Execute {
                    contract_addr: n.contract_address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                Ok(exec)
            })
            .collect();

        msgs.extend(nft_msgs?);

        Ok(msgs)
    }

    /// Construct a GenericBalance from a `cw20::Balance`
    pub fn from_balance(bal: &Balance) -> GenericBalance {
        match bal {
            Balance::Native(balance) => GenericBalance {
                native: balance.to_owned().into_vec(),
                cw20: vec![],
                nfts: vec![],
            },
            Balance::Cw20(token) => GenericBalance {
                native: vec![],
                cw20: vec![token.to_owned()],
                nfts: vec![],
            },
        }
    }

    /// Construct a GenericBalance from a single NFT
    pub fn from_nft(nft: Nft) -> GenericBalance {
        GenericBalance {
            native: vec![],
            cw20: vec![],
            nfts: vec![nft],
        }
    }

    /// Takes `add` as a `Balance(NativeBalance || Cw20CoinVerified)`
    ///
    /// If a token(s) in the `Balance` already exists in the **GenericBalance**,
    /// - The amount is added
    ///
    /// If a token(s) in the `Balance` does not already exist in the **GenericBalance**,
    /// - It is pushed onto the Vec
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }

    pub fn add_nft(&mut self, nft: Nft) {
        self.nfts.push(nft);
    }

    /// Errors if any are true:
    ///
    /// - Any Native token amount == 0
    /// - Any Cw20 token amount == 0
    /// - Any duplicate Native Denom
    /// - Any duplicate Cw20 Contract addresses
    /// - Number of Natives, CW20's, and NFTs are over 25
    pub fn check_valid(&self) -> Result<(), ContractError> {
        // Check that it isn't completely empty
        let length = self.native.len() + self.cw20.len() + self.nfts.len();
        if length == 0 {
            return Err(ContractError::GenericError("Cannot be empty".to_string()));
        }

        // Check that length is not over 35 to avoid out of gas issues
        if length > 25 {
            return Err(ContractError::GenericError("25 asset maximum exceeded".to_string()));
        }

        // Check Natives for 0's
        if self.native.iter().any(|n| n.amount.is_zero()) {
            return Err(ContractError::GenericError("Cannot contain 0 value amounts".to_string()));
        }

        // Check CW20's for 0's
        if self.cw20.iter().any(|c| c.is_empty()) {
            return Err(ContractError::GenericError("Cannot contain 0 value amounts".to_string()));
        }

        // Do not chain the following 2 checks together, as in theory a native denom could
        // be the same as a cw20 contract address while still being different tokens
        // Check Natives for duplicates (same denom)
        let n_bt =
            self.native.iter().map(|n| (n.denom.clone(), 1u8)).collect::<BTreeMap<String, u8>>();
        if n_bt.len() != self.native.len() {
            return Err(ContractError::GenericError(
                "Cannot contain duplicate Native Tokens".to_string(),
            ));
        }

        // Check CW20's for duplicates (same address)
        let cw_bt = self
            .cw20
            .iter()
            .map(|cw| (cw.address.to_string(), 1u8))
            .collect::<BTreeMap<String, u8>>();
        if cw_bt.len() != self.cw20.len() {
            return Err(ContractError::GenericError(
                "Cannot contain duplicate CW20 Tokens".to_string(),
            ));
        }

        // Check NFTs for duplicates (same address + same token_id)
        let nft_bt = self
            .nfts
            .iter()
            .map(|nft| ((nft.contract_address.to_string(), nft.token_id.clone()), 1u8))
            .collect::<BTreeMap<(String, String), u8>>();
        if nft_bt.len() != self.nfts.len() {
            return Err(ContractError::GenericError("Cannot contain duplicate NFTs".to_string()));
        }

        Ok(())
    }

    /// Get royalty messages for a GenericBalance and update balances
    /// 
    /// - Returns `Vec<CosmosMsg>` of Royalty Payments to be sent
    /// - Mutates GenericBalnce in place by subtracting all royalty payments
    pub fn royalties(&mut self, royalty_responses: Vec<Option<RoyaltyInfo>>) -> Result<(Vec<CosmosMsg>, u64), ContractError> {

        // - Sum the BPS of all royalties contained in the generic balance (1 = 0.01%)
        // - Remove collections without royalties
        let (sum_royalties, all_royalties): (u64, Vec<RoyaltyInfo>) = royalty_responses.into_iter()
            .filter_map(|r| r)
            .fold((0, vec![]), |(acc, mut vec), royalty| {
                (acc + royalty.bps, {
                    vec.push(royalty);
                    vec
                })
            });

        // 100 = 1%  |  5_000 = 50%
        // If royalties are greater than 50% (at 3% cap would require min. 17 different NFT collections), 
        // fail transaction & provide helpful error message. this seems like an acceptable solution for now
        if sum_royalties > 5000 {
            return Err(ContractError::GenericError("50% Royalty Max hit, try a Listing with fewer NFTs".to_string()));
        }

        let mut cosmos_msgs: Vec<CosmosMsg> = vec![];

        for native_balance in self.native.iter_mut() {

            let original_balance = native_balance.amount.clone();

            let mut new_balance = native_balance.amount.clone();

            for royalty in all_royalties.iter() {
                // Calculate royalty amount (1 bip = 0.01%)
                let amt_to_send = original_balance.checked_multiply_ratio(royalty.bps as u128, 10_000u128).unwrap_or_else(|_| Uint128::zero());

                if !amt_to_send.is_zero() {

                    // Create CosmosMsg to send tokens to Royalty Payout Addr
                    let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                        to_address: royalty.payout_addr.clone().to_string(),
                        amount: vec![coin(amt_to_send.u128(), &native_balance.denom)]
                    });

                    // Subtract the royalty amount from balance tracker
                    new_balance = new_balance.checked_sub(amt_to_send).map_err(|_e| ContractError::GenericError("Invalid Royalty amount | native".to_string()))?;

                    // Add the message 
                    cosmos_msgs.push(bank_msg);

                }

            }
        
            native_balance.amount = new_balance;
        
        }

        for cw20_balance in self.cw20.iter_mut() {

            let original_balance = cw20_balance.amount.clone();

            let mut new_balance = cw20_balance.amount.clone();

            for royalty in all_royalties.iter() {
                // Calculate royalty amt (1 bip = 0.01%)
                let amt_to_send = original_balance.checked_multiply_ratio(royalty.bps as u128, 10_000u128).unwrap_or_else(|_| Uint128::zero());

                if !amt_to_send.is_zero() {

                    let cw20_msg = Cw20ExecuteMsg::Transfer { 
                        recipient: royalty.payout_addr.to_string(), 
                        amount: amt_to_send
                    };
                    let exc_msg = CosmosMsg::from(WasmMsg::Execute {
                        contract_addr: cw20_balance.address.to_string(),
                        msg: to_binary(&cw20_msg)?,
                        funds: vec![]
                    });

                    // Subtract royalty amount from balance tracker
                    new_balance = new_balance.checked_sub(amt_to_send).map_err(|_e| ContractError::GenericError("Invalid Royalty amount | cw20".to_string()))?;

                    // push msg
                    cosmos_msgs.push(exc_msg);
                }
            
            

            }
        
            cw20_balance.amount = new_balance;
        }

        Ok((cosmos_msgs, sum_royalties))

    }
}



/// Accepts 2 x `&GenericBalance` and checks all fields for equality
/// - Fields do not need to be sorted
/// - Errors if they are not equal
pub fn genbal_cmp(one: &GenericBalance, two: &GenericBalance) -> Result<(), ContractError> {
    // Compare Natives
    // as long as there's nothing in one that two doesn't have
    if one.native.iter().any(|c| !two.native.contains(c)) || one.native.len() != two.native.len() {
        return Err(ContractError::GenericError("Native balances not equal".to_string()));
    }

    // Compare cw20s
    if one.cw20.iter().any(|cw| !two.cw20.contains(cw)) || one.cw20.len() != two.cw20.len() {
        return Err(ContractError::GenericError("Cw20 balances not equal".to_string()));
    }

    // Compare NFTs
    if one.nfts.iter().any(|nft| !two.nfts.contains(nft)) || one.nfts.len() != two.nfts.len() {
        return Err(ContractError::GenericError("NFTs not equal".to_string()));
    }

    Ok(())
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Cosmwasm types
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub trait BalanceUtil {
    /// Errors if `Balance::Native(__b__)` contains any 0 amounts or duplicate denoms
    ///
    /// Errors if `Balance::Cw20(__cw__)` has 0 amount
    fn normalized_check(&self) -> Result<(), ContractError>;
}

impl BalanceUtil for Balance {
    fn normalized_check(&self) -> Result<(), ContractError> {
        match self {
            Self::Native(balance) => {
                // Check for 0 length
                if balance.0.is_empty() {
                    return Err(ContractError::GenericError(
                        "Cannot contain 0 value amounts".to_string(),
                    ));
                }
                // Check for 0's
                if balance.0.iter().any(|n| n.amount.is_zero()) {
                    return Err(ContractError::GenericError(
                        "Cannot contain 0 value amounts".to_string(),
                    ));
                }

                // Check for duplicate denoms
                let n_bt = balance
                    .0
                    .iter()
                    .map(|n| (n.denom.clone(), 1u8))
                    .collect::<BTreeMap<String, u8>>();
                if n_bt.len() != balance.0.len() {
                    return Err(ContractError::GenericError(
                        "Cannot contain duplicate Native token denoms".to_string(),
                    ));
                }

                Ok(())
            }

            Self::Cw20(cw) => {
                if cw.is_empty() {
                    return Err(ContractError::GenericError(
                        "Cannot contain 0 value amounts".to_string(),
                    ));
                }

                Ok(())
            }
        }
    }
}

pub trait GetComPoolMsg {
    fn get_cp_msg(&self, contract_addr: Addr) -> Result<CosmosMsg, ContractError>;
}

impl GetComPoolMsg for Coin {
    fn get_cp_msg(&self, contract_addr: Addr) -> Result<CosmosMsg, ContractError> {
        // Can replace when cosmwasm_1_3 is live
        //Ok(CosmosMsg::Distribution(cosmwasm_std::DistributionMsg::FundCommunityPool { amount: vec![self.clone()] }))
        let coin = Anybuf::new()
            .append_string(1, self.denom.clone())
            .append_string(2, self.amount.to_string());
        let buf = Anybuf::new()
            .append_message(1, &coin)
            .append_string(2, contract_addr.as_str())
            .into_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
            value: buf.into(),
        };

        Ok(msg)

    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Tests
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod state_tests {

    use crate::state::*;
    use cosmwasm_std::{coin, Uint128};
    use std::fmt::Display;

    fn here(ctx: impl Display, line: impl Display, col: impl Display) -> String {
        format!(
            "~~~~~~~~~~~~~~~~~~~ \n \n {} \n line {} | column {} \n ________________________",
            ctx, line, col
        )
    }

    fn cw20(addr: impl Into<String>, amt: u128) -> Cw20CoinVerified {
        Cw20CoinVerified {
            address: Addr::unchecked(addr),
            amount: Uint128::from(amt),
        }
    }

    fn nft(addr: impl Into<String>, id: impl Into<String>) -> Nft {
        Nft {
            contract_address: Addr::unchecked(addr),
            token_id: id.into(),
        }
    }

    #[test]
    fn genericbalance_compare() {
        let natives = vec![coin(100, "JUNO"), coin(200, "ATOM"), coin(300, "OSMO")];
        let cw20s = vec![cw20("foo", 1), cw20("bar", 2), cw20("baz", 3)];
        let nfts = vec![nft("boredcats", "30"), nft("dogs", "31"), nft("sharks", "32")];
        let gen_bal_main = GenericBalance {
            native: natives,
            cw20: cw20s,
            nfts,
        };

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Switch Order, should still be equal
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_x = vec![coin(200, "ATOM"), coin(100, "JUNO"), coin(300, "OSMO")];
        let cw20s_x = vec![cw20("bar", 2), cw20("foo", 1), cw20("baz", 3)];
        let nfts_x = vec![nft("dogs", "31"), nft("boredcats", "30"), nft("sharks", "32")];
        let gen_bal_x = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };

        genbal_cmp(&gen_bal_main, &gen_bal_x).unwrap_or_else(|_| {
            panic!("{}", here("Reordered should be equal", line!(), column!(),))
        });

        //let _resx = fake(&gen_bal_main, &gen_bal_x).expect(&here("Reordered should be equal", line!(), column!()));

        // All of the following should not be equal

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different native denom
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_y = vec![coin(200, "ATOm"), coin(100, "JUNO"), coin(300, "OSMO")];
        let gen_bal_y = GenericBalance {
            native: natives_y,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res = genbal_cmp(&gen_bal_main, &gen_bal_y).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different native amount (too big)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_yy = vec![coin(201, "ATOM"), coin(100, "JUNO"), coin(300, "OSMO")];
        let gen_bal_yy = GenericBalance {
            native: natives_yy,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_yy).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different native amount (too small)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_z = vec![coin(199, "ATOM"), coin(100, "JUNO"), coin(300, "OSMO")];
        let gen_bal_z = GenericBalance {
            native: natives_z,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res = genbal_cmp(&gen_bal_main, &gen_bal_z).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // An extra native coin (duplicate, shouldn't ever occur anyway)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_zz =
            vec![coin(200, "ATOM"), coin(100, "JUNO"), coin(200, "ATOM"), coin(300, "OSMO")];
        let gen_bal_zz = GenericBalance {
            native: natives_zz,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_zz).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A extra native coin (different)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_df =
            vec![coin(200, "ATOM"), coin(100, "JUNO"), coin(200, "DOGE"), coin(300, "OSMO")];
        let gen_bal_df = GenericBalance {
            native: natives_df,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_df).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // 1 less native denoms
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let natives_l = vec![coin(200, "ATOM"), coin(300, "OSMO")];
        let gen_bal_l = GenericBalance {
            native: natives_l,
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res = genbal_cmp(&gen_bal_main, &gen_bal_l).expect_err(&here("y", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Empty natives
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let gen_bal_e = GenericBalance {
            native: Vec::with_capacity(3),
            cw20: cw20s_x.clone(),
            nfts: nfts_x.clone(),
        };
        let _res = genbal_cmp(&gen_bal_main, &gen_bal_e).expect_err(&here("y", line!(), column!()));

        //~~~~~~~~~~
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different cw20 addr
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_xx = vec![cw20("bad", 2), cw20("foo", 1), cw20("baz", 3)];
        let gen_bal_xx = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_xx,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_xx).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different cw20 amount (too much)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_xxx = vec![cw20("bar", 3), cw20("foo", 1), cw20("baz", 3)];
        let gen_bal_xxx = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_xxx,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_xxx).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different cw20 amount (too little)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_a = vec![cw20("bar", 1), cw20("foo", 1), cw20("baz", 3)];
        let gen_bal_a = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_a,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_a).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // An extra cw20 (duplicate, shouldn't ever occur anyway)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_o = vec![cw20("bar", 2), cw20("foo", 1), cw20("bar", 2), cw20("baz", 3)];
        let gen_bal_o = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_o,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_o).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A extra cw20 (different addr)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_oo = vec![cw20("bar", 2), cw20("foo", 1), cw20("pip", 2), cw20("baz", 3)];
        let gen_bal_oo = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_oo,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_oo).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // 1 less cw20
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let cw20s_ooo = vec![cw20("bar", 2), cw20("foo", 1)];
        let gen_bal_ooo = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_ooo,
            nfts: nfts_x.clone(),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_ooo).expect_err(&here("cw", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Empty cw20
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let gen_bal_et = GenericBalance {
            native: natives_x.clone(),
            cw20: Vec::with_capacity(3),
            nfts: nfts_x,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_et).expect_err(&here("cw", line!(), column!()));

        //~~~~~~~~~~~~~
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different nft addr
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let nfts_m = vec![nft("mice", "31"), nft("boredcats", "30"), nft("sharks", "32")];
        let gen_bal_m = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_m,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_m).expect_err(&here("nft", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // A different nft token id
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let nfts_mm = vec![nft("dogs", "31"), nft("boredcats", "29"), nft("sharks", "32")];
        let gen_bal_mm = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_mm,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_mm).expect_err(&here("nft", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // One extra (duplicate)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let nfts_h =
            vec![nft("dogs", "31"), nft("boredcats", "30"), nft("dogs", "31"), nft("sharks", "32")];
        let gen_bal_h = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_h,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_h).expect_err(&here("nft", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // One extra (different)
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let nfts_hh =
            vec![nft("dogs", "31"), nft("boredcats", "30"), nft("dogs", "35"), nft("sharks", "32")];
        let gen_bal_hh = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_hh,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_hh).expect_err(&here("nft", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // One less
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let nfts_lv = vec![nft("dogs", "31"), nft("sharks", "32")];
        let gen_bal_lv = GenericBalance {
            native: natives_x.clone(),
            cw20: cw20s_x.clone(),
            nfts: nfts_lv,
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_lv).expect_err(&here("nft", line!(), column!()));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Empty
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let gen_bal_mph = GenericBalance {
            native: natives_x,
            cw20: cw20s_x,
            nfts: Vec::with_capacity(3),
        };
        let _res =
            genbal_cmp(&gen_bal_main, &gen_bal_mph).expect_err(&here("nft", line!(), column!()));
    }
}
