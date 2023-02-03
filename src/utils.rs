use crate::error::ContractError;
use crate::state::GenericBalance;

use cosmwasm_std::{coins, Binary};
use cosmwasm_std::{to_binary, Addr, BankMsg, CosmosMsg, Empty, StdResult, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;

include!("protos/mod.rs");
use coin::Coin as SDKCoin;
use protobuf::Message;
use CosmosDistrubtionV1beta1MsgFundCommunityPool::MsgFundCommunityPool;

// Actual community pool on mainnet
// const COMMUNITY_POOL: &str = "juno1jv65s3grqf6v6jl3dp4t6c9t9rk99cd83d88wr";
// use fake contract address for testnet
//const COMMUNITY_POOL: &str = ""

const NATIVE: &str = "ujunox";

pub fn send_tokens_cosmos(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = Vec::new();

    if !balance.native.is_empty() {
        msgs.push(CosmosMsg::from(BankMsg::Send {
            to_address: to.into(),
            amount: balance.native.clone(),
        }));
    }

    let cw20_msgs: StdResult<Vec<_>> = balance
        .cw20
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = CosmosMsg::from(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: Vec::new(),
            });
            Ok(exec)
        })
        .collect();
    msgs.extend(cw20_msgs?);

    let nft_msgs: StdResult<Vec<CosmosMsg<Empty>>> = balance
        .nfts
        .iter()
        .map(|n| {
            let msg = Cw721ExecuteMsg::TransferNft {
                recipient: to.into(),
                token_id: n.token_id.clone(),
            };
            let exec = CosmosMsg::from(WasmMsg::Execute {
                contract_addr: n.contract_address.to_string(),
                msg: to_binary(&msg)?,
                funds: Vec::new(),
            });
            Ok(exec)
        })
        .collect();
    msgs.extend(nft_msgs?);

    Ok(msgs)
}

// Validate Ask
// Removes any 0 values, returns error if duplicate is found
pub fn normalize_ask_error_on_dup(ask: GenericBalance) -> Result<GenericBalance, ContractError> {
    let mut normalized = ask;

    // Remove 0 values
    normalized.native.retain(|c| c.amount.u128() != 0);

    let dup_check = |mut val: GenericBalance| -> Result<(), ContractError> {
        // Sort
        val.native.sort_unstable_by(|a, b| a.denom.cmp(&b.denom));
        // Length of original
        let len_pre_dedup = val.native.len();
        // Dedup
        val.native.dedup_by_key(|i| i.denom.clone());
        // Length after dedup
        let len_post_dedup = val.native.len();
        // If lengths different, dupes were found so return error
        if len_pre_dedup != len_post_dedup {
            return Err(ContractError::GenericError("Duplicates found in Ask Price".to_string()));
        };

        Ok(())
    };

    dup_check(normalized.clone())?;
    Ok(normalized)
}

pub fn calc_fee(
    this_contract_addr: &Addr,
    balance: &GenericBalance,
) -> StdResult<Option<(CosmosMsg, GenericBalance)>> {
    let juno_in_balance = balance.native.iter().find(|n| n.denom == *NATIVE);

    // If balance DOES NOT contain juno, return Ok(None)
    // If balance DOES contain juno, calculate 0.1% of the JUNO in the balance,
    // Create CosmosMsg sending that to the Community Pool,
    // and return this CosmosMsg + a generic balance with the fee removed for the user
    if let Some(juno) = juno_in_balance {
        // 0.1% = amount * 1 / 1000
        let ten_pips = juno.amount.multiply_ratio(1_u128, 1000_u128);

        // small amounts (like 1ujuno) will be 0
        if ten_pips.is_zero() {
            return Ok(None);
        }

        let fee_msg = MsgFundCommunityPool {
            amount: vec![SDKCoin {
                amount: ten_pips.u128().to_string(),
                denom: NATIVE.to_string(),
                special_fields: Default::default(),
            }],
            depositor: this_contract_addr.to_string(), // TODO: change to this contracts addr
            special_fields: Default::default(),
        };
        let pool_msg_bytes: Vec<u8> = fee_msg.write_to_bytes().unwrap_or_default();

        let final_msg: CosmosMsg = CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
            value: Binary::from(pool_msg_bytes),
        };

        let juno_amount_after_fee_removed = juno.amount.checked_sub(ten_pips)?;

        let balance_with_fee_removed = {
            let mut x = balance.clone();
            x.native.retain(|n| n.denom != *NATIVE);
            x.native.append(&mut coins(juno_amount_after_fee_removed.u128(), NATIVE));
            x
        };

        Ok(Some((final_msg, balance_with_fee_removed)))
    } else {
        Ok(None)
    }
}
