use crate::error::*;
use crate::state::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use cosmwasm_schema::cw_serde;

use cosmwasm_std::coins;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, CosmosMsg, DepsMut, Empty, StdError, StdResult, WasmMsg,
};
use cw20::{Balance, Cw20ExecuteMsg};
use cw721::Cw721ExecuteMsg;

// Actual community pool on mainnet
const COMMUNITY_POOL: &str = "juno1jv65s3grqf6v6jl3dp4t6c9t9rk99cd83d88wr";
// use fake contract address for testnet
//const COMMUNITY_POOL: &str = ""

pub fn send_tokens_cosmos(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![CosmosMsg::from(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.clone(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            // Only works if recipient is User Address, doesn't work for DAO / Contracts
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

    let nft_balance = &balance.nfts;
    let nft_msgs: StdResult<Vec<CosmosMsg<Empty>>> = nft_balance
        .iter()
        .map(|n| {
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

pub fn calc_fee(balance: &GenericBalance) -> StdResult<Option<(CosmosMsg, GenericBalance)>> {
    let juno_in_balance = balance.native.iter().find(|n| n.denom == *"ujunox");

    // If balance DOES NOT contain juno, return Ok(None)
    // If balance DOES contain juno, calculate 0.1% of the JUNO in the balance,
    // Create CosmosMsg sending that to the Community Pool,
    // and return this CosmosMsg + a generic balance with the fee removed for the user
    if let Some(juno) = juno_in_balance {
        // 0.1% = amount * 1 / 1000
        let ten_pips = juno.amount.multiply_ratio(1_u128, 1000_u128);

        let fee_msg: CosmosMsg<Empty> = CosmosMsg::from(BankMsg::Send {
            to_address: COMMUNITY_POOL.to_string(),
            amount: coins(ten_pips.u128(), "ujunox"),
        });

        let juno_amount_after_fee_removed = juno.amount.checked_sub(ten_pips)?;

        let balance_with_fee_removed = {
            let mut x = balance.clone();
            x.native.retain(|n| n.denom != *"ujunox");
            x.native.append(&mut coins(juno_amount_after_fee_removed.u128(), "ujunox"));
            x
        };

        Ok(Some((fee_msg, balance_with_fee_removed)))
    } else {
        Ok(None)
    }
}

pub fn is_balance_whitelisted(
    balance: &Balance, 
    deps: &DepsMut
) -> Result<(), ContractError> {

    match balance {
        Balance::Native(natives) => {

            let _valid = natives
                .0
                .iter()
                .map(|native| -> Result<(), ContractError> {
                    if !WHITELIST_NATIVE.has(deps.storage, native.denom.clone()) {
                        Err(ContractError::NotWhitelisted {  })
                    } else {
                        Ok(())
                    }
                })
                .collect::<Result<(), ContractError>>()?;

            Ok(())

        },

        Balance::Cw20(cw20) => {

            if !WHITELIST_CW20.has(deps.storage, cw20.address.clone()) {
                Err(ContractError::NotWhitelisted {  })
            } else {
                Ok(())
            }
        }
    }

}

pub fn is_genericbalance_whitelisted(
    genericbalance: &GenericBalance,
    //config: &Config,
    deps: &DepsMut
) -> Result<(), ContractError> {


    // Check for Natives
    for native in &genericbalance.native {
        if !WHITELIST_NATIVE.has(deps.storage, native.denom.clone()) {
            return Err(ContractError::NotWhitelist { which: "Native".to_string() });
        }
    }

    // Check for cw20s
    for cw20 in &genericbalance.cw20 {
        if !WHITELIST_CW20.has(deps.storage, cw20.address.clone()) {
            return Err(ContractError::NotWhitelist {which: "Cw20".to_string()});
        }
    }

    // Check for NFTs
    for nft in &genericbalance.nfts {
        if !WHITELIST_NFT.has(deps.storage, nft.contract_address.clone()) {
            return Err(ContractError::NotWhitelist {which: "NFT".to_string()});
        }
    }

    Ok(())

}

pub fn is_nft_whitelisted(nft_addr: &Addr, deps: &DepsMut) -> Result<(), ContractError> {

    if !WHITELIST_NFT.has(deps.storage, nft_addr.to_owned()) {
        return Err(ContractError::NotWhitelisted {  });
    };

    Ok(())

}

/// Get allowed purchasers for a given listing.
/// If any address string is not valid, returns an error
pub fn get_whitelisted_addresses(
    deps: &DepsMut,
    whitelisted_addrs: Option<Vec<String>>,
) -> Result<Option<Vec<Addr>>, ContractError> {
    let Some(addrs) = whitelisted_addrs else {
        return Ok(None);
    };

    if addrs.is_empty() {
        return Ok(None);
    };

    let valid: Vec<Addr> = addrs
        .iter()
        .map(|address| {
            deps.api.addr_validate(&address).map_err(|_| ContractError::InvalidAddressFormat)
        })
        .collect::<Result<Vec<Addr>, ContractError>>()?;

    Ok(Some(valid))
}

#[cw_serde]
pub struct EzTimeStruct {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

pub trait EzTime {
    fn eztime_struct(&self) -> StdResult<EzTimeStruct>;
    fn eztime_string(&self) -> StdResult<String>;
}

impl EzTime for cosmwasm_std::Timestamp {
    fn eztime_struct(&self) -> StdResult<EzTimeStruct> {
        let seconds = &self.seconds();
        let nano = &self.subsec_nanos();

        let Some(dt) = NaiveDateTime::from_timestamp_opt(*seconds as i64, *nano as u32) else {
            return Err(StdError::GenericErr { msg: "Invalid Timestamp".to_string() });
        };

        Ok(EzTimeStruct {
            year: dt.year() as u32,
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
            minute: dt.minute(),
            second: dt.second(),
        })
    }

    fn eztime_string(&self) -> StdResult<String> {
        let seconds = &self.seconds();
        let nano = &self.subsec_nanos();

        let Some(dt) = NaiveDateTime::from_timestamp_opt(*seconds as i64, *nano as u32) else {
            return Err(StdError::GenericErr { msg: "Invalid Timestamp".to_string() });
        };

        match dt.month() {
            1 => Ok(format!(
                "January {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            2 => Ok(format!(
                "February {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            3 => Ok(format!(
                "March {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            4 => Ok(format!(
                "April {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            5 => Ok(format!(
                "May {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            6 => Ok(format!(
                "June {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            7 => Ok(format!(
                "July {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            8 => Ok(format!(
                "August {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            9 => Ok(format!(
                "September {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            10 => Ok(format!(
                "October {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            11 => Ok(format!(
                "November {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            12 => Ok(format!(
                "December {}, {} | {}:{}:{} UTC",
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )),
            _ => Err(StdError::GenericErr {
                msg: "Invalid Timestamp".to_string(),
            }),
        }
    }
}
