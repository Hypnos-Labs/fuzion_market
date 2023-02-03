use crate::error::ContractError;
use crate::state::GenericBalance;

use cosmwasm_std::{coins, StdError};
use cosmwasm_std::{to_binary, Addr, BankMsg, CosmosMsg, Empty, StdResult, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as SdkCoin;
use cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgFundCommunityPool;


// Actual community pool on mainnet
const COMMUNITY_POOL: &str = "juno1jv65s3grqf6v6jl3dp4t6c9t9rk99cd83d88wr";
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


// encode a protobuf into a cosmos message
// Inspired by https://github.com/alice-ltd/smart-contracts/blob/master/contracts/alice_terra_token/src/execute.rs#L73-L76
pub(crate) fn proto_encode<M: prost::Message>(msg: M, type_url: String) -> StdResult<CosmosMsg> {
    let mut bytes = Vec::new();
    prost::Message::encode(&msg, &mut bytes)
        .map_err(|_e| StdError::generic_err("Message encoding must be infallible"))?;
    Ok(cosmwasm_std::CosmosMsg::<cosmwasm_std::Empty>::Stargate {
        type_url,
        value: cosmwasm_std::Binary(bytes),
    })
}

pub fn calc_fee(balance: &GenericBalance, depositor: &Addr) -> StdResult<Option<(CosmosMsg, GenericBalance)>> {
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

        let fee_msg: CosmosMsg = proto_encode(
            MsgFundCommunityPool {
                amount: vec![SdkCoin {
                    denom: NATIVE.to_string(),
                    amount: ten_pips.to_string(),
                }],
                depositor: depositor.to_string(),
            },
            "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
        )?;

        let juno_amount_after_fee_removed = juno.amount.checked_sub(ten_pips)?;

        let balance_with_fee_removed = {
            let mut x = balance.clone();
            x.native.retain(|n| n.denom != *NATIVE);
            x.native.append(&mut coins(juno_amount_after_fee_removed.u128(), NATIVE));
            x
        };

        Ok(Some((fee_msg, balance_with_fee_removed)))
    } else {
        Ok(None)
    }
}
