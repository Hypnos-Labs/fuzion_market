use crate::error::*;
use crate::state::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, CosmosMsg, Empty, StdError, StdResult, Timestamp, WasmMsg,
};
use cw20::{Balance, Cw20ExecuteMsg};
use cw721::Cw721ExecuteMsg;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Create send tokens messages
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Needs update to enable sending to a contract or DAO address
pub fn send_tokens_cosmos(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![CosmosMsg::from(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
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

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Whitelist checks
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Works for both Native and CW20 tokens
pub fn is_balance_whitelisted(balance: &Balance, config: &Config) -> Result<(), ContractError> {
    // config.whitelist_native contains (String-Symbol, String-Denom)
    // ex: (JUNO, ujunox) or (ATOM, ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9)

    // config.whitelist_cw20 contains (String-Symbol, Addr-Token Address)
    // ex: (NETA, juno168ctmpyppk90d34p3jjy658zf5a5l3w8wk35wht6ccqj4mr0yv8s4j5awr)

    let wl_native_denoms: Vec<_> = config
        .whitelist_native
        .iter()
        .map(|double| double.1.clone())
        .collect();

    let wl_cw20_addys: Vec<_> = config
        .whitelist_cw20
        .iter()
        .map(|double2| double2.1.clone())
        .collect();

    match balance {
        Balance::Native(natives_sent_in) => {
            let bool_vec: Vec<bool> = natives_sent_in
                .0
                .iter()
                .map(|native| {
                    if wl_native_denoms.contains(&native.denom) {
                        true
                    } else {
                        false
                    }
                })
                .collect();
            // If balance contains any denom that's not on the whitelist, return error
            if bool_vec.contains(&false) {
                return Err(ContractError::NotWhitelist {
                    which: "fail 1 Native".to_string(),
                });
            }
        }
        Balance::Cw20(cw20) => {
            if !wl_cw20_addys.contains(&cw20.address) {
                return Err(ContractError::NotWhitelist {
                    which: "fail 2 Cw20".to_string(),
                });
            }
        }
    }

    Ok(())
}

pub fn is_genericbalance_whitelisted(
    genericbalance: &GenericBalance,
    config: &Config,
) -> Result<(), ContractError> {
    let wl_native_denoms: Vec<_> = config
        .whitelist_native
        .iter()
        .map(|double| double.1.clone())
        .collect();

    if genericbalance.native.len() > 0 as usize {
        for native in genericbalance.native.clone() {
            if !wl_native_denoms.contains(&native.denom) {
                return Err(ContractError::NotWhitelist {
                    which: native.denom,
                });
            };
        }
    }

    let wl_cw20_addys: Vec<_> = config
        .whitelist_cw20
        .iter()
        .map(|double2| double2.1.clone())
        .collect();

    if genericbalance.cw20.len() > 0 as usize {
        for cw20coin in genericbalance.cw20.clone() {
            if !wl_cw20_addys.contains(&cw20coin.address) {
                return Err(ContractError::NotWhitelist {
                    which: cw20coin.address.into_string(),
                });
            };
        }
    }

    let wl_nft_addys: Vec<_> = config
        .whitelist_nft
        .iter()
        .map(|double3| double3.1.clone())
        .collect();

    if genericbalance.nfts.len() > 0 as usize {
        for nft in genericbalance.nfts.clone() {
            if !wl_nft_addys.contains(&nft.contract_address) {
                return Err(ContractError::NotWhitelist {
                    which: nft.contract_address.into_string(),
                });
            };
        }
    }

    Ok(())
}

pub fn is_nft_whitelisted(nft_addr: &Addr, config: &Config) -> Result<(), ContractError> {
    let wl_nfts: Vec<_> = config
        .whitelist_nft
        .iter()
        .map(|double| double.1.clone())
        .collect();

    if !wl_nfts.contains(nft_addr) {
        return Err(ContractError::NotWhitelist {
            which: nft_addr.to_string(),
        });
    };

    Ok(())
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// EzTime
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
        let dt = NaiveDateTime::from_timestamp(*seconds as i64, *nano as u32);
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
        let dt = NaiveDateTime::from_timestamp(*seconds as i64, *nano as u32);
        match dt.month() {
            1 => {
                return Ok(format!(
                    "January {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            2 => {
                return Ok(format!(
                    "February {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            3 => {
                return Ok(format!(
                    "March {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            4 => {
                return Ok(format!(
                    "April {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            5 => {
                return Ok(format!(
                    "May {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            6 => {
                return Ok(format!(
                    "June {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            7 => {
                return Ok(format!(
                    "July {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            8 => {
                return Ok(format!(
                    "August {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            9 => {
                return Ok(format!(
                    "September {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            10 => {
                return Ok(format!(
                    "October {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            11 => {
                return Ok(format!(
                    "November {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            12 => {
                return Ok(format!(
                    "December {}, {} | {}:{}:{} UTC",
                    dt.day(),
                    dt.year(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                ));
            }
            _ => {
                return Err(StdError::GenericErr {
                    msg: "Invalid Timestamp".to_string(),
                });
            }
        };
    }
}
